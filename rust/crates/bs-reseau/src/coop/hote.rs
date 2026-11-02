//! Côté hôte (src/reseau/hote.ts) : ouvre le salon, tient la liste des joueurs, lance la
//! partie, puis simule tout le monde. Avant chaque pas, il prend les entrées et les cartes
//! des invités ; après, il envoie les coffres ouverts, la fin, les bilans et l'instantané
//! commun (15 par seconde). Un invité coupé garde sa place `ABSENCE_MAX` secondes (il
//! reste immobile), puis quitte la partie pour de bon.

use bs_protocole::ANNONCE_MAX;
use bs_protocole::coop::{
    CHIFFRES_MAX, CarteJouee, Chiffre, Entete, INSTANTANES_PAR_SECONDE, InfoJoueur, MessageFiable,
    Presentation, StatsReseau, TYPE_ENTREE, ecrire_instantane_partie, lire_entree,
};
use bs_regles::Partie;
use bs_regles::etat::Issue;
use bs_regles::evenements::Evenement;

use super::{
    ABSENCE_MAX, ATTENTE_PRETS, ECART_STATS, PLACES, PhaseCoop, SILENCE_MAX, ajouter_joueur,
    fiable, par_place,
};
use crate::{EvenementSession, Session};

pub struct HoteCoop {
    pub session: Session,
    pub phase: PhaseCoop,
    /// Joueurs du salon (0 : l'hôte) ; au lancement, figés dans l'ordre de la partie.
    pub joueurs: Vec<InfoJoueur>,
    public: bool,
    prets: [bool; PLACES],
    attente: f32,
    /// Par place : coupé depuis (s).
    absences: [Option<f32>; PLACES],
    /// Par joueur de la partie : dernière entrée prise en compte, silence (s), place.
    entrees: Vec<u32>,
    silences: Vec<f32>,
    rangs: Vec<u8>,
    numero: u32,
    horloge: f32,
    horloge_stats: f32,
    chiffres: Vec<Chiffre>,
    tampon: Vec<u8>,
    /// Choix de coffre (restants, ender) avant le pas.
    coffres: (u32, u32),
    fin_envoyee: bool,
}

impl HoteCoop {
    /// Se connecte au relais ; le salon s'ouvre dès la connexion.
    pub fn nouveau(adresse: &str, moi: Presentation, public: bool) -> Self {
        Self {
            session: Session::ouvrir(adresse),
            phase: PhaseCoop::Connexion,
            joueurs: vec![InfoJoueur {
                index: 0,
                presentation: moi,
            }],
            public,
            prets: [false; PLACES],
            attente: 0.0,
            absences: [None; PLACES],
            entrees: Vec::with_capacity(PLACES),
            silences: Vec::with_capacity(PLACES),
            rangs: Vec::with_capacity(PLACES),
            numero: 0,
            horloge: 0.0,
            horloge_stats: 0.0,
            chiffres: Vec::with_capacity(CHIFFRES_MAX),
            tampon: Vec::with_capacity(16 * 1024),
            coffres: (0, 0),
            fin_envoyee: false,
        }
    }

    /// Code du salon (« ABC-DEF »), une fois ouvert.
    pub fn code(&self) -> Option<String> {
        self.session.salon.map(|(c, _)| crate::code_lisible(&c))
    }

    /// À chaque image hors partie (salon, chargement).
    pub fn maj_salon(&mut self) {
        self.traiter(None);
    }

    /// Fige la liste et demande à chacun de préparer l'arène `arene`.
    pub fn lancer(&mut self, arene: &str) {
        self.phase = PhaseCoop::Chargement {
            arene: arene.to_owned(),
        };
        self.prets = [false; PLACES];
        self.attente = 0.0;
        self.envoyer(&MessageFiable::Lancer {
            arene: arene.to_owned(),
            joueurs: self.joueurs.clone(),
        });
    }

    /// Pendant le chargement : vrai quand tous les invités sont prêts (ou trop tard).
    pub fn tous_prets(&mut self, dt: f32) -> bool {
        self.traiter(None);
        self.attente += dt;
        self.attente >= ATTENTE_PRETS
            || self.joueurs[1..]
                .iter()
                .all(|j| self.prets[j.index as usize] || self.absences[j.index as usize].is_some())
    }

