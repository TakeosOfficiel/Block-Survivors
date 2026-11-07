//! Côté invité (src/reseau/client.ts) : rejoint un salon par son code, se présente, prépare
//! sa partie au lancement, puis à chaque pas envoie son entrée, prédit sa marche
//! (`simuler_invite`) et applique le dernier instantané reçu de l'hôte. Ses cartes partent
//! chez l'hôte, qui les joue pour de vrai ; celles des autres arrivent en `CarteJouee`.

use bs_donnees::Donnees;
use bs_protocole::coop::{
    CarteJouee, InfoJoueur, IssueReseau, MessageFiable, Presentation, Recepteur, StatsReseau,
    TYPE_INSTANTANE, ecrire_entree,
};
use bs_protocole::{REFUS_PLEIN, REFUS_SALON, REFUS_VERSION};
use bs_regles::Partie;
use bs_regles::joueur::Commandes;

use super::{PhaseCoop, ajouter_joueur, fiable, index_arene};
use crate::{EvenementSession, Session};

pub struct InviteCoop {
    pub session: Session,
    pub phase: PhaseCoop,
    /// Ma place dans le salon.
    pub place: u8,
    /// Joueurs du salon (0 : l'hôte) ; au lancement, figés dans l'ordre de la partie.
    pub joueurs: Vec<InfoJoueur>,
    moi: Presentation,
    code: [u8; 6],
    pub recepteur: Option<Recepteur>,
    numero: u32,
    tampon: Vec<u8>,
    /// Dernier instantané reçu, pas encore appliqué.
    instantane: Vec<u8>,
    nouveau: bool,
    /// Mon bilan, envoyé par l'hôte régulièrement.
    pub stats: Option<StatsReseau>,
    /// Fin de partie : issue, temps de jeu (s), bilan.
    pub fin: Option<(IssueReseau, f64, StatsReseau)>,
    /// L'hôte est coupé (il a 60 s pour revenir).
    pub hote_absent: bool,
}

impl InviteCoop {
    /// Se connecte au relais et rejoint le salon `code`.
    pub fn nouveau(adresse: &str, moi: Presentation, code: [u8; 6]) -> Self {
        Self {
            session: Session::ouvrir(adresse),
            phase: PhaseCoop::Connexion,
            place: 0,
            joueurs: Vec::new(),
            moi,
            code,
            recepteur: None,
            numero: 0,
            tampon: Vec::with_capacity(64),
            instantane: Vec::with_capacity(16 * 1024),
            nouveau: false,
            stats: None,
            fin: None,
            hote_absent: false,
        }
    }

    /// À chaque image hors partie (salon, chargement).
    pub fn maj_salon(&mut self) {
        self.traiter(None);
    }

    /// Index de l'arène générée lancée par l'hôte (pendant le chargement).
    pub fn arene(&self, d: &Donnees) -> Option<usize> {
        match &self.phase {
            PhaseCoop::Chargement { arene } => index_arene(d, arene),
            _ => None,
        }
    }

    /// Au lancement : `p` est ma partie (arène lancée, mon personnage, mes talents) ; elle
    /// devient une partie d'invité, avec les autres joueurs dans l'ordre de la liste.
    pub fn preparer(&mut self, p: &mut Partie) {
        p.invite = true;
        let mut table = vec![(self.place, 0)];
        for j in self.joueurs.iter().filter(|j| j.index != self.place) {
            let k = ajouter_joueur(p, &j.presentation);
            table.push((j.index, k));
        }
        self.recepteur = Some(Recepteur::nouveau(&table));
    }

    /// Ma partie est prête (textures chargées) : j'attends le départ.
    pub fn pret(&mut self) {
        self.envoyer(&MessageFiable::Pret);
    }

    /// Un pas de partie : messages, dernier instantané, entrée, prédiction, lissage.
    /// `sans_commandes` : menu ouvert ou cartes à choisir, le joueur ne bouge pas.
    pub fn pas(&mut self, p: &mut Partie, dt: f32, sans_commandes: bool) {
        self.traiter(Some(p));
        let Some(r) = &mut self.recepteur else {
            return;
        };
        if std::mem::take(&mut self.nouveau) {
            r.appliquer(p, &self.instantane);
        }
        if sans_commandes {
            p.joueur.commandes = Commandes {
                lacet: p.joueur.commandes.lacet,
                ..Commandes::default()
            };
        }
        self.numero += 1;
        ecrire_entree(self.numero, &p.joueur.commandes, &mut self.tampon);
        self.session.vers_hote(&self.tampon);
        p.simuler_invite(dt);
        let j = &p.etat.joueur;
        r.noter_entree(self.numero, j.x, j.z);
        r.interpoler(p, dt);
    }

