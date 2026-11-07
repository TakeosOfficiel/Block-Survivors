//! Choix d'une partie et passages entre le menu et la partie (src/lancement.ts).
//!
//! Arènes proposées : celles du jeu, puis les maps construites dans Minecraft (dossier
//! cartes/ du dépôt, puis maps du joueur, glissées sur la fenêtre). Le dernier choix
//! (classe, arène) est gardé dans le profil (clé `choix`, même JSON que le jeu TS). Au
//! retour au menu, tout ce que la partie a créé est retiré (entités affichées,
//! ressources de la partie), comme le rechargement de page du jeu TS.

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_donnees::arenes::Arene;
use bs_monde::generer_monde;
use bs_monde::minecraft::cartes::{Carte, cartes, definition};
use bs_monde::minecraft::dossier::cartes_du_dossier;
use bs_regles::partie::Partie;
use serde_json::json;

use crate::sauvegarde::Sauvegarde;
use crate::{Monde, PartieEnCours, SourceArene, carte_mc, simulation};

/// D'où vient une arène du menu.
#[derive(Clone, Copy)]
pub enum SourceMenu {
    /// Arène du jeu : index dans les données.
    Generee(usize),
    /// Map Minecraft embarquée.
    Carte(&'static Carte),
}

#[derive(Clone, Copy)]
pub struct AreneMenu {
    pub def: &'static Arene,
    pub source: SourceMenu,
}

/// Données du jeu et arènes proposées (lues une fois).
#[derive(Resource)]
pub struct DonneesJeu {
    pub d: &'static Donnees,
    pub arenes: Vec<AreneMenu>,
}

impl DonneesJeu {
    /// Arènes du jeu, maps embarquées puis maps du joueur (dans `dossier_cartes`).
    pub fn nouvelles(d: &'static Donnees, dossier_cartes: &std::path::Path) -> Self {
        let arenes: Vec<AreneMenu> = d
            .arenes
            .arenes
            .iter()
            .enumerate()
            .map(|(k, def)| AreneMenu {
                def,
                source: SourceMenu::Generee(k),
            })
            .collect();
        let mut dj = Self { d, arenes };
        let mut erreurs = Vec::new();
        let du_joueur = cartes_du_dossier(dossier_cartes, &mut erreurs);
        for carte in cartes().iter().chain(du_joueur) {
            if let Err(e) = dj.ajouter(carte) {
                erreurs.push(e);
            }
        }
        for e in erreurs {
            warn!("map écartée : {e}");
        }
        dj
    }

    /// Ajoute une map (ou remplace celle du même identifiant) ; renvoie son index.
    pub fn ajouter(&mut self, carte: &'static Carte) -> Result<usize, String> {
        let def = definition(self.d, carte).map_err(|e| format!("{} : {e}", carte.id))?;
        let a = AreneMenu {
            def: Box::leak(Box::new(def)),
            source: SourceMenu::Carte(carte),
        };
        Ok(match self.arene(carte.id) {
            Some(k) => {
                self.arenes[k] = a;
                k
            }
            None => {
                self.arenes.push(a);
                self.arenes.len() - 1
            }
        })
    }

    /// Index de l'arène d'identifiant `id`.
    pub fn arene(&self, id: &str) -> Option<usize> {
        self.arenes.iter().position(|a| a.def.id == id)
    }
}

/// Classe (index dans les données) et arène (index dans `DonneesJeu::arenes`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChoixPartie {
    pub personnage: usize,
    pub arene: usize,
}

/// Dernier choix gardé dans le profil, sinon la classe et l'arène par défaut.
pub fn choix_memorise(dj: &DonneesJeu, sauvegarde: &Sauvegarde) -> ChoixPartie {
    let d = dj.d;
    let p = &sauvegarde.profils;
    let lu: Option<serde_json::Value> = p.stockage().lire(&p.cle("choix"));
    let lu = lu.unwrap_or_default();
    let personnages = &d.personnages.personnages;
    let personnage = lu["personnage"]
        .as_str()
        .and_then(|id| personnages.get_index_of(id))
        .or_else(|| personnages.get_index_of(&d.personnages.par_defaut))
        .unwrap_or(0);
    let arene = lu["arene"]
        .as_str()
        .and_then(|id| dj.arene(id))
        .or_else(|| dj.arene(&d.arenes.par_defaut))
        .unwrap_or(0);
    ChoixPartie { personnage, arene }
}

