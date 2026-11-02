//! Tests de la logique des salons, sans réseau : chaque client est un canal, l'heure est
//! avancée à la main.

use std::time::Duration;

use bs_protocole::{
    EntreeListe, REFUS_JETON, REFUS_PLEIN, REFUS_SALON, REFUS_SALONS, REFUS_TENTATIVES,
    REFUS_VERSION, VERSION_PROTOCOLE,
};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use super::*;

const V: u16 = VERSION_PROTOCOLE;
const IP: IpAddr = IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1));

struct Client {
    id: u64,
    envoi: Envoi,
    recu: UnboundedReceiver<Sortant>,
    siege: Option<Siege>,
    jeton: u64,
}

impl Client {
    fn nouveau(id: u64) -> Self {
        let (envoi, recu) = unbounded_channel();
        Self {
            id,
            envoi,
            recu,
            siege: None,
            jeton: 0,
        }
    }

    /// Envoie une trame d'accueil ; retient la place et le jeton si admis.
    fn accueil(&mut self, s: &mut Salons, t: Trame, ip: IpAddr, quand: Instant) -> Option<Siege> {
        self.siege = s.accueillir(t, ip, self.id, &self.envoi, quand);
        if self.siege.is_some() {
            let o = self.octets();
            let Some(Trame::Salon { jeton, .. }) = lire_trame(&o) else {
                panic!("attendu Salon : {o:?}")
            };
            self.jeton = jeton;
        }
        self.siege
    }

    fn envoyer(&self, s: &mut Salons, t: Trame, quand: Instant) {
        let mut o = Vec::new();
        t.ecrire(&mut o);
        s.recevoir(self.siege.unwrap(), self.id, o.into(), quand);
    }

    fn couper(&self, s: &mut Salons, volontaire: bool, quand: Instant) {
        s.deconnecter(self.siege.unwrap(), self.id, volontaire, quand);
    }

    fn octets(&mut self) -> Bytes {
        match self.recu.try_recv() {
            Ok(Sortant::Octets(o)) => o,
            autre => panic!("attendu une trame : {autre:?}"),
        }
    }

    fn attendre(&mut self, t: Trame) {
        let o = self.octets();
        assert_eq!(lire_trame(&o), Some(t));
    }

    fn ferme(&mut self) {
        assert_eq!(self.recu.try_recv(), Ok(Sortant::Fermer));
    }

    fn rien(&mut self) {
        assert!(self.recu.try_recv().is_err(), "message inattendu");
    }
}

fn creer(public: bool) -> Trame<'static> {
    Trame::Creer {
        version: V,
        public,
        annonce: b"plaine",
    }
}

/// Un salon avec un hôte (id 0) et `n` invités (ids 1 à n), files vidées.
fn salon(s: &mut Salons, n: u64, t: Instant) -> (Client, Vec<Client>, [u8; 6]) {
    let mut hote = Client::nouveau(0);
    let code = hote.accueil(s, creer(true), IP, t).unwrap().code;
    let mut invites = Vec::new();
    for id in 1..=n {
        let mut c = Client::nouveau(id);
        let siege = c.accueil(s, Trame::Rejoindre { version: V, code }, IP, t);
        assert_eq!(siege.map(|s| s.place), Some(id as u8));
        hote.attendre(Trame::Arrivee { place: id as u8 });
        invites.push(c);
    }
    (hote, invites, code)
}

fn reglages() -> Reglages {
    Reglages {
        versions: 1..=VERSION_PROTOCOLE + 1,
        ..Reglages::default()
    }
}

#[test]
fn creer_rejoindre_et_echanger() {
    let (mut s, t) = (Salons::nouveaux(reglages()), Instant::now());
    let (mut hote, mut inv, code) = salon(&mut s, 2, t);
    assert!(
        code.iter()
            .all(|l| l.is_ascii_uppercase() && *l != b'I' && *l != b'O')
    );
    assert_ne!(hote.jeton, inv[0].jeton);
    // Invité → hôte : place réécrite
    inv[1].envoyer(
        &mut s,
        Trame::VersHote {
            place: 4,
            charge: b"bonjour",
        },
        t,
    );
    hote.attendre(Trame::VersHote {
        place: 2,
        charge: b"bonjour",
    });
    // Hôte → tous les invités ; un invité ne peut pas parler aux autres
    hote.envoyer(&mut s, Trame::VersInvites { charge: &[1, 2] }, t);
    inv[0].envoyer(&mut s, Trame::VersInvites { charge: &[9] }, t);
    for c in &mut inv {
        c.attendre(Trame::VersInvites { charge: &[1, 2] });
        c.rien();
    }
    let e = s.etat();
    assert_eq!((e.salons, e.publics, e.joueurs, e.reservees), (1, 1, 3, 0));
}

