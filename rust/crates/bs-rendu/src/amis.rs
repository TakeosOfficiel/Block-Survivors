//! Amis en jeu (src/reseau/amis.ts) : la ressource `ReseauAmis` tient le service des amis
//! (bs-reseau) du profil actif, créée à la première entrée au menu et refaite au
//! changement de profil. À chaque image, dans toutes les phases :
//! - on publie où on en est : au menu, dans un salon (son code s'il reste de la place,
//!   pour qu'un ami le rejoigne d'un clic) ou en partie ;
//! - on retient dans la sauvegarde ce que le service apprend (clé d'un ami, son pseudo) ;
//! - on garde les invitations, demandes et ajouts reçus pour la page Multijoueur
//!   (`NOTES_MAX` au plus, les plus vieux oubliés).

use bevy::prelude::*;
use bs_protocole::PLACES_SALON;
use bs_reseau::amis::{Etat, EtatPresence, Lettre};
use bs_reseau::{EvenementAmis, PhaseCoop, ServiceAmis, code_saisi};
use bs_sauvegarde::amis::RefusAmi;

use crate::coop::Coop;
use crate::sauvegarde::Sauvegarde;

/// Messages reçus gardés au plus.
const NOTES_MAX: usize = 8;

/// Un message reçu, en attente d'une réponse du joueur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    /// `code` vous invite dans son salon.
    Invitation { code: String, salon: [u8; 6] },
    /// `code` voudrait jouer avec vous.
    Demande { code: String },
    /// `code` (`pseudo`) vous a ajouté.
    Ajout { code: String, pseudo: String },
}

impl Note {
    pub fn code(&self) -> &str {
        match self {
            Self::Invitation { code, .. } | Self::Demande { code } | Self::Ajout { code, .. } => {
                code
            }
        }
    }
}

#[derive(Resource)]
pub struct ReseauAmis {
    pub service: ServiceAmis,
    pub notes: Vec<Note>,
    /// Ami à inviter dès que notre salon (en train de s'ouvrir) aura son code.
    pub a_inviter: Option<String>,
    /// Change quand la page Multijoueur est à refaire (présences, notes).
    pub version: u64,
}

impl ReseauAmis {
    fn nouveau(sauvegarde: &Sauvegarde) -> Self {
        let a = &sauvegarde.amis;
        let etat = Etat {
            pseudo: sauvegarde.profil.pseudo.clone(),
            joueurs: 1,
            ..default()
        };
        let mut service =
            ServiceAmis::nouveau(&bs_reseau::adresse_relais(), &a.code, &a.secret, etat);
        for ami in &a.amis {
            service.suivre(&ami.code, ami.cle);
        }
        Self {
            service,
            notes: Vec::new(),
            a_inviter: None,
            version: 0,
        }
    }

    fn noter(&mut self, note: Note) {
        // Une seule note par ami et par sorte : la dernière
        self.notes.retain(|n| {
            std::mem::discriminant(n) != std::mem::discriminant(&note) || n.code() != note.code()
        });
        if self.notes.len() >= NOTES_MAX {
            self.notes.remove(0);
        }
        self.notes.push(note);
        self.version += 1;
    }

    /// Ajoute un ami (sauvegardé, suivi, prévenu).
    pub fn ajouter(
        &mut self,
        sauvegarde: &mut Sauvegarde,
        saisie: &str,
        pseudo: &str,
    ) -> Result<(), RefusAmi> {
        let ami = sauvegarde.amis.ajouter(saisie, pseudo)?;
        let code = ami.code.clone();
        self.service.suivre(&code, None);
        self.service.poster(&code, &Lettre::Ajout);
        self.notes
            .retain(|n| !matches!(n, Note::Ajout { code: c, .. } if *c == code));
        self.version += 1;
        Ok(())
    }

    pub fn retirer(&mut self, sauvegarde: &mut Sauvegarde, code: &str) {
        sauvegarde.amis.retirer(code);
        self.service.oublier(code);
        self.version += 1;
    }