/// Retient le choix pour le prochain passage au menu.
pub fn memoriser(dj: &DonneesJeu, sauvegarde: &Sauvegarde, choix: ChoixPartie) {
    let personnage =
        dj.d.personnages
            .personnages
            .get_index(choix.personnage)
            .map_or("", |(id, _)| id.as_str());
    let v = json!({ "personnage": personnage, "arene": dj.arenes[choix.arene].def.id });
    let p = &sauvegarde.profils;
    p.stockage().ecrire(&p.cle("choix"), &v);
}

/// Partie prête à charger : map à importer, ou arène générée.
pub enum PartiePreparee {
    Import(Box<carte_mc::Import>),
    Generee(Box<(PartieEnCours, Monde)>),
}

impl PartiePreparee {
    pub fn nouvelle(
        dj: &DonneesJeu,
        sauvegarde: &Sauvegarde,
        choix: ChoixPartie,
        graine: u32,
    ) -> Result<Self, String> {
        let d = dj.d;
        let depart = sauvegarde.depart(d, choix.personnage);
        let perso = choix.personnage;
        Ok(match dj.arenes[choix.arene].source {
            SourceMenu::Carte(carte) => Self::Import(Box::new(carte_mc::Import::nouvel(
                d, carte, perso, graine, depart,
            )?)),
            SourceMenu::Generee(index) => {
                let partie = Partie::nouvelle(d, index, perso, graine, true);
                let monde = generer_monde(d, partie.etat.arene);
                let mut jeu =
                    PartieEnCours::nouvelle(partie, graine, SourceArene::Generee(index), depart);
                simulation::avancer_debut(&mut jeu);
                Self::Generee(Box::new((jeu, Monde(monde))))
            }
        })
    }

    pub fn inserer(self, commandes: &mut Commands) {
        match self {
            Self::Import(i) => commandes.insert_resource(*i),
            Self::Generee(partie) => {
                let (jeu, monde) = *partie;
                commandes.insert_resource(jeu);
                commandes.insert_resource(monde);
            }
        }
    }
}

/// Graine d'une partie lancée du menu (horloge du système).
pub fn graine_au_hasard() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(1, |t| t.subsec_nanos() ^ t.as_secs() as u32)
}

/// Entité gardée d'une partie à l'autre (compteur de FPS).
#[derive(Component)]
pub struct Permanent;

/// Retour au menu : retire tout ce que la partie a créé.
#[allow(clippy::type_complexity)]
pub fn nettoyer(
    mut commandes: Commands,
    coop: Option<ResMut<crate::coop::Coop>>,
    racines: Query<
        Entity,
        (
            Or<(With<Transform>, With<Node>)>,
            Without<ChildOf>,
            Without<Permanent>,
        ),
    >,
) {
    for e in &racines {
        commandes.entity(e).despawn();
    }
    if let Some(mut coop) = coop {
        coop.quitter();
        commandes.remove_resource::<crate::coop::Coop>();
    }
    commandes.remove_resource::<PartieEnCours>();
    commandes.remove_resource::<Monde>();
    commandes.remove_resource::<carte_mc::CarteImportee>();
    commandes.remove_resource::<carte_mc::Import>();
    commandes.remove_resource::<crate::modeles::Rendus>();
    commandes.remove_resource::<crate::hud::Hud>();
    commandes.remove_resource::<crate::lumiere::VolumeLumineux>();
    commandes.insert_resource(crate::ecrans::Ecrans::default());
    // Partie lancée du menu : le joueur joue (pas de robot de démonstration)
    commandes.insert_resource(crate::ecran_cartes::EcranCartes::nouveau(true));
    commandes.insert_resource(crate::camera::Secousse::default());
    commandes.insert_resource(crate::camera::Cible::default());
}
