//! Pari 3 du prototype : un hôte (la simulation, 300 zombies) et 3 invités passent par le
//! relais. L'hôte envoie 20 instantanés/s ; chaque invité les décode et mesure le temps
//! d'aller-retour invité → relais → hôte → relais → invité (10 pings/s).
//!
//!   cargo run --release -p bs-relais --example aller_retour [ws://hôte:port/salon] [secondes]
//!
//! Sans adresse, un relais local est lancé dans le programme.

use std::time::{Duration, Instant};

use bs_commun::PAS_SIMULATION;
use bs_protocole::{
    Instantane, Trame, VERSION_PROTOCOLE, ecrire_instantane, lire_instantane, lire_trame,
};
use bs_regles::{Commande, Parametres, Simulation};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

const INVITES: usize = 3;
/// Contenu entre hôte et invités : premier octet.
const INSTANTANE: u8 = 0;
const PING: u8 = 1;
const PONG: u8 = 2;

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connecter(url: &str, premiere: Trame<'_>) -> (Socket, [u8; 6], u8) {
    // nodelay aussi côté client (sinon Nagle : +40 ms sur un aller-retour sur dix)
    let (mut ws, _) = tokio_tungstenite::connect_async_with_config(url, None, true)
        .await
        .expect("connexion au relais");
    let mut o = Vec::new();
    premiere.ecrire(&mut o);
    ws.send(Message::Binary(o.into())).await.unwrap();
    let Some(Ok(Message::Binary(r))) = ws.next().await else {
        panic!("pas de réponse du relais")
    };
    match lire_trame(&r) {
        Some(Trame::Salon { code, place, .. }) => (ws, code, place),
        autre => panic!("refusé : {autre:?}"),
    }
}

/// Fin de partie : la simulation avance de 20 min avant la mesure (et on la chronomètre).
fn simulation_de_fin() -> Simulation {
    let mut s = Simulation::nouvelle(42, Parametres::default());
    let debut = Instant::now();
    let pas = 20 * 60 * 60;
    for k in 0..pas {
        let a = k as f32 * 0.004;
        s.avancer(Commande {
            dx: a.cos(),
            dz: a.sin(),
        });
    }
    let t = debut.elapsed().as_secs_f64();
    println!(
        "simulation : {pas} pas en {t:.2} s, soit {:.1} µs par pas (300 zombies, 1 arme)",
        t * 1e6 / pas as f64
    );
    s
}

async fn hote(mut ws: Socket, mut s: Simulation, secondes: u64) {
    let (mut contenu, mut corps, mut trame) = (Vec::new(), Vec::new(), Vec::new());
    let mut tic = tokio::time::interval(Duration::from_secs_f32(PAS_SIMULATION));
    let fin = Instant::now() + Duration::from_secs(secondes);
    let (mut envois, mut octets, mut max) = (0u64, 0u64, 0usize);
    while Instant::now() < fin {
        tokio::select! {
            _ = tic.tick() => {
                let a = s.pas as f32 * 0.004;
                s.avancer(Commande { dx: a.cos(), dz: a.sin() });
                if s.pas.is_multiple_of(3) {
                    contenu.clear();
                    contenu.push(INSTANTANE);
                    ecrire_instantane(&s, &mut corps);
                    contenu.extend_from_slice(&corps);
                    Trame::VersInvites { charge: &contenu }.ecrire(&mut trame);
                    (envois, octets, max) = (envois + 1, octets + trame.len() as u64, max.max(trame.len()));
                    ws.send(Message::Binary(trame.clone().into())).await.unwrap();
                }
            }
            m = ws.next() => if let Some(Ok(Message::Binary(o))) = m
                && let Some(Trame::VersHote { place, charge }) = lire_trame(&o)
                && charge.first() == Some(&PING)
            {
                let mut pong = vec![PONG, place];
                pong.extend_from_slice(&charge[1..]);
                Trame::VersInvites { charge: &pong }.ecrire(&mut trame);
                ws.send(Message::Binary(trame.clone().into())).await.unwrap();
            },
        }
    }
    println!(
        "hôte : {envois} instantanés, {:.0} octets en moyenne, {max} au plus, {:.1} Ko/s par invité",
        octets as f64 / envois as f64,
        octets as f64 / secondes as f64 / 1024.0
    );
}

