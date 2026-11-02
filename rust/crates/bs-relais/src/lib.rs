//! Relais de la coop : colle axum + tokio autour de la logique des salons (`salons`).
//! L'hôte ouvre un salon (code de 6 lettres), les invités le rejoignent ou le trouvent dans
//! la liste publique ; le relais fait suivre les octets de l'hôte à tous les invités et
//! ceux de chaque invité à l'hôte, sans les lire (il ne dépend que de `bs-protocole`).
//!
//! Une tâche par connexion : elle lit le WebSocket, passe chaque message à la logique (sous
//! un verrou) et écrit ce que la logique lui a déposé dans son canal. Elle envoie un Ping
//! toutes les `battement` et ferme une connexion silencieuse depuis `silence_max`. Une
//! tâche d'entretien libère chaque seconde les places réservées expirées et ferme les
//! salons abandonnés et oublie les présences d'amis muettes (`amis`). Routes : `/salon` (WebSocket) et `/etat` (JSON).

mod amis;
mod limites;
mod salons;

use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};
use std::time::Instant;

use axum::Router;
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade, close_code};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use bs_protocole::amis::lire_trame_ami;
use bs_protocole::{PLACES_SALON, VERSION_PROTOCOLE, lire_trame};
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

pub use limites::{
    BATTEMENT, DELAI_RECONNEXION, ENTRETIEN, FENETRE_TENTATIVES, INACTIVITE_SALON, MESSAGE_MAX,
    Reglages, SALONS_PAR_ADRESSE, SILENCE_MAX, TENTATIVES,
};
use salons::{Salons, Siege, Sortant};

/// Joueurs par salon, hôte compris.
pub const PLACES: usize = PLACES_SALON as usize;

/// État partagé par toutes les connexions.
struct Partage {
    salons: Mutex<Salons>,
    amis: Mutex<amis::Amis>,
    reglages: Reglages,
    connexions: AtomicUsize,
    ids: AtomicU64,
}

type Relais = Arc<Partage>;

impl Partage {
    /// La logique ne laisse jamais les salons incohérents : un verrou empoisonné (panique
    /// dans une autre tâche) reste utilisable.
    fn salons(&self) -> MutexGuard<'_, Salons> {
        self.salons.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn amis(&self) -> MutexGuard<'_, amis::Amis> {
        self.amis.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Sert le relais sur `ecoute` avec les vrais réglages.
pub async fn servir(ecoute: tokio::net::TcpListener) -> std::io::Result<()> {
    servir_avec(ecoute, Reglages::default()).await
}

/// Sert le relais sur `ecoute`, avec TCP_NODELAY : sans lui, Nagle retient les petits
/// messages (≈ 40 ms de plus par aller-retour).
pub async fn servir_avec(
    ecoute: tokio::net::TcpListener,
    reglages: Reglages,
) -> std::io::Result<()> {
    use axum::serve::ListenerExt;
    let ecoute = ecoute.tap_io(|tcp| {
        let _ = tcp.set_nodelay(true);
    });
    let service = routes_avec(reglages).into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(ecoute, service).await
}

/// Routes du relais avec les vrais réglages (voir `routes_avec`).
pub fn routes() -> Router {
    routes_avec(Reglages::default())
}

/// Routes du relais : `/salon` (WebSocket) et `/etat`. À appeler dans un runtime tokio
/// (lance la tâche d'entretien, arrêtée avec le routeur) et à servir avec
/// `into_make_service_with_connect_info::<SocketAddr>()` (limites par adresse).
pub fn routes_avec(reglages: Reglages) -> Router {
    let relais = Arc::new(Partage {
        salons: Mutex::new(Salons::nouveaux(reglages.clone())),
        amis: Mutex::new(amis::Amis::default()),
        reglages,
        connexions: AtomicUsize::new(0),
        ids: AtomicU64::new(0),
    });
    tokio::spawn(entretien(Arc::downgrade(&relais)));
    Router::new()
        .route("/salon", get(ouvrir))
        .route("/etat", get(etat))
        .with_state(relais)
}

async fn entretien(relais: Weak<Partage>) {
    let Some(periode) = relais.upgrade().map(|r| r.reglages.entretien) else {
        return;
    };
    let mut tic = tokio::time::interval(periode);
    tic.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        tic.tick().await;
        let Some(r) = relais.upgrade() else { return };
        r.salons().entretenir(Instant::now());
        r.amis().entretenir(Instant::now());
    }
}