    /// Oublie la note de ce rang.
    pub fn effacer(&mut self, k: usize) -> Option<Note> {
        (k < self.notes.len()).then(|| {
            self.version += 1;
            self.notes.remove(k)
        })
    }
}

/// Première entrée au menu : le service du profil actif.
pub fn creer(mut commandes: Commands, sauvegarde: Res<Sauvegarde>, amis: Option<Res<ReseauAmis>>) {
    if amis.is_none() && std::env::var_os("BS_SANS_AMIS").is_none() {
        commandes.insert_resource(ReseauAmis::nouveau(&sauvegarde));
    }
}

/// Ce que l'on publie de soi.
fn etat(sauvegarde: &Sauvegarde, coop: Option<&Coop>) -> Etat {
    let mut e = Etat {
        pseudo: sauvegarde.profil.pseudo.clone(),
        joueurs: 1,
        ..default()
    };
    let Some(c) = coop else {
        return e;
    };
    let joueurs = c.joueurs().len().max(1);
    e.joueurs = joueurs.min(usize::from(u8::MAX)) as u8;
    match c.phase() {
        PhaseCoop::Salon => {
            e.etat = EtatPresence::Salon;
            if joueurs < usize::from(PLACES_SALON) {
                e.salon = c.code().as_deref().and_then(code_saisi);
            }
        }
        PhaseCoop::Chargement { .. } | PhaseCoop::Partie => e.etat = EtatPresence::Partie,
        PhaseCoop::Connexion | PhaseCoop::Finie(_) => {}
    }
    e
}

/// À chaque image : publie, relève le service, retient ce qu'il apprend.
pub fn suivre(
    mut commandes: Commands,
    amis: Option<ResMut<ReseauAmis>>,
    mut sauvegarde: ResMut<Sauvegarde>,
    coop: Option<Res<Coop>>,
    temps: Res<Time>,
    mut attente: Local<f32>,
) {
    let Some(mut amis) = amis else {
        return;
    };
    if amis.service.code() != sauvegarde.amis.code {
        // Autre profil : autre code, autre clé, autres amis
        commandes.insert_resource(ReseauAmis::nouveau(&sauvegarde));
        return;
    }
    // Où on en est : regardé deux fois par seconde (sans allouer à chaque image)
    *attente -= temps.delta_secs();
    if *attente <= 0.0 {
        *attente = 0.5;
        let e = etat(&sauvegarde, coop.as_deref());
        if let Some(code) = amis.a_inviter.take() {
            match (&e.salon, coop.as_deref()) {
                (Some(s), Some(c)) if c.est_hote() => {
                    amis.service.poster(&code, &Lettre::Invitation(*s));
                }
                // Salon pas encore ouvert : on réessaie
                (_, Some(c)) if c.est_hote() && c.finie().is_none() => {
                    amis.a_inviter = Some(code);
                }
                _ => {}
            }
        }
        amis.service.publier(e);
    }
    amis.service.maj();
    while let Some(ev) = amis.service.evenement() {
        match ev {
            EvenementAmis::Cle { code, cle } => {
                if !sauvegarde.amis.lier_cle(&code, cle) {
                    warn!("ami {code} : clé différente de celle retenue, ignoré");
                }
            }
            EvenementAmis::Pseudo { code, pseudo } => sauvegarde.amis.renommer(&code, &pseudo),
            EvenementAmis::Ajout { code, pseudo } => {
                if sauvegarde.amis.amis.iter().all(|a| a.code != code) {
                    amis.noter(Note::Ajout { code, pseudo });
                }
            }
            EvenementAmis::Invitation { code, salon } => {
                amis.noter(Note::Invitation { code, salon });
            }
            EvenementAmis::Demande { code } => amis.noter(Note::Demande { code }),
            EvenementAmis::Change => amis.version += 1,
        }
    }
}