#[test]
fn plein_inconnu_et_version() {
    let (mut s, t) = (Salons::nouveaux(reglages()), Instant::now());
    let (_hote, _inv, code) = salon(&mut s, 4, t);
    let mut c = Client::nouveau(9);
    let mut refus = |s: &mut Salons, tr: Trame, raison: u8| {
        assert_eq!(c.accueil(s, tr, IP, t), None);
        c.attendre(Trame::Refus { raison });
    };
    refus(&mut s, Trame::Rejoindre { version: V, code }, REFUS_PLEIN);
    refus(
        &mut s,
        Trame::Rejoindre {
            version: V,
            code: *b"ZZZZZZ",
        },
        REFUS_SALON,
    );
    // Version acceptée par le relais mais pas celle du salon
    refus(
        &mut s,
        Trame::Rejoindre {
            version: V + 1,
            code,
        },
        REFUS_VERSION,
    );
    // Version inconnue du relais
    refus(
        &mut s,
        Trame::Creer {
            version: V + 2,
            public: false,
            annonce: &[],
        },
        REFUS_VERSION,
    );
}

#[test]
fn invite_coupe_puis_reprend() {
    let (mut s, t) = (Salons::nouveaux(reglages()), Instant::now());
    let (mut hote, inv, code) = salon(&mut s, 2, t);
    inv[0].couper(&mut s, false, t);
    hote.attendre(Trame::Depart { place: 1 });
    assert_eq!(s.etat().reservees, 1);
    // La place réservée n'est pas donnée à un nouveau venu
    let mut nouveau = Client::nouveau(3);
    let siege = nouveau.accueil(&mut s, Trame::Rejoindre { version: V, code }, IP, t);
    assert_eq!(siege.map(|s| s.place), Some(3));
    hote.attendre(Trame::Arrivee { place: 3 });
    // Mauvais jeton, puis le bon
    let mut retour = Client::nouveau(10);
    let jeton = inv[0].jeton;
    let mauvais = Trame::Reprendre {
        version: V,
        code,
        jeton: jeton ^ 1,
    };
    assert_eq!(retour.accueil(&mut s, mauvais, IP, t), None);
    retour.attendre(Trame::Refus {
        raison: REFUS_JETON,
    });
    let bon = Trame::Reprendre {
        version: V,
        code,
        jeton,
    };
    let t2 = t + Duration::from_secs(59);
    assert_eq!(
        retour.accueil(&mut s, bon, IP, t2).map(|s| s.place),
        Some(1)
    );
    assert_eq!(retour.jeton, jeton);
    hote.attendre(Trame::Arrivee { place: 1 });
    // L'ancienne connexion n'agit plus
    inv[0].envoyer(
        &mut s,
        Trame::VersHote {
            place: 1,
            charge: &[],
        },
        t2,
    );
    hote.rien();
}

#[test]
fn hote_coupe_puis_reprend() {
    let (mut s, t) = (Salons::nouveaux(reglages()), Instant::now());
    let (hote, mut inv, code) = salon(&mut s, 2, t);
    hote.couper(&mut s, false, t);
    for c in &mut inv {
        c.attendre(Trame::HotePerdu);
    }
    // Personne n'entre pendant l'absence de l'hôte, et la liste ne le montre pas
    let mut c = Client::nouveau(5);
    assert_eq!(
        c.accueil(&mut s, Trame::Rejoindre { version: V, code }, IP, t),
        None
    );
    c.attendre(Trame::Refus {
        raison: REFUS_SALON,
    });
    c.accueil(&mut s, Trame::Lister { version: V }, IP, t);
    let o = c.octets();
    assert!(matches!(lire_trame(&o), Some(Trame::Liste { salons }) if salons.est_vide()));
    // Un invité qui revient pendant l'absence de l'hôte l'apprend
    inv[1].couper(&mut s, false, t);
    let jeton = inv[1].jeton;
    let mut retour = Client::nouveau(6);
    let reprise = Trame::Reprendre {
        version: V,
        code,
        jeton,
    };
    retour.accueil(&mut s, reprise, IP, t).unwrap();
    retour.attendre(Trame::HotePerdu);
    // Retour de l'hôte : les invités sont prévenus, l'hôte apprend qui est là
    let mut h2 = Client::nouveau(7);
    let jeton = hote.jeton;
    let reprise = Trame::Reprendre {
        version: V,
        code,
        jeton,
    };
    let t2 = t + Duration::from_secs(30);
    assert_eq!(
        h2.accueil(&mut s, reprise, IP, t2).map(|s| s.place),
        Some(0)
    );
    inv[0].attendre(Trame::HoteRevenu);
    retour.attendre(Trame::HoteRevenu);
    h2.attendre(Trame::Arrivee { place: 1 });
    h2.attendre(Trame::Arrivee { place: 2 });
    h2.rien();
}