/// Adresse du client : celle de l'en-tête `CF-Connecting-IP` derrière Cloudflare, sinon
/// celle de la connexion TCP.
fn adresse_client(entetes: &HeaderMap, pair: SocketAddr, cloudflare: bool) -> IpAddr {
    let cf = || {
        entetes
            .get("cf-connecting-ip")?
            .to_str()
            .ok()?
            .trim()
            .parse()
            .ok()
    };
    cloudflare.then(cf).flatten().unwrap_or(pair.ip())
}

async fn ouvrir(
    ws: WebSocketUpgrade,
    State(relais): State<Relais>,
    ConnectInfo(pair): ConnectInfo<SocketAddr>,
    entetes: HeaderMap,
) -> Response {
    let ip = adresse_client(&entetes, pair, relais.reglages.cloudflare);
    let max = relais.reglages.message_max;
    ws.max_message_size(max)
        .max_frame_size(max)
        .on_upgrade(move |socket| connexion(relais, socket, ip))
}

async fn etat(State(relais): State<Relais>) -> impl IntoResponse {
    let e = relais.salons().etat();
    let connexions = relais.connexions.load(Ordering::Relaxed);
    let (abonnes, presences) = relais.amis().etat();
    let corps = format!(
        "{{\"protocole\":{VERSION_PROTOCOLE},\"salons\":{},\"publics\":{},\"joueurs\":{},\"reservees\":{},\"connexions\":{connexions},\"amis\":{abonnes},\"presences\":{presences}}}",
        e.salons, e.publics, e.joueurs, e.reservees
    );
    ([(header::CONTENT_TYPE, "application/json")], corps)
}

/// Vie d'une connexion : accueil (lister, créer, rejoindre, reprendre), puis échanges
/// dans le salon jusqu'à la coupure.
async fn connexion(relais: Relais, mut socket: WebSocket, ip: IpAddr) {
    relais.connexions.fetch_add(1, Ordering::Relaxed);
    let id = relais.ids.fetch_add(1, Ordering::Relaxed);
    let (envoi, mut a_envoyer) = mpsc::unbounded_channel();
    let mut siege: Option<Siege> = None;
    let r = &relais.reglages;
    let debut = tokio::time::Instant::now();
    let mut battement = tokio::time::interval_at(debut + r.battement, r.battement);
    battement.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut dernier_recu = debut;
    let volontaire = loop {
        tokio::select! {
            sortant = a_envoyer.recv() => match sortant {
                Some(Sortant::Octets(o)) => if socket.send(Message::Binary(o)).await.is_err() {
                    break false;
                },
                // Exclusion, salon fermé ou place reprise par une autre connexion
                Some(Sortant::Fermer) | None => {
                    let _ = socket.send(Message::Close(None)).await;
                    break false;
                }
            },
            entrant = socket.recv() => {
                dernier_recu = tokio::time::Instant::now();
                match entrant {
                    Some(Ok(Message::Binary(o))) => {
                        let maintenant = Instant::now();
                        match siege {
                            Some(s) => relais.salons().recevoir(s, id, o, maintenant),
                            None => if let Some(t) = lire_trame(&o) {
                                siege = relais.salons().accueillir(t, ip, id, &envoi, maintenant);
                            } else if let Some(t) = lire_trame_ami(&o) {
                                // Amis : sur une connexion sans salon
                                relais.amis().recevoir(t, &o, id, &envoi, maintenant);
                            } else {
                                // Trame illisible avant d'avoir un salon : on raccroche
                                break false;
                            },
                        }
                    }
                    // Fermeture normale (code 1000) : départ volontaire, place libérée
                    Some(Ok(Message::Close(c))) => {
                        break c.is_some_and(|c| c.code == close_code::NORMAL);
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => break false,
                }
            }
            _ = battement.tick() => {
                if dernier_recu.elapsed() >= r.silence_max
                    || socket.send(Message::Ping(Bytes::new())).await.is_err()
                {
                    break false;
                }
            }
        }
    };
    relais.amis().deconnecter(id);
    if let Some(s) = siege {
        relais
            .salons()
            .deconnecter(s, id, volontaire, Instant::now());
    }
    relais.connexions.fetch_sub(1, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adresse_cloudflare() {
        let pair: SocketAddr = "10.0.0.1:5000".parse().unwrap();
        let mut h = HeaderMap::new();
        assert_eq!(adresse_client(&h, pair, true), pair.ip());
        h.insert("cf-connecting-ip", "2001:db8::7".parse().unwrap());
        let v6: IpAddr = "2001:db8::7".parse().unwrap();
        assert_eq!(adresse_client(&h, pair, true), v6);
        assert_eq!(adresse_client(&h, pair, false), pair.ip());
        h.insert("cf-connecting-ip", "n'importe quoi".parse().unwrap());
        assert_eq!(adresse_client(&h, pair, true), pair.ip());
    }
}
