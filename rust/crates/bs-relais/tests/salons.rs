//! Le relais de bout en bout (sur un port local) : salon, refus, transmission dans les
//! deux sens sans toucher aux octets, reconnexions, exclusion, liste publique, battement
//! de cœur et page d'état.

use std::time::Duration;

use bs_protocole::{
    PLACES_SALON, REFUS_SALON, REFUS_VERSION, Trame, VERSION_PROTOCOLE, lire_trame,
};
use bs_relais::Reglages;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::tungstenite::Message;

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

const V: u16 = VERSION_PROTOCOLE;

/// Lance un relais ; renvoie l'adresse `hôte:port`.
async fn relais(reglages: Reglages) -> String {
    let ecoute = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let adresse = ecoute.local_addr().unwrap();
    tokio::spawn(bs_relais::servir_avec(ecoute, reglages));
    adresse.to_string()
}

async fn envoyer(ws: &mut Socket, t: Trame<'_>) {
    let mut o = Vec::new();
    t.ecrire(&mut o);
    ws.send(Message::Binary(o.into())).await.unwrap();
}

/// Prochain message binaire (les Ping/Pong sont passés).
async fn recevoir(ws: &mut Socket) -> Vec<u8> {
    let attente = async {
        loop {
            match ws.next().await {
                Some(Ok(Message::Binary(o))) => return o.to_vec(),
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                autre => panic!("attendu une trame : {autre:?}"),
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(5), attente)
        .await
        .expect("rien reçu")
}

async fn attendre(ws: &mut Socket, t: Trame<'_>) {
    assert_eq!(lire_trame(&recevoir(ws).await), Some(t));
}

/// La connexion se termine (fermeture, ou coupure) sans autre trame.
async fn attendre_fin(ws: &mut Socket) {
    let fin = async {
        loop {
            match ws.next().await {
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                Some(Ok(Message::Close(_)) | Err(_)) | None => return,
                autre => panic!("attendu la fin : {autre:?}"),
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(5), fin)
        .await
        .expect("connexion pas fermée");
}

async fn connecter(adresse: &str) -> Socket {
    let url = format!("ws://{adresse}/salon");
    tokio_tungstenite::connect_async(url).await.unwrap().0
}

/// Connexion + première trame ; renvoie la réponse.
async fn client(adresse: &str, t: Trame<'_>) -> (Socket, Vec<u8>) {
    let mut ws = connecter(adresse).await;
    envoyer(&mut ws, t).await;
    let r = recevoir(&mut ws).await;
    (ws, r)
}

/// Admis dans un salon : (connexion, code, place, jeton).
async fn admis(adresse: &str, t: Trame<'_>) -> (Socket, [u8; 6], u8, u64) {
    let (ws, r) = client(adresse, t).await;
    match lire_trame(&r) {
        Some(Trame::Salon { code, place, jeton }) => (ws, code, place, jeton),
        autre => panic!("pas admis : {autre:?}"),
    }
}

fn creer(public: bool, annonce: &[u8]) -> Trame<'_> {
    Trame::Creer {
        version: V,
        public,
        annonce,
    }
}

/// Un hôte et `n` invités ; renvoie aussi les jetons (hôte d'abord).
async fn salon(adresse: &str, n: u8) -> (Socket, Vec<Socket>, [u8; 6], Vec<u64>) {
    let (mut hote, code, place, jeton) = admis(adresse, creer(true, b"plaine")).await;
    assert_eq!(place, 0);
    let (mut invites, mut jetons) = (Vec::new(), vec![jeton]);
    for k in 1..=n {
        let (ws, c, place, jeton) = admis(adresse, Trame::Rejoindre { version: V, code }).await;
        assert_eq!((c, place), (code, k));
        attendre(&mut hote, Trame::Arrivee { place: k }).await;
        invites.push(ws);
        jetons.push(jeton);
    }
    (hote, invites, code, jetons)
}

#[tokio::test]
async fn hote_et_deux_invites_echangent() {
    let a = relais(Reglages::default()).await;
    let (mut hote, mut inv, _, _) = salon(&a, 2).await;
    // Invité → hôte : le relais inscrit la vraie place, quoi qu'annonce l'invité
    let vers_hote = Trame::VersHote {
        place: 3,
        charge: b"bonjour",
    };
    envoyer(&mut inv[1], vers_hote).await;
    let attendu = Trame::VersHote {
        place: 2,
        charge: b"bonjour",
    };
    attendre(&mut hote, attendu).await;
    // Hôte → tous les invités, octets intacts
    let charge = Trame::VersInvites {
        charge: &[0, 255, 7],
    };
    envoyer(&mut hote, charge).await;
    for ws in &mut inv {
        attendre(ws, charge).await;
    }
}

#[tokio::test]
async fn refus() {
    let a = relais(Reglages::default()).await;
    let (_, r) = client(&a, Trame::Lister { version: V + 1 }).await;
    assert!(matches!(lire_trame(&r), Some(Trame::Liste { salons }) if salons.est_vide()));
    let (_, r) = client(
        &a,
        Trame::Creer {
            version: V + 1,
            public: false,
            annonce: &[],
        },
    )
    .await;
    let refus = Trame::Refus {
        raison: REFUS_VERSION,
    };
    assert_eq!(lire_trame(&r), Some(refus));
    let inconnu = Trame::Rejoindre {
        version: V,
        code: *b"ZZZZZZ",
    };
    let (_, r) = client(&a, inconnu).await;
    let refus = Trame::Refus {
        raison: REFUS_SALON,
    };
    assert_eq!(lire_trame(&r), Some(refus));
}

#[tokio::test]
async fn invite_coupe_et_revient() {
    let a = relais(Reglages::default()).await;
    let (mut hote, mut inv, code, jetons) = salon(&a, 2).await;
    // Coupure brutale (pas de fermeture WebSocket) : place réservée
    drop(inv.remove(0));
    attendre(&mut hote, Trame::Depart { place: 1 }).await;
    let reprise = Trame::Reprendre {
        version: V,
        code,
        jeton: jetons[1],
    };
    let (mut retour, c, place, jeton) = admis(&a, reprise).await;
    assert_eq!((c, place, jeton), (code, 1, jetons[1]));
    attendre(&mut hote, Trame::Arrivee { place: 1 }).await;
    let charge = Trame::VersInvites { charge: b"complet" };
    envoyer(&mut hote, charge).await;
    attendre(&mut retour, charge).await;
}

#[tokio::test]
async fn hote_coupe_et_revient() {
    let a = relais(Reglages::default()).await;
    let (hote, mut inv, code, jetons) = salon(&a, 2).await;
    drop(hote);
    for ws in &mut inv {
        attendre(ws, Trame::HotePerdu).await;
    }
    let reprise = Trame::Reprendre {
        version: V,
        code,
        jeton: jetons[0],
    };
    let (mut hote, _, place, _) = admis(&a, reprise).await;
    assert_eq!(place, 0);
    for ws in &mut inv {
        attendre(ws, Trame::HoteRevenu).await;
    }
    attendre(&mut hote, Trame::Arrivee { place: 1 }).await;
    attendre(&mut hote, Trame::Arrivee { place: 2 }).await;
    envoyer(
        &mut inv[0],
        Trame::VersHote {
            place: 0,
            charge: &[1],
        },
    )
    .await;
    attendre(
        &mut hote,
        Trame::VersHote {
            place: 1,
            charge: &[1],
        },
    )
    .await;
}

#[tokio::test]
async fn hote_ne_revient_pas() {
    let reglages = Reglages {
        delai_reconnexion: Duration::from_millis(200),
        entretien: Duration::from_millis(20),
        ..Reglages::default()
    };
    let a = relais(reglages).await;
    let (hote, mut inv, code, jetons) = salon(&a, 1).await;
    drop(hote);
    attendre(&mut inv[0], Trame::HotePerdu).await;
    attendre(&mut inv[0], Trame::Ferme).await;
    attendre_fin(&mut inv[0]).await;
    let reprise = Trame::Reprendre {
        version: V,
        code,
        jeton: jetons[0],
    };
    let (_, r) = client(&a, reprise).await;
    let refus = Trame::Refus {
        raison: REFUS_SALON,
    };
    assert_eq!(lire_trame(&r), Some(refus));
}

#[tokio::test]
async fn exclusion() {
    let a = relais(Reglages::default()).await;
    let (mut hote, mut inv, _, _) = salon(&a, 2).await;
    envoyer(&mut hote, Trame::Exclure { place: 1 }).await;
    attendre(&mut inv[0], Trame::Exclu).await;
    attendre_fin(&mut inv[0]).await;
    attendre(&mut hote, Trame::Depart { place: 1 }).await;
    // L'autre invité est toujours là
    let charge = Trame::VersInvites { charge: &[4] };
    envoyer(&mut hote, charge).await;
    attendre(&mut inv[1], charge).await;
}

#[tokio::test]
async fn liste_publique_puis_rejoindre() {
    let a = relais(Reglages::default()).await;
    let (mut hote, _inv, code, _) = salon(&a, 1).await;
    envoyer(&mut hote, Trame::Annoncer { annonce: b"foret" }).await;
    let (_prive, ..) = admis(&a, creer(false, b"secret")).await;
    // L'annonce est prise en compte avant la demande de liste (connexions différentes)
    tokio::time::sleep(Duration::from_millis(50)).await;
    let (mut ws, r) = client(&a, Trame::Lister { version: V }).await;
    let Some(Trame::Liste { salons }) = lire_trame(&r) else {
        panic!("{r:?}")
    };
    let lus: Vec<_> = salons.iter().collect();
    assert_eq!(lus.len(), 1);
    assert_eq!(
        (lus[0].code, lus[0].joueurs, lus[0].places, lus[0].annonce),
        (code, 2, PLACES_SALON, &b"foret"[..])
    );
    // Même connexion : rejoindre le salon trouvé
    envoyer(&mut ws, Trame::Rejoindre { version: V, code }).await;
    let r = recevoir(&mut ws).await;
    assert!(matches!(
        lire_trame(&r),
        Some(Trame::Salon { place: 2, .. })
    ));
    attendre(&mut hote, Trame::Arrivee { place: 2 }).await;
}

#[tokio::test]
async fn battement_de_coeur() {
    let reglages = Reglages {
        battement: Duration::from_millis(50),
        silence_max: Duration::from_millis(200),
        ..Reglages::default()
    };
    let a = relais(reglages).await;
    // Un client qui lit répond aux Ping (tungstenite) : il reste connecté
    let mut vivant = connecter(&a).await;
    let lire = async {
        loop {
            match vivant.next().await {
                Some(Ok(Message::Ping(_))) => {}
                autre => return autre,
            }
        }
    };
    let lecture = tokio::time::timeout(Duration::from_millis(500), lire).await;
    assert!(lecture.is_err(), "rien d'autre que des Ping : {lecture:?}");
    let (mut hote, ..) = admis(&a, creer(false, &[])).await;
    envoyer(&mut vivant, Trame::Lister { version: V }).await;
    assert!(matches!(
        lire_trame(&recevoir(&mut vivant).await),
        Some(Trame::Liste { .. })
    ));
    // Un client muet (ne lit pas, donc ne répond pas aux Ping) est coupé
    tokio::time::sleep(Duration::from_millis(500)).await;
    attendre_fin(&mut hote).await;
}

#[tokio::test]
async fn page_etat() {
    let a = relais(Reglages::default()).await;
    let (_hote, _inv, _, _) = salon(&a, 1).await;
    let mut tcp = tokio::net::TcpStream::connect(&a).await.unwrap();
    tcp.write_all(b"GET /etat HTTP/1.1\r\nHost: relais\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut reponse = String::new();
    tcp.read_to_string(&mut reponse).await.unwrap();
    assert!(reponse.starts_with("HTTP/1.1 200"), "{reponse}");
    let attendu = format!(
        "{{\"protocole\":{V},\"salons\":1,\"publics\":1,\"joueurs\":2,\"reservees\":0,\"connexions\":2,\"amis\":0,\"presences\":0}}"
    );
    assert!(reponse.ends_with(&attendu), "{reponse}");
}
