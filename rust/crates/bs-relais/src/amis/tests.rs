//! Tests des amis sur le relais, sans réseau : chaque client est un canal.

use bs_protocole::amis::{Genre, empreinte_boite, empreinte_presence, lire_trame_ami, signer};
use ed25519_dalek::SigningKey;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use super::*;

struct Client {
    id: u64,
    envoi: Envoi,
    recu: UnboundedReceiver<Sortant>,
    cle: SigningKey,
    code: &'static str,
}

impl Client {
    fn nouveau(id: u64, code: &'static str) -> Self {
        let (envoi, recu) = unbounded_channel();
        let cle = SigningKey::from_bytes(&[id as u8 + 1; 32]);
        Self {
            id,
            envoi,
            recu,
            cle,
            code,
        }
    }

    fn envoyer(&self, a: &mut Amis, t: TrameAmi, quand: Instant) {
        let mut o = Vec::new();
        t.ecrire(&mut o);
        a.recevoir(lire_trame_ami(&o).unwrap(), &o, self.id, &self.envoi, quand);
    }

    fn ecouter(&self, a: &mut Amis, amis: &[&str], quand: Instant) {
        let l: Vec<u8> = amis.iter().flat_map(|c| empreinte_presence(c)).collect();
        let t = TrameAmi::Ecouter {
            boite: empreinte_boite(self.code),
            suivis: &l,
        };
        self.envoyer(a, t, quand);
    }

    fn publier(&self, a: &mut Amis, cle: &SigningKey, instant: u64, quand: Instant) {
        let empreinte = empreinte_presence(self.code);
        let signe = signer(cle, Genre::Presence, &empreinte, instant, b"present");
        self.envoyer(a, TrameAmi::Publier { empreinte, signe }, quand);
    }

    fn poster(&self, a: &mut Amis, vers: &str, quand: Instant) {
        let boite = empreinte_boite(vers);
        let signe = signer(&self.cle, Genre::Courrier, &boite, 1, b"invitation");
        self.envoyer(a, TrameAmi::Poster { boite, signe }, quand);
    }

    /// Trames reçues depuis la dernière fois.
    fn recues(&mut self) -> Vec<Vec<u8>> {
        let mut v = Vec::new();
        while let Ok(Sortant::Octets(o)) = self.recu.try_recv() {
            v.push(o.to_vec());
        }
        v
    }
}

fn genre(o: &[u8]) -> &'static str {
    match lire_trame_ami(o) {
        Some(TrameAmi::Presence { .. }) => "presence",
        Some(TrameAmi::Courrier { .. }) => "courrier",
        Some(TrameAmi::Absent { .. }) => "absent",
        _ => "autre",
    }
}

fn genres(c: &mut Client) -> Vec<&'static str> {
    c.recues().iter().map(|o| genre(o)).collect()
}

#[test]
fn presence_suivie_puis_absente() {
    let (mut a, t0) = (Amis::default(), Instant::now());
    let alex = Client::nouveau(1, "AAA-AAA-AAA");
    let mut steve = Client::nouveau(2, "BBB-BBB-BBB");
    alex.publier(&mut a, &alex.cle, 1, t0);
    // Steve s'abonne après : il reçoit la présence gardée
    steve.ecouter(&mut a, &["AAA-AAA-AAA"], t0);
    assert_eq!(genres(&mut steve), ["presence"]);
    alex.publier(&mut a, &alex.cle, 2, t0);
    assert_eq!(genres(&mut steve), ["presence"]);
    assert_eq!(a.etat(), (2, 1));
    // Alex se déconnecte : Steve l'apprend
    a.deconnecter(alex.id);
    assert_eq!(genres(&mut steve), ["absent"]);
    assert_eq!(a.etat(), (1, 0));
}