#[test]
fn delais_expires() {
    let r = Reglages {
        delai_reconnexion: Duration::from_secs(5),
        ..reglages()
    };
    let (mut s, t) = (Salons::nouveaux(r), Instant::now());
    let (mut hote, mut inv, code) = salon(&mut s, 2, t);
    // Invité coupé trop longtemps : place libérée, jeton invalide
    inv[0].couper(&mut s, false, t);
    hote.attendre(Trame::Depart { place: 1 });
    s.entretenir(t + Duration::from_secs(4));
    assert_eq!(s.etat().reservees, 1);
    s.entretenir(t + Duration::from_secs(5));
    assert_eq!(s.etat().reservees, 0);
    let mut c = Client::nouveau(8);
    let jeton = inv[0].jeton;
    let reprise = Trame::Reprendre {
        version: V,
        code,
        jeton,
    };
    let t6 = t + Duration::from_secs(6);
    assert_eq!(c.accueil(&mut s, reprise, IP, t6), None);
    c.attendre(Trame::Refus {
        raison: REFUS_JETON,
    });
    // Hôte coupé trop longtemps : salon fermé
    hote.couper(&mut s, false, t6);
    inv[1].attendre(Trame::HotePerdu);
    s.entretenir(t + Duration::from_secs(11));
    inv[1].attendre(Trame::Ferme);
    inv[1].ferme();
    assert_eq!(s.etat(), Etat::default());
    // Plus de salon, plus de compte pour l'adresse : on peut en rouvrir 3
    for k in 0..3 {
        assert!(
            Client::nouveau(20 + k)
                .accueil(&mut s, creer(false), IP, t6)
                .is_some()
        );
    }
}

#[test]
fn salon_inactif_ferme() {
    let (mut s, t) = (Salons::nouveaux(reglages()), Instant::now());
    let (mut hote, mut inv, _) = salon(&mut s, 1, t);
    let presque = t + INACTIVITE - Duration::from_secs(1);
    hote.envoyer(&mut s, Trame::VersInvites { charge: &[] }, presque);
    inv[0].attendre(Trame::VersInvites { charge: &[] });
    s.entretenir(t + INACTIVITE);
    assert_eq!(s.etat().salons, 1);
    s.entretenir(presque + INACTIVITE);
    assert_eq!(s.etat().salons, 0);
    for c in [&mut hote, &mut inv[0]] {
        c.attendre(Trame::Ferme);
        c.ferme();
    }
}
const INACTIVITE: Duration = crate::limites::INACTIVITE_SALON;

#[test]
fn departs_volontaires() {
    let (mut s, t) = (Salons::nouveaux(reglages()), Instant::now());
    let (mut hote, mut inv, _) = salon(&mut s, 2, t);
    inv[0].couper(&mut s, true, t);
    hote.attendre(Trame::Depart { place: 1 });
    assert_eq!(s.etat().reservees, 0);
    hote.couper(&mut s, true, t);
    inv[1].attendre(Trame::Ferme);
    inv[1].ferme();
    assert_eq!(s.etat().salons, 0);
}

#[test]
fn exclure() {
    let (mut s, t) = (Salons::nouveaux(reglages()), Instant::now());
    let (mut hote, mut inv, code) = salon(&mut s, 2, t);
    // Un invité ne peut pas exclure
    inv[1].envoyer(&mut s, Trame::Exclure { place: 1 }, t);
    inv[0].rien();
    hote.envoyer(&mut s, Trame::Exclure { place: 1 }, t);
    inv[0].attendre(Trame::Exclu);
    inv[0].ferme();
    hote.attendre(Trame::Depart { place: 1 });
    // La fin de sa connexion ne change plus rien ; son jeton n'est plus valide
    inv[0].couper(&mut s, false, t);
    hote.rien();
    let mut c = Client::nouveau(9);
    let jeton = inv[0].jeton;
    let reprise = Trame::Reprendre {
        version: V,
        code,
        jeton,
    };
    assert_eq!(c.accueil(&mut s, reprise, IP, t), None);
    c.attendre(Trame::Refus {
        raison: REFUS_JETON,
    });
    // Place libérée
    let siege = c.accueil(&mut s, Trame::Rejoindre { version: V, code }, IP, t);
    assert_eq!(siege.map(|s| s.place), Some(1));
}