    /// Ajoute les invités à la partie de l'hôte (déjà créée, à ses talents) et donne le départ.
    pub fn demarrer(&mut self, p: &mut Partie) {
        self.rangs.clear();
        self.entrees.clear();
        self.silences.clear();
        self.rangs.push(0);
        for j in &self.joueurs[1..] {
            ajouter_joueur(p, &j.presentation);
            self.rangs.push(j.index);
        }
        self.entrees.resize(self.rangs.len(), 0);
        self.silences.resize(self.rangs.len(), 0.0);
        self.coffres = (p.etat.choix_coffre.restants, p.etat.choix_coffre.ender);
        self.phase = PhaseCoop::Partie;
        self.envoyer(&MessageFiable::Depart);
    }

    /// Avant le pas : messages des invités (entrées, cartes), absences, silences.
    pub fn avant_pas(&mut self, p: &mut Partie, dt: f32) {
        self.traiter(Some(p));
        for k in 1..self.rangs.len() {
            let place = self.rangs[k] as usize;
            if let Some(t) = &mut self.absences[place] {
                *t += dt;
                if *t >= ABSENCE_MAX && !p.equipe.membres[k - 1].parti {
                    p.retirer_combattant(k);
                }
                continue;
            }
            self.silences[k] += dt;
            if self.silences[k] > SILENCE_MAX {
                p.equipe.membres[k - 1].joueur.commandes = Default::default();
            }
        }
        self.coffres = (p.etat.choix_coffre.restants, p.etat.choix_coffre.ender);
    }

    /// Après le pas : coffres, fin, bilans et instantané.
    pub fn apres_pas(&mut self, p: &Partie, dt: f32) {
        for e in &p.ev.liste {
            if let Evenement::Degats {
                x,
                y,
                z,
                valeur,
                style,
            } = *e
                && self.chiffres.len() < CHIFFRES_MAX
            {
                self.chiffres.push(Chiffre {
                    x,
                    y,
                    z,
                    valeur,
                    style,
                });
            }
        }
        let choix = &p.etat.choix_coffre;
        let nombre = choix.restants.saturating_sub(self.coffres.0);
        if nombre > 0 {
            let ender = choix.ender.saturating_sub(self.coffres.1);
            self.envoyer(&MessageFiable::Coffre { nombre, ender });
        }
        match p.etat.issue {
            Some(issue) if !self.fin_envoyee => {
                self.fin_envoyee = true;
                self.envoyer_fin(p, issue);
            }
            _ => {}
        }
        self.horloge_stats += dt;
        if self.horloge_stats >= ECART_STATS {
            self.horloge_stats = 0.0;
            for k in 1..self.rangs.len() {
                let stats = StatsReseau::from(&p.equipe.membres[k - 1].stats);
                self.envoyer(&MessageFiable::Stats {
                    place: self.rangs[k],
                    stats,
                });
            }
        }
        self.horloge += dt;
        let periode = 1.0 / INSTANTANES_PAR_SECONDE;
        if self.horloge >= periode {
            self.horloge = (self.horloge - periode).min(periode);
            self.numero += 1;
            let entete = Entete {
                numero: self.numero,
                rangs: &self.rangs,
                entrees: &self.entrees,
            };
            ecrire_instantane_partie(p, &entete, &self.chiffres, &mut self.tampon);
            self.session.vers_invites(&self.tampon);
            self.chiffres.clear();
        }
    }

    /// L'hôte a choisi une carte (déjà jouée chez lui) : les invités la jouent aussi.
    pub fn carte_locale(&mut self, carte: CarteJouee) {
        self.envoyer(&MessageFiable::CarteJouee { index: 0, carte });
    }

    /// « Continuer sans fin » après la victoire, pour tout le monde.
    pub fn continuer(&mut self, p: &mut Partie) {
        p.continuer_sans_fin();
        self.fin_envoyee = false;
        self.envoyer(&MessageFiable::Reprise);
    }

    pub fn quitter(&mut self) {
        self.session.quitter();
        self.phase = PhaseCoop::Finie("Salon fermé".into());
    }

    fn envoyer(&mut self, m: &MessageFiable) {
        self.session.vers_invites(&m.json());
    }

    fn envoyer_salon(&mut self) {
        self.envoyer(&MessageFiable::Salon {
            arene: String::new(),
            joueurs: self.joueurs.clone(),
        });
    }

    fn envoyer_fin(&mut self, p: &Partie, issue: Issue) {
        for k in 1..self.rangs.len() {
            let stats = StatsReseau::from(&p.equipe.membres[k - 1].stats);
            self.envoyer(&MessageFiable::Fin {
                place: self.rangs[k],
                issue: issue.into(),
                temps: p.etat.temps_jeu,
                stats,
            });
        }
    }