    /// J'ai choisi une carte (déjà jouée chez moi) : l'hôte la joue pour de vrai.
    pub fn carte_locale(&mut self, carte: CarteJouee) {
        self.envoyer(&MessageFiable::Carte { carte });
    }

    /// Demande à l'hôte de « continuer sans fin » après la victoire.
    pub fn continuer(&mut self) {
        self.envoyer(&MessageFiable::Continuer);
    }

    pub fn quitter(&mut self) {
        self.session.quitter();
        self.phase = PhaseCoop::Finie("Salon quitté".into());
    }

    fn envoyer(&mut self, m: &MessageFiable) {
        self.session.vers_hote(&m.json());
    }

    fn finir(&mut self, raison: &str) {
        if !matches!(self.phase, PhaseCoop::Finie(_)) {
            self.phase = PhaseCoop::Finie(raison.to_owned());
        }
    }

    fn traiter(&mut self, mut p: Option<&mut Partie>) {
        while let Some(ev) = self.session.maj() {
            match ev {
                EvenementSession::Connectee if self.session.salon.is_none() => {
                    self.session.rejoindre(self.code);
                }
                EvenementSession::Salon { place, .. } => {
                    self.place = place;
                    self.phase = PhaseCoop::Salon;
                    let bonjour = MessageFiable::Bonjour(self.moi.clone());
                    self.envoyer(&bonjour);
                }
                EvenementSession::Refus(code) if self.session.salon.is_none() => {
                    self.finir(match code {
                        REFUS_SALON => "Aucun salon avec ce code",
                        REFUS_PLEIN => "Le salon est plein",
                        REFUS_VERSION => "Version du jeu différente du relais",
                        _ => "Le relais refuse l'entrée",
                    });
                }
                EvenementSession::Message { octets, .. } => {
                    if octets.first() == Some(&TYPE_INSTANTANE) {
                        self.instantane.clear();
                        self.instantane.extend_from_slice(&octets);
                        self.nouveau = true;
                    } else if let Some(m) = fiable(&octets) {
                        self.message(m, p.as_deref_mut());
                    }
                }
                EvenementSession::HotePerdu => self.hote_absent = true,
                EvenementSession::HoteRevenu => self.hote_absent = false,
                EvenementSession::Exclu => self.finir("Exclu du salon par l'hôte"),
                EvenementSession::Ferme => self.finir("L'hôte a fermé le salon"),
                EvenementSession::Perdue(raison) => self.finir(&raison),
                _ => {}
            }
        }
    }

    fn message(&mut self, m: MessageFiable, p: Option<&mut Partie>) {
        match m {
            MessageFiable::Salon { joueurs, .. } if self.phase == PhaseCoop::Salon => {
                self.joueurs = joueurs;
            }
            MessageFiable::Lancer { arene, joueurs } if self.phase == PhaseCoop::Salon => {
                self.joueurs = joueurs;
                self.phase = PhaseCoop::Chargement { arene };
            }
            MessageFiable::Depart if self.recepteur.is_some() => self.phase = PhaseCoop::Partie,
            MessageFiable::Refus { place, raison } if place == self.place => self.finir(&raison),
            MessageFiable::Stats { place, stats } if place == self.place => {
                if let Some(p) = p {
                    stats.copier_dans(&mut p.etat.stats);
                }
                self.stats = Some(stats);
            }
            MessageFiable::Fin {
                place,
                issue,
                temps,
                stats,
            } if place == self.place => {
                if let Some(p) = p {
                    stats.copier_dans(&mut p.etat.stats);
                }
                self.fin = Some((issue, temps, stats));
            }
            MessageFiable::CarteJouee { index, carte } => {
                let k = self
                    .recepteur
                    .as_ref()
                    .and_then(|r| r.joueurs.get(index as usize).copied().flatten());
                if let (Some(p), Some(k)) = (p, k)
                    && k > 0
                {
                    p.avec_combattant(k, |p| carte.jouer(p));
                }
            }
            MessageFiable::Coffre { nombre, ender } => {
                if let Some(p) = p {
                    p.etat.choix_coffre.restants += nombre;
                    p.etat.choix_coffre.ender += ender;
                }
            }
            MessageFiable::Reprise => {
                if let Some(p) = p {
                    p.continuer_sans_fin();
                }
                self.fin = None;
            }
            _ => {}
        }
    }
}