#[test]
fn reprise_sur_connexion_encore_ouverte() {
    let (mut s, t) = (Salons::nouveaux(reglages()), Instant::now());
    let (mut hote, mut inv, code) = salon(&mut s, 1, t);
    let mut c = Client::nouveau(5);
    let jeton = inv[0].jeton;
    let reprise = Trame::Reprendre {
        version: V,
        code,
        jeton,
    };
    c.accueil(&mut s, reprise, IP, t).unwrap();
    inv[0].ferme();
    hote.attendre(Trame::Arrivee { place: 1 });
    // La fin de l'ancienne connexion ne libère pas la place
    inv[0].couper(&mut s, false, t);
    hote.rien();
    assert_eq!(s.etat().joueurs, 2);
}

#[test]
fn liste_publique_par_version() {
    let (mut s, t) = (Salons::nouveaux(reglages()), Instant::now());
    let (hote, _inv, code) = salon(&mut s, 2, t);
    hote.envoyer(&mut s, Trame::Annoncer { annonce: b"foret" }, t);
    // Un salon privé et un salon d'une autre version
    let autre_ip: IpAddr = [10, 0, 0, 2].into();
    Client::nouveau(10).accueil(&mut s, creer(false), autre_ip, t);
    let autre_version = Trame::Creer {
        version: V + 1,
        public: true,
        annonce: &[],
    };
    Client::nouveau(11).accueil(&mut s, autre_version, autre_ip, t);
    let mut c = Client::nouveau(12);
    assert_eq!(c.accueil(&mut s, Trame::Lister { version: V }, IP, t), None);
    let o = c.octets();
    let Some(Trame::Liste { salons }) = lire_trame(&o) else {
        panic!()
    };
    let lus: Vec<_> = salons.iter().collect();
    assert_eq!(
        lus,
        [EntreeListe {
            code,
            joueurs: 3,
            places: PLACES_SALON,
            annonce: b"foret"
        }]
    );
    c.accueil(&mut s, Trame::Lister { version: V + 1 }, IP, t);
    let o = c.octets();
    let Some(Trame::Liste { salons }) = lire_trame(&o) else {
        panic!()
    };
    assert_eq!(salons.iter().count(), 1);
}

#[test]
fn limites_par_adresse() {
    let (mut s, t) = (Salons::nouveaux(reglages()), Instant::now());
    let mut hotes: Vec<_> = (0..3).map(Client::nouveau).collect();
    for h in &mut hotes {
        assert!(h.accueil(&mut s, creer(false), IP, t).is_some());
    }
    let mut c = Client::nouveau(9);
    assert_eq!(c.accueil(&mut s, creer(false), IP, t), None);
    c.attendre(Trame::Refus {
        raison: REFUS_SALONS,
    });
    // Un salon fermé rend une place à l'adresse
    hotes[0].couper(&mut s, true, t);
    hotes[0].attendre(Trame::Ferme);
    assert!(c.accueil(&mut s, creer(false), IP, t).is_some());
    // Tentatives : 20 par minute, même sur des codes faux
    let mut pirate = Client::nouveau(30);
    let essai = |k: u8| Trame::Rejoindre {
        version: V,
        code: [b'A' + k, b'B', b'C', b'D', b'E', b'F'],
    };
    for k in 0..20 {
        pirate.accueil(&mut s, essai(k), IP, t);
        pirate.attendre(Trame::Refus {
            raison: REFUS_SALON,
        });
    }
    pirate.accueil(&mut s, essai(0), IP, t);
    pirate.attendre(Trame::Refus {
        raison: REFUS_TENTATIVES,
    });
    s.entretenir(t + Duration::from_secs(60));
    pirate.accueil(&mut s, essai(0), IP, t + Duration::from_secs(60));
    pirate.attendre(Trame::Refus {
        raison: REFUS_SALON,
    });
}
