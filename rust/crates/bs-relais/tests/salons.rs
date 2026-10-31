//! Le relais de bout en bout (sur un port local) : salon, refus, transmission dans les
//! deux sens sans toucher aux octets.

use bs_protocole::{REFUS_SALON, REFUS_VERSION, Trame, VERSION_PROTOCOLE, lire_trame};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn relais() -> String {
    let ecoute = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let adresse = ecoute.local_addr().unwrap();
    tokio::spawn(bs_relais::servir(ecoute));
    format!("ws://{adresse}/salon")
}

async fn envoyer(ws: &mut Socket, t: Trame<'_>) {
    let mut o = Vec::new();
    t.ecrire(&mut o);
    ws.send(Message::Binary(o.into())).await.unwrap();
}

async fn recevoir(ws: &mut Socket) -> Vec<u8> {
    match ws.next().await {
        Some(Ok(Message::Binary(o))) => o.to_vec(),
        autre => panic!("attendu une trame : {autre:?}"),
    }
}

async fn client(url: &str, t: Trame<'_>) -> (Socket, Vec<u8>) {
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    envoyer(&mut ws, t).await;
    let r = recevoir(&mut ws).await;
    (ws, r)
}

#[tokio::test]
async fn salon_et_transmission() {
    let url = relais().await;
    let (mut hote, r) = client(
        &url,
        Trame::Creer {
            version: VERSION_PROTOCOLE,
        },
    )
    .await;
    let Some(Trame::Salon { code, place: 0 }) = lire_trame(&r) else {
        panic!("{r:?}")
    };
    let (mut invite, r) = client(
        &url,
        Trame::Rejoindre {
            version: VERSION_PROTOCOLE,
            code,
        },
    )
    .await;
    assert_eq!(lire_trame(&r), Some(Trame::Salon { code, place: 1 }));

    // Invité → hôte : le relais inscrit la vraie place, quoi qu'annonce l'invité
    envoyer(
        &mut invite,
        Trame::VersHote {
            place: 3,
            charge: b"bonjour",
        },
    )
    .await;
    assert_eq!(
        lire_trame(&recevoir(&mut hote).await),
        Some(Trame::VersHote {
            place: 1,
            charge: b"bonjour"
        })
    );
    // Hôte → invités, octets intacts
    envoyer(
        &mut hote,
        Trame::VersInvites {
            charge: &[0, 255, 7],
        },
    )
    .await;
    assert_eq!(
        lire_trame(&recevoir(&mut invite).await),
        Some(Trame::VersInvites {
            charge: &[0, 255, 7]
        })
    );
}

#[tokio::test]
async fn refus() {
    let url = relais().await;
    let (_, r) = client(
        &url,
        Trame::Creer {
            version: VERSION_PROTOCOLE + 1,
        },
    )
    .await;
    assert_eq!(
        lire_trame(&r),
        Some(Trame::Refus {
            raison: REFUS_VERSION
        })
    );
    let (_, r) = client(
        &url,
        Trame::Rejoindre {
            version: VERSION_PROTOCOLE,
            code: *b"ZZZZZZ",
        },
    )
    .await;
    assert_eq!(
        lire_trame(&r),
        Some(Trame::Refus {
            raison: REFUS_SALON
        })
    );
}
