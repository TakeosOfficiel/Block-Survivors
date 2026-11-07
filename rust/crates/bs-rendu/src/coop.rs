//! La coop en jeu (src/reseau/), par-dessus `bs-reseau` : la ressource `Coop` tient
//! l'hôte ou l'invité, du salon jusqu'au retour au menu.
//! - Chargement : l'hôte attend que tous soient prêts ; l'invité dit qu'il l'est une fois
//!   ses textures chargées, puis attend le départ.
//! - Partie : rien ne se fige (ni pause, ni cartes) ; l'hôte simule l'équipe, l'invité
//!   prédit et affiche les instantanés. Les cartes choisies partent aux autres.
//! - Une coop finie (refus, coupure trop longue, salon fermé) ramène au menu, où
//!   `MessageCoop` explique pourquoi.

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_protocole::coop::{CarteJouee, InfoJoueur, Presentation};
use bs_regles::partie::Partie;
use bs_regles::textes::Choix;
use bs_reseau::{HoteCoop, InviteCoop, PhaseCoop};
use bs_sauvegarde::collection::PIECES;
use bs_sauvegarde::profil_joueur::ChoixSkin;

use crate::PartieEnCours;
use crate::ecran_cartes::{self, EcranCartes};
use crate::sauvegarde::Sauvegarde;

/// Skin importée plus lourde (data URL) : non envoyée, les autres voient celle de la classe.
const SKIN_MAX: usize = 48 * 1024;

pub enum Role {
    Hote(Box<HoteCoop>),
    Invite(Box<InviteCoop>),
}

#[derive(Resource)]
pub struct Coop {
    pub role: Role,
    /// Invité : `Pret` envoyé.
    pret: bool,
}

/// Pourquoi la dernière coop s'est arrêtée (affiché au menu).
#[derive(Resource, Default)]
pub struct MessageCoop(pub Option<String>);

impl Coop {
    pub fn hote(moi: Presentation, public: bool) -> Self {
        let adresse = bs_reseau::adresse_relais();
        Self {
            role: Role::Hote(Box::new(HoteCoop::nouveau(&adresse, moi, public))),
            pret: false,
        }
    }

    pub fn invite(moi: Presentation, code: [u8; 6]) -> Self {
        let adresse = bs_reseau::adresse_relais();
        Self {
            role: Role::Invite(Box::new(InviteCoop::nouveau(&adresse, moi, code))),
            pret: false,
        }
    }

    pub fn phase(&self) -> &PhaseCoop {
        match &self.role {
            Role::Hote(h) => &h.phase,
            Role::Invite(i) => &i.phase,
        }
    }

    pub fn joueurs(&self) -> &[InfoJoueur] {
        match &self.role {
            Role::Hote(h) => &h.joueurs,
            Role::Invite(i) => &i.joueurs,
        }
    }

    /// Ma place dans le salon (0 : l'hôte).
    pub fn place(&self) -> u8 {
        match &self.role {
            Role::Hote(_) => 0,
            Role::Invite(i) => i.place,
        }
    }

    pub fn est_hote(&self) -> bool {
        matches!(self.role, Role::Hote(_))
    }

    /// Code du salon (« ABC-DEF »).
    pub fn code(&self) -> Option<String> {
        let s = match &self.role {
            Role::Hote(h) => &h.session,
            Role::Invite(i) => &i.session,
        };
        s.salon.map(|(c, _)| bs_reseau::code_lisible(&c))
    }

    /// Relève la session hors partie (menu, salon).
    pub fn maj_salon(&mut self) {
        match &mut self.role {
            Role::Hote(h) => h.maj_salon(),
            Role::Invite(i) => i.maj_salon(),
        }
    }

    pub fn quitter(&mut self) {
        match &mut self.role {
            Role::Hote(h) => h.quitter(),
            Role::Invite(i) => i.quitter(),
        }
    }

    /// Coop finie : sa raison.
    pub fn finie(&self) -> Option<&str> {
        match self.phase() {
            PhaseCoop::Finie(r) => Some(r),
            _ => None,
        }
    }
}

