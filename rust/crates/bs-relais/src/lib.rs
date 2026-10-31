//! Relais de la coop : l'hôte ouvre un salon (code de 6 lettres), les invités le
//! rejoignent ; le relais fait suivre les octets de l'hôte à tous les invités et ceux de
//! chaque invité à l'hôte, sans les lire (il ne dépend que de `bs-protocole`). Un client
//! d'une autre version du protocole est refusé.
//!
//! Prototype (étape 0 bis) : salons en mémoire, pas encore de reconnexion ni de liste
//! publique (étape 6 du plan).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use bs_protocole::{REFUS_PLEIN, REFUS_SALON, REFUS_VERSION, Trame, VERSION_PROTOCOLE, lire_trame};
use tokio::sync::mpsc;

/// Joueurs par salon, hôte compris.
pub const PLACES: usize = 4;

type Envoi = mpsc::UnboundedSender<Message>;

#[derive(Default)]
struct Salon {
    hote: Option<Envoi>,
    /// Place 1 à PLACES-1 (indice + 1).
    invites: [Option<Envoi>; PLACES - 1],
}

#[derive(Default, Clone)]
pub struct Relais {
    salons: Arc<Mutex<HashMap<[u8; 6], Salon>>>,
}

/// Sert le relais sur `ecoute`, avec TCP_NODELAY : sans lui, Nagle retient les petits
/// messages (≈ 40 ms de plus par aller-retour).
pub async fn servir(ecoute: tokio::net::TcpListener) -> std::io::Result<()> {
    use axum::serve::ListenerExt;
    let ecoute = ecoute.tap_io(|tcp| {
        let _ = tcp.set_nodelay(true);
    });
    axum::serve(ecoute, routes()).await
}

/// Routes du relais : `/salon` (WebSocket).
pub fn routes() -> Router {
    Router::new()
        .route("/salon", get(ouvrir))
        .with_state(Relais::default())
}

async fn ouvrir(ws: WebSocketUpgrade, State(relais): State<Relais>) -> Response {
    ws.on_upgrade(move |socket| relais.servir(socket))
}

/// Code de salon tiré au hasard (lettres sans I ni O, pour éviter les confusions).
fn nouveau_code(graine: u64) -> [u8; 6] {
    const LETTRES: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
    let mut x = graine | 1;
    core::array::from_fn(|_| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        LETTRES[(x % LETTRES.len() as u64) as usize]
    })
}

impl Relais {
    async fn servir(self, mut socket: WebSocket) {
        // Première trame : créer ou rejoindre
        let Some(Ok(Message::Binary(premiere))) = socket.recv().await else {
            return;
        };
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        let mut tampon = Vec::new();
        let place = match lire_trame(&premiere) {
            Some(Trame::Creer { version }) | Some(Trame::Rejoindre { version, .. })
                if version != VERSION_PROTOCOLE =>
            {
                Err(REFUS_VERSION)
            }
            Some(Trame::Creer { .. }) => Ok(self.creer(tx)),
            Some(Trame::Rejoindre { code, .. }) => self.rejoindre(code, tx),
            _ => return,
        };
        let (code, place) = match place {
            Ok(p) => p,
            Err(raison) => {
                Trame::Refus { raison }.ecrire(&mut tampon);
                let _ = socket.send(Message::Binary(tampon.into())).await;
                return;
            }
        };
        Trame::Salon { code, place }.ecrire(&mut tampon);
        if socket
            .send(Message::Binary(tampon.clone().into()))
            .await
            .is_err()
        {
            self.quitter(code, place);
            return;
        }
        loop {
            tokio::select! {
                sortant = rx.recv() => match sortant {
                    Some(m) => if socket.send(m).await.is_err() { break },
                    None => break,
                },
                entrant = socket.recv() => match entrant {
                    Some(Ok(Message::Binary(o))) => self.faire_suivre(code, place, &o, &mut tampon),
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                    _ => {}
                },
            }
        }
        self.quitter(code, place);
    }

    fn creer(&self, tx: Envoi) -> ([u8; 6], u8) {
        let mut salons = self.salons.lock().unwrap();
        let mut graine = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.as_nanos() as u64);
        let mut code = nouveau_code(graine);
        while salons.contains_key(&code) {
            graine = graine.wrapping_add(0x9e37_79b9_7f4a_7c15);
            code = nouveau_code(graine);
        }
        salons.insert(
            code,
            Salon {
                hote: Some(tx),
                ..Salon::default()
            },
        );
        (code, 0)
    }

    fn rejoindre(&self, code: [u8; 6], tx: Envoi) -> Result<([u8; 6], u8), u8> {
        let mut salons = self.salons.lock().unwrap();
        let salon = salons
            .get_mut(&code)
            .filter(|s| s.hote.is_some())
            .ok_or(REFUS_SALON)?;
        let libre = salon
            .invites
            .iter()
            .position(Option::is_none)
            .ok_or(REFUS_PLEIN)?;
        salon.invites[libre] = Some(tx);
        Ok((code, libre as u8 + 1))
    }

    /// Hôte → tous les invités ; invité → hôte (avec sa place). Le reste est ignoré.
    fn faire_suivre(&self, code: [u8; 6], place: u8, octets: &[u8], tampon: &mut Vec<u8>) {
        let salons = self.salons.lock().unwrap();
        let Some(salon) = salons.get(&code) else {
            return;
        };
        match lire_trame(octets) {
            Some(Trame::VersInvites { .. }) if place == 0 => {
                // Même message pour tous : les octets sont partagés, pas copiés
                let m = Message::Binary(octets.to_vec().into());
                for invite in salon.invites.iter().flatten() {
                    let _ = invite.send(m.clone());
                }
            }
            Some(Trame::VersHote { charge, .. }) if place > 0 => {
                if let Some(hote) = &salon.hote {
                    Trame::VersHote { place, charge }.ecrire(tampon);
                    let _ = hote.send(Message::Binary(tampon.clone().into()));
                }
            }
            _ => {}
        }
    }

    /// Le départ de l'hôte ferme le salon (les invités sont déconnectés).
    fn quitter(&self, code: [u8; 6], place: u8) {
        let mut salons = self.salons.lock().unwrap();
        if place == 0 {
            salons.remove(&code);
        } else if let Some(s) = salons.get_mut(&code) {
            s.invites[place as usize - 1] = None;
        }
    }
}