    /// Relève la session ; `p` : la partie, une fois commencée.
    fn traiter(&mut self, mut p: Option<&mut Partie>) {
        while let Some(ev) = self.session.maj() {
            match ev {
                EvenementSession::Connectee if self.session.salon.is_none() => {
                    let pseudo = self.joueurs[0].presentation.pseudo.as_bytes();
                    let annonce = &pseudo[..pseudo.len().min(ANNONCE_MAX)];
                    self.session.creer(self.public, annonce);
                }
                EvenementSession::Salon { .. } => {
                    if self.phase == PhaseCoop::Connexion {
                        self.phase = PhaseCoop::Salon;
                    }
                }
                EvenementSession::Message { place, octets } => {
                    self.message(place, &octets, p.as_deref_mut());
                }
                EvenementSession::Arrivee(place) => {
                    if let Some(a) = self.absences.get_mut(place as usize) {
                        *a = None;
                    }
                }
                EvenementSession::Depart(place) => self.depart(place),
                EvenementSession::Refus(_) if self.session.salon.is_none() => {
                    self.phase = PhaseCoop::Finie("Le relais refuse d'ouvrir le salon".into());
                }
                EvenementSession::Ferme | EvenementSession::Exclu => {
                    self.phase = PhaseCoop::Finie("Salon fermé".into());
                }
                EvenementSession::Perdue(raison) => self.phase = PhaseCoop::Finie(raison),
                _ => {}
            }
        }
    }

    fn depart(&mut self, place: u8) {
        if self.phase == PhaseCoop::Salon {
            self.joueurs.retain(|j| j.index != place);
            self.envoyer_salon();
        } else if let Some(a) = self.absences.get_mut(place as usize)
            && par_place(&self.joueurs, place).is_some()
        {
            a.get_or_insert(0.0);
        }
    }

    fn message(&mut self, place: u8, octets: &[u8], p: Option<&mut Partie>) {
        let k = par_place(&self.joueurs, place);
        if octets.first() == Some(&TYPE_ENTREE) {
            if let (Some(p), Some(k), Some((numero, c))) = (p, k, lire_entree(octets))
                && k > 0
                && k < self.entrees.len()
                && numero > self.entrees[k]
            {
                self.entrees[k] = numero;
                self.silences[k] = 0.0;
                p.equipe.membres[k - 1].joueur.commandes = c;
            }
            return;
        }
        let Some(m) = fiable(octets) else {
            return;
        };
        match m {
            MessageFiable::Bonjour(presentation) => self.bonjour(place, presentation),
            MessageFiable::Pret => {
                if let Some(pret) = self.prets.get_mut(place as usize) {
                    *pret = true;
                }
            }
            MessageFiable::Carte { carte } => {
                let (Some(p), Some(k)) = (p, k) else {
                    return;
                };
                let Some(c) = p.equipe.membres.get_mut(k.wrapping_sub(1)) else {
                    return;
                };
                if c.cartes_en_attente == 0 || c.parti {
                    return;
                }
                c.cartes_en_attente -= 1;
                c.temps_choix = 0.0;
                if p.avec_combattant(k, |p| carte.jouer(p)) {
                    self.envoyer(&MessageFiable::CarteJouee {
                        index: place,
                        carte,
                    });
                }
            }
            MessageFiable::Continuer => {
                if let Some(p) = p
                    && p.etat.issue == Some(Issue::Victoire)
                {
                    self.continuer(p);
                }
            }
            _ => {}
        }
    }

    fn bonjour(&mut self, place: u8, presentation: Presentation) {
        let connu = par_place(&self.joueurs, place).is_some();
        if self.phase != PhaseCoop::Salon {
            // Revenu après une coupure : déjà dans la partie ; sinon trop tard
            if !connu {
                self.refuser(place, "La partie a déjà commencé");
            }
            return;
        }
        if presentation.version != self.joueurs[0].presentation.version {
            let raison = format!(
                "Version différente de celle de l'hôte ({})",
                self.joueurs[0].presentation.version
            );
            self.refuser(place, &raison);
            return;
        }
        let info = InfoJoueur {
            index: place,
            presentation,
        };
        match self.joueurs.iter_mut().find(|j| j.index == place) {
            Some(j) => *j = info,
            None => self.joueurs.push(info),
        }
        self.joueurs.sort_by_key(|j| j.index);
        self.envoyer_salon();
    }

    fn refuser(&mut self, place: u8, raison: &str) {
        self.envoyer(&MessageFiable::Refus {
            place,
            raison: raison.to_owned(),
        });
        self.session.exclure(place);
    }
}