/// Qui je suis pour les autres : profil, personnage choisi, talents, armure portée.
pub fn presentation(d: &Donnees, s: &Sauvegarde, personnage: usize) -> Presentation {
    let id = d
        .personnages
        .personnages
        .get_index(personnage)
        .map_or("", |(id, _)| id.as_str());
    let talents = s.progression.noeuds_actifs(id);
    let armures = PIECES
        .iter()
        .zip(s.collection.portees())
        .filter_map(|(&piece, monde)| monde.map(|m| (m, piece)))
        .collect();
    let skin_importe = (s.profil.skin == ChoixSkin::Importe)
        .then(|| s.profil.skin_importe.clone())
        .flatten()
        .filter(|png| png.len() <= SKIN_MAX);
    let skin = match (&s.profil.skin, &skin_importe) {
        (ChoixSkin::Importe, None) => ChoixSkin::Classe.id(),
        (c, _) => c.id(),
    };
    Presentation {
        version: bs_commun::VERSION.into(),
        pseudo: s.profil.pseudo.clone(),
        personnage: id.into(),
        skin: skin.into(),
        bras_fins: s.profil.bras_fins,
        skin_importe,
        arbre: 0,
        talents,
        armures,
    }
}

/// Chargement : `Ok(true)` quand la partie peut commencer ; `Err` si la coop a pris fin.
pub fn partie_prete(coop: &mut Coop, textures_chargees: bool, dt: f32) -> Result<bool, String> {
    let pret = &mut coop.pret;
    let prete = match &mut coop.role {
        Role::Hote(h) => textures_chargees && h.tous_prets(dt),
        Role::Invite(i) => {
            if textures_chargees && !std::mem::replace(pret, true) {
                i.pret();
            }
            i.maj_salon();
            i.phase == PhaseCoop::Partie
        }
    };
    match coop.finie() {
        Some(r) => Err(r.to_owned()),
        None => Ok(prete),
    }
}

/// Fin du chargement : l'hôte ajoute les invités à sa partie et donne le départ.
pub fn demarrer(coop: Option<ResMut<Coop>>, jeu: Option<ResMut<PartieEnCours>>) {
    if let (Some(mut coop), Some(mut jeu)) = (coop, jeu)
        && let Role::Hote(h) = &mut coop.role
        && matches!(h.phase, PhaseCoop::Chargement { .. })
    {
        h.demarrer(&mut jeu.partie);
    }
}

/// Un pas de partie à plusieurs ; `Err` si la coop a pris fin.
pub fn pas(coop: &mut Coop, jeu: &mut PartieEnCours, dt: f32) -> Result<(), String> {
    let p = &mut jeu.partie;
    match &mut coop.role {
        Role::Hote(h) => {
            p.equipe.choix_hote = ecran_cartes::en_attente(p);
            h.avant_pas(p, dt);
            if p.etat.issue.is_none() {
                p.pas(dt);
            }
            h.apres_pas(p, dt);
        }
        Role::Invite(i) => {
            i.pas(p, dt, false);
            let (x, z) = (p.etat.joueur.x, p.etat.joueur.z);
            p.vue.suivre(x, z, dt);
        }
    }
    if p.etat.issue.is_some() {
        jeu.fin += dt;
    }
    coop.finie().map_or(Ok(()), |r| Err(r.to_owned()))
}

/// Cartes choisies à cette image : les autres les jouent aussi.
pub fn envoyer_cartes(
    coop: Option<ResMut<Coop>>,
    mut ecran: ResMut<EcranCartes>,
    jeu: Res<PartieEnCours>,
) {
    let Some(mut coop) = coop else {
        return;
    };
    let d = jeu.partie.d;
    for choix in ecran.jouees.drain(..) {
        let carte = match choix {
            Choix::Niveau(a) => Some(CarteJouee::depuis_amelioration(d, a)),
            Choix::Coffre(c) => CarteJouee::depuis_equipement(d, c),
        };
        let Some(carte) = carte else { continue };
        match &mut coop.role {
            Role::Hote(h) => h.carte_locale(carte),
            Role::Invite(i) => i.carte_locale(carte),
        }
    }
}

/// « Continuer sans fin » : l'hôte reprend pour tous ; l'invité le demande (vrai si la
/// partie reprend tout de suite).
pub fn continuer(coop: &mut Coop, p: &mut Partie) -> bool {
    match &mut coop.role {
        Role::Hote(h) => {
            h.continuer(p);
            true
        }
        Role::Invite(i) => {
            i.continuer();
            false
        }
    }
}
