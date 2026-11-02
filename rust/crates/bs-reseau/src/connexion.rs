//! Une WebSocket vers le relais, tenue par un fil à part (tungstenite, TLS par rustls) :
//! le fil se connecte, envoie ce qu'on lui confie et fait remonter ce qu'il reçoit par des
//! canaux ; le jeu n'attend jamais. Lecture avec un court délai pour rester réactif à
//! l'envoi ; TCP_NODELAY (sinon Nagle retient les petits messages). Fermer envoie le code
//! 1000 : pour le relais, c'est un départ volontaire (la place est libérée).

use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::time::Duration;

use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Error, Message, WebSocket};

/// Attente maximale d'une lecture avant de regarder s'il y a à envoyer.
const DELAI_LECTURE: Duration = Duration::from_millis(3);

/// Ce que la connexion fait remonter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recu {
    /// Connectée au relais.
    Ouverte,
    /// Un message binaire.
    Octets(Vec<u8>),
    /// Connexion finie (raison, si ce n'est pas une fermeture normale).
    Fermee(Option<String>),
}

enum Ordre {
    Envoyer(Vec<u8>),
    Fermer,
    /// Coupure brutale, sans fermeture normale (essais de reprise).
    Couper,
}

pub struct Connexion {
    ordres: Sender<Ordre>,
    recus: Receiver<Recu>,
    finie: bool,
}

impl Connexion {
    /// Se connecte à `adresse` (ws:// ou wss://) dans un nouveau fil.
    pub fn ouvrir(adresse: &str) -> Self {
        let (ordres, ordres_fil) = channel();
        let (recus_fil, recus) = channel();
        let adresse = adresse.to_owned();
        let lance = std::thread::Builder::new()
            .name("relais".into())
            .spawn(move || boucle(&adresse, &ordres_fil, &recus_fil));
        let recus = match lance {
            Ok(_) => recus,
            Err(e) => {
                // Le fil n'a pas démarré : la connexion est finie d'emblée
                let (envoi, recus) = channel();
                let _ = envoi.send(Recu::Fermee(Some(e.to_string())));
                recus
            }
        };
        Self {
            ordres,
            recus,
            finie: false,
        }
    }

    /// Confie un message à envoyer (gardé jusqu'à l'ouverture).
    pub fn envoyer(&self, octets: Vec<u8>) {
        let _ = self.ordres.send(Ordre::Envoyer(octets));
    }

    /// Relève le prochain évènement, sans attendre. Un fil arrêté donne `Fermee` une fois.
    pub fn recevoir(&mut self) -> Option<Recu> {
        if self.finie {
            return None;
        }
        match self.recus.try_recv() {
            Ok(r) => {
                self.finie = matches!(r, Recu::Fermee(_));
                Some(r)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.finie = true;
                Some(Recu::Fermee(Some("fil du relais arrêté".into())))
            }
        }
    }

    /// Ferme proprement (départ volontaire).
    pub fn fermer(&self) {
        let _ = self.ordres.send(Ordre::Fermer);
    }

    /// Coupe sans fermeture normale, comme une perte de réseau (essais).
    #[doc(hidden)]
    pub fn couper(&self) {
        let _ = self.ordres.send(Ordre::Couper);
    }

    pub fn finie(&self) -> bool {
        self.finie
    }
}

impl Drop for Connexion {
    fn drop(&mut self) {
        self.fermer();
    }
}

fn tcp(ws: &WebSocket<MaybeTlsStream<TcpStream>>) -> Option<&TcpStream> {
    match ws.get_ref() {
        MaybeTlsStream::Plain(s) => Some(s),
        MaybeTlsStream::Rustls(s) => Some(s.get_ref()),
        _ => None,
    }
}

fn passager(e: &Error) -> bool {
    matches!(e, Error::Io(io) if matches!(io.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut))
}

fn boucle(adresse: &str, ordres: &Receiver<Ordre>, recus: &Sender<Recu>) {
    let mut ws = match tungstenite::connect(adresse) {
        Ok((ws, _)) => ws,
        Err(e) => {
            let _ = recus.send(Recu::Fermee(Some(e.to_string())));
            return;
        }
    };
    if let Some(s) = tcp(&ws) {
        let _ = s.set_nodelay(true);
        let _ = s.set_read_timeout(Some(DELAI_LECTURE));
    }
    let _ = recus.send(Recu::Ouverte);
    let raison = loop {
        // Envois en attente
        let mut fermer = false;
        loop {
            match ordres.try_recv() {
                Ok(Ordre::Envoyer(o)) => {
                    if let Err(e) = ws.write(Message::binary(o))
                        && !passager(&e)
                    {
                        let _ = recus.send(Recu::Fermee(Some(e.to_string())));
                        return;
                    }
                }
                Ok(Ordre::Fermer) | Err(TryRecvError::Disconnected) => {
                    fermer = true;
                    break;
                }
                Ok(Ordre::Couper) => {
                    let _ = recus.send(Recu::Fermee(Some("coupure".into())));
                    return;
                }
                Err(TryRecvError::Empty) => break,
            }
        }
        if fermer {
            fermer_proprement(&mut ws);
            break None;
        }
        if let Err(e) = ws.flush()
            && !passager(&e)
        {
            break Some(e.to_string());
        }
        match ws.read() {
            Ok(Message::Binary(o)) => {
                if recus.send(Recu::Octets(o.to_vec())).is_err() {
                    fermer_proprement(&mut ws);
                    break None;
                }
            }
            Ok(Message::Close(_)) | Err(Error::ConnectionClosed | Error::AlreadyClosed) => {
                break None;
            }
            Ok(_) => {}
            Err(e) if passager(&e) => {}
            Err(e) => break Some(e.to_string()),
        }
    };
    let _ = recus.send(Recu::Fermee(raison));
}

/// Envoie la fermeture normale et attend brièvement la réponse du relais.
fn fermer_proprement(ws: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
    let _ = ws.close(Some(CloseFrame {
        code: CloseCode::Normal,
        reason: "".into(),
    }));
    for _ in 0..100 {
        match ws.read() {
            Err(e) if passager(&e) => {}
            Err(_) => return,
            Ok(_) => {}
        }
        if ws.flush().is_err() {
            return;
        }
    }
}