#[test]
fn cle_liee_et_rejeu() {
    let (mut a, t0) = (Amis::default(), Instant::now());
    let alex = Client::nouveau(1, "AAA-AAA-AAA");
    let imposteur = Client::nouveau(3, "AAA-AAA-AAA");
    let mut steve = Client::nouveau(2, "BBB-BBB-BBB");
    steve.ecouter(&mut a, &["AAA-AAA-AAA"], t0);
    alex.publier(&mut a, &alex.cle, 5, t0);
    assert_eq!(genres(&mut steve), ["presence"]);
    // Une autre clé sous la même empreinte : ignorée
    imposteur.publier(&mut a, &imposteur.cle, 9, t0);
    // Un instant déjà vu (rejeu) : ignoré
    alex.publier(&mut a, &alex.cle, 5, t0);
    assert!(steve.recues().is_empty());
    // Signature fausse : ignorée
    let empreinte = empreinte_presence("AAA-AAA-AAA");
    let mut signe = signer(&alex.cle, Genre::Presence, &empreinte, 6, b"x");
    signe.instant = 7;
    alex.envoyer(&mut a, TrameAmi::Publier { empreinte, signe }, t0);
    assert!(steve.recues().is_empty());
    // La liaison expire après LIAISON_MAX sans nouvelles
    a.entretenir(t0 + LIAISON_MAX);
    assert_eq!(genres(&mut steve), ["absent"]);
    imposteur.publier(&mut a, &imposteur.cle, 1, t0 + LIAISON_MAX);
    assert_eq!(genres(&mut steve), ["presence"]);
}

#[test]
fn presence_muette_oubliee() {
    let (mut a, t0) = (Amis::default(), Instant::now());
    let alex = Client::nouveau(1, "AAA-AAA-AAA");
    let mut steve = Client::nouveau(2, "BBB-BBB-BBB");
    steve.ecouter(&mut a, &["AAA-AAA-AAA"], t0);
    alex.publier(&mut a, &alex.cle, 1, t0);
    steve.recues();
    a.entretenir(t0 + PRESENCE_MAX / 2);
    assert!(steve.recues().is_empty());
    a.entretenir(t0 + PRESENCE_MAX);
    assert_eq!(genres(&mut steve), ["absent"]);
}

#[test]
fn lettres_et_limite() {
    let (mut a, t0) = (Amis::default(), Instant::now());
    let mut alex = Client::nouveau(1, "AAA-AAA-AAA");
    let steve = Client::nouveau(2, "BBB-BBB-BBB");
    alex.ecouter(&mut a, &[], t0);
    steve.poster(&mut a, "AAA-AAA-AAA", t0);
    assert_eq!(genres(&mut alex), ["courrier"]);
    // Personne n'écoute cette boîte : rien n'est gardé
    steve.poster(&mut a, "CCC-CCC-CCC", t0);
    assert!(alex.recues().is_empty());
    // Une lettre signée pour une autre boîte est refusée
    let boite = empreinte_boite("AAA-AAA-AAA");
    let signe = signer(&steve.cle, Genre::Courrier, &empreinte_boite("X"), 1, b"x");
    steve.envoyer(&mut a, TrameAmi::Poster { boite, signe }, t0);
    assert!(alex.recues().is_empty());
    // Limite d'envois par minute (les 3 essais ci-dessus comptent)
    for _ in 0..ENVOIS_MAX {
        steve.poster(&mut a, "AAA-AAA-AAA", t0);
    }
    assert_eq!(alex.recues().len(), ENVOIS_MAX - 3);
    steve.poster(&mut a, "AAA-AAA-AAA", t0 + FENETRE_ENVOIS);
    assert_eq!(alex.recues().len(), 1);
    // Un nouvel abonnement remplace l'ancien
    alex.code = "DDD-DDD-DDD";
    alex.ecouter(&mut a, &[], t0);
    steve.poster(&mut a, "AAA-AAA-AAA", t0 + FENETRE_ENVOIS);
    assert!(alex.recues().is_empty());
}