async fn invite(mut ws: Socket, place: u8, depart: Instant, secondes: u64) -> (Vec<f64>, u64, f64) {
    let mut trame = Vec::new();
    let mut lu = Instantane::default();
    let (mut recus, mut decodage) = (0u64, 0f64);
    let mut allers_retours = Vec::new();
    let mut ping = tokio::time::interval(Duration::from_millis(100));
    let fin = Instant::now() + Duration::from_secs(secondes);
    while Instant::now() < fin {
        tokio::select! {
            _ = ping.tick() => {
                let t = depart.elapsed().as_nanos() as u64;
                let mut c = vec![PING];
                c.extend_from_slice(&t.to_le_bytes());
                Trame::VersHote { place: 0, charge: &c }.ecrire(&mut trame);
                if ws.send(Message::Binary(trame.clone().into())).await.is_err() { break }
            }
            m = ws.next() => match m {
                Some(Ok(Message::Binary(o))) => if let Some(Trame::VersInvites { charge }) = lire_trame(&o) {
                    match charge.first() {
                        Some(&INSTANTANE) => {
                            let d = Instant::now();
                            assert!(lire_instantane(&charge[1..], &mut lu));
                            decodage += d.elapsed().as_secs_f64();
                            recus += 1;
                        }
                        Some(&PONG) if charge[1] == place => {
                            let t = u64::from_le_bytes(charge[2..10].try_into().unwrap());
                            allers_retours.push((depart.elapsed().as_nanos() as u64 - t) as f64 / 1e6);
                        }
                        _ => {}
                    }
                },
                _ => break,
            },
        }
    }
    (allers_retours, recus, decodage * 1e6 / recus.max(1) as f64)
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let url = match args.next() {
        Some(u) => u,
        None => {
            let ecoute = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let adresse = ecoute.local_addr().unwrap();
            tokio::spawn(async move { bs_relais::servir(ecoute).await.unwrap() });
            format!("ws://{adresse}/salon")
        }
    };
    let secondes: u64 = args.next().map_or(10, |s| s.parse().unwrap());
    let s = simulation_de_fin();
    let mut corps = Vec::new();
    ecrire_instantane(&s, &mut corps);
    println!(
        "instantané de fin de partie : {} octets ({} zombies vivants, {} projectiles)",
        corps.len(),
        s.ennemis.vivant.iter().filter(|v| **v).count(),
        s.projectiles.nombre
    );

    let (ws_hote, code, _) = connecter(
        &url,
        Trame::Creer {
            version: VERSION_PROTOCOLE,
            public: false,
            annonce: &[],
        },
    )
    .await;
    println!("salon {} ouvert sur {url}", String::from_utf8_lossy(&code));
    let depart = Instant::now();
    let mut taches = Vec::new();
    for _ in 0..INVITES {
        let (ws, _, place) = connecter(
            &url,
            Trame::Rejoindre {
                version: VERSION_PROTOCOLE,
                code,
            },
        )
        .await;
        taches.push(tokio::spawn(invite(ws, place, depart, secondes)));
    }
    hote(ws_hote, s, secondes).await;
    let mut tous = Vec::new();
    for (k, t) in taches.into_iter().enumerate() {
        let (ar, recus, decodage) = t.await.unwrap();
        println!(
            "invité {} : {recus} instantanés reçus, décodage {decodage:.1} µs, {} allers-retours",
            k + 1,
            ar.len()
        );
        tous.extend(ar);
    }
    tous.sort_by(f64::total_cmp);
    let q = |p: f64| tous[((tous.len() - 1) as f64 * p) as usize];
    println!(
        "aller-retour invité → relais → hôte → relais → invité : médiane {:.2} ms, 90 % sous {:.2} ms, 99 % sous {:.2} ms, max {:.2} ms",
        q(0.5),
        q(0.9),
        q(0.99),
        q(1.0)
    );
}
