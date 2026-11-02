//! Écran de cartes (src/ui/ecranCartes.ts et `proposerCartes` de src/jeu.ts) : un niveau
//! en attente à la fois, puis les objets des coffres ouverts. Choix aux touches 1 à 4
//! (rangée du haut ou pavé numérique) ou à la souris, après un court délai qui évite un
//! choix involontaire en pleine action ; R relance le tirage (relances communes aux
//! niveaux et aux coffres). Emplacements pleins : 2e étape « Remplacer quel objet ? »,
//! d'où « Retour » rouvre les mêmes propositions du coffre. Tant que des cartes
//! attendent, la partie est en pause (`avancer` ne fait plus de pas).
//!
//! Le joueur choisit dès qu'il a touché au clavier (ou avec `BS_CARTES=1`) ; avant, le
//! robot de démonstration choisit et joue seul.

mod vue;

use bevy::prelude::*;
use bs_regles::equipement::ChoixEquipement;
use bs_regles::partie::Partie;
use bs_regles::textes::{Carte, Choix};

pub(crate) use vue::couleur_famille;
pub use vue::{BoutonCarte, BoutonRelancer};

use crate::PartieEnCours;
use crate::dessins::Dessins;
use crate::textures::Textures;

/// Délai avant qu'un clic ou une touche soit accepté (s).
const DELAI_ACTIVATION: f32 = 0.35;
/// Touches des cartes, dans l'ordre.
const TOUCHES: [[KeyCode; 2]; 4] = [
    [KeyCode::Digit1, KeyCode::Numpad1],
    [KeyCode::Digit2, KeyCode::Numpad2],
    [KeyCode::Digit3, KeyCode::Numpad3],
    [KeyCode::Digit4, KeyCode::Numpad4],
];
/// Carte survolée : soulevée (px).
const SOULEVEMENT: f32 = 8.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Etape {
    Niveau,
    Coffre { ender: bool },
    Remplacement { ender: bool },
}

/// État de l'écran : étape ouverte, cartes décrites, propositions du coffre en cours.
#[derive(Resource, Default)]
pub struct EcranCartes {
    /// Le joueur choisit lui-même (sinon le robot de démonstration).
    pub humain: bool,
    etape: Option<Etape>,
    cartes: Vec<Carte>,
    /// Propositions du coffre, gardées pour « Retour ».
    propositions: Vec<ChoixEquipement>,
    ouvert_depuis: f32,
    racine: Option<Entity>,
    /// Cartes jouées à cette image (envoyées aux autres joueurs en coop).
    pub(crate) jouees: Vec<Choix>,
}

impl EcranCartes {
    pub fn nouveau(humain: bool) -> Self {
        Self {
            humain,
            ..default()
        }
    }

    /// Des cartes sont affichées.
    pub fn ouvert(&self) -> bool {
        self.etape.is_some()
    }
}

/// Vrai quand des cartes attendent un choix (la partie est alors en pause).
pub fn en_attente(p: &Partie) -> bool {
    let e = &p.etat;
    e.issue.is_none() && !e.joueur.mort && (e.niveaux_en_attente > 0 || e.choix_coffre.restants > 0)
}

pub fn demander_textures(d: &bs_donnees::Donnees, demander: &mut impl FnMut(&str)) {
    for a in d.armes.values() {
        for e in a.eveils.iter().flatten() {
            demander(&e.icone);
        }
    }
    demander(&d.regles.progression.soin.icone);
}

/// Ouvre l'écran quand des cartes attendent, puis lit touches et clics.
#[allow(clippy::too_many_arguments)]
pub fn gerer(
    mut commandes: Commands,
    mut ecran: ResMut<EcranCartes>,
    mut jeu: ResMut<PartieEnCours>,
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    dessins: Res<Dessins>,
    touches: Res<ButtonInput<KeyCode>>,
    temps: Res<Time>,
    cartes: Query<(&Interaction, &BoutonCarte), Changed<Interaction>>,
    relance: Query<&Interaction, (Changed<Interaction>, With<BoutonRelancer>)>,
) {
    let maintenant = temps.elapsed_secs();
    let p = &mut jeu.partie;
    let e = &mut *ecran;
    e.jouees.clear();
    if e.etape.is_none() {
        if !e.humain || !en_attente(p) {
            return;
        }
        let etape = if p.etat.niveaux_en_attente > 0 {
            Etape::Niveau
        } else {
            Etape::Coffre {
                ender: p.etat.choix_coffre.ender > 0,
            }
        };
        e.etape = Some(etape);
        tirer(p, e);
    } else if maintenant - e.ouvert_depuis >= DELAI_ACTIVATION {
        // Touche ou clic : carte choisie, ou relance
        let mut choix = TOUCHES
            .iter()
            .position(|t| touches.any_just_pressed(t.iter().copied()));
        for (i, b) in &cartes {
            if *i == Interaction::Pressed {
                choix = Some(b.0);
            }
        }
        let relancer = touches.just_pressed(KeyCode::KeyR)
            || relance.iter().any(|i| *i == Interaction::Pressed);
        if let Some(k) = choix.filter(|&k| k < e.cartes.len()) {
            choisir(p, e, k);
        } else if relancer && !matches!(e.etape, Some(Etape::Remplacement { .. })) {
            if p.etat.relances_restantes == 0 {
                return;
            }
            p.etat.relances_restantes -= 1;
            tirer(p, e);
        } else {
            return;
        }
    } else {
        return;
    }
    // Écran à refaire (ouvert, relancé, autre étape) ou à fermer
    if let Some(r) = e.racine.take() {
        commandes.entity(r).despawn();
    }
    let Some(etape) = e.etape else {
        return;
    };
    e.ouvert_depuis = maintenant;
    let d = p.d;
    let titre = match etape {
        Etape::Niveau => {
            let niveau = p.etat.joueur.niveau + 1 - p.etat.niveaux_en_attente;
            format!("Niveau {niveau} !")
        }
        Etape::Coffre { ender } => {
            let id = if ender { "coffre_ender" } else { "coffre" };
            d.coffres
                .coffres
                .get(id)
                .map_or_else(|| "Coffre !".into(), |c| c.titre.clone())
        }
        Etape::Remplacement { .. } => "Remplacer quel objet ?".into(),
    };
    let relances = match etape {
        Etape::Remplacement { .. } => None,
        _ => Some(p.etat.relances_restantes),
    };
    let contexte = vue::Contexte {
        serveur: &serveur,
        textures: &textures,
        dessins: &dessins,
    };
    e.racine = Some(vue::construire(
        &mut commandes,
        &contexte,
        &titre,
        &e.cartes,
        relances,
    ));
}

/// Carte survolée : soulevée.
pub fn survoler(
    mut cartes: Query<(&Interaction, &mut UiTransform), (Changed<Interaction>, With<BoutonCarte>)>,
) {
    for (i, mut t) in &mut cartes {
        let y = if *i == Interaction::None {
            0.0
        } else {
            -SOULEVEMENT
        };
        t.translation = Val2::px(0.0, y);
    }
}

/// Nouveau tirage de l'étape (niveau ou coffre), décrit.
fn tirer(p: &mut Partie, e: &mut EcranCartes) {
    match e.etape {
        Some(Etape::Niveau) => {
            p.tirer_cartes();
        }
        Some(Etape::Coffre { ender }) => {
            let ouverts = p.ramassables.coffres.payants_ouverts;
            p.tirer_equipement(ender, ouverts);
            e.propositions.clone_from(&p.equipement.proposees);
        }
        _ => {}
    }
    decrire(p, e);
}

/// Décrit les cartes proposées de l'étape ouverte.
fn decrire(p: &Partie, e: &mut EcranCartes) {
    e.cartes.clear();
    if e.etape == Some(Etape::Niveau) {
        let liste = p.cartes.proposees.iter();
        e.cartes.extend(liste.map(|&a| p.decrire_amelioration(a)));
    } else {
        let liste = p.equipement.proposees.iter();
        e.cartes.extend(liste.map(|&c| p.decrire_equipement(c)));
    }
}

/// Applique la carte `k` ; l'étape suivante (remplacement, retour au coffre) ou la
/// fermeture est notée dans `e.etape`.
fn choisir(p: &mut Partie, e: &mut EcranCartes, k: usize) {
    match e.cartes[k].choix {
        Choix::Niveau(a) => {
            p.appliquer_amelioration(a);
            e.jouees.push(Choix::Niveau(a));
            p.etat.niveaux_en_attente = p.etat.niveaux_en_attente.saturating_sub(1);
            e.etape = None;
        }
        Choix::Coffre(c) => {
            let ender = matches!(
                e.etape,
                Some(Etape::Coffre { ender: true } | Etape::Remplacement { ender: true })
            );
            match c {
                ChoixEquipement::Retour => {
                    p.equipement.proposees.clone_from(&e.propositions);
                    e.etape = Some(Etape::Coffre { ender });
                    decrire(p, e);
                }
                ChoixEquipement::Objet(o) if p.remplacement_necessaire(o) => {
                    p.cartes_remplacement(o);
                    e.etape = Some(Etape::Remplacement { ender });
                    decrire(p, e);
                }
                _ => {
                    p.appliquer_equipement(c);
                    e.jouees.push(Choix::Coffre(c));
                    let choix = &mut p.etat.choix_coffre;
                    choix.restants = choix.restants.saturating_sub(1);
                    if ender {
                        choix.ender = choix.ender.saturating_sub(1);
                    }
                    e.etape = None;
                }
            }
        }
    }
}

#[cfg(test)]
mod essais {
    use super::*;
    use bs_donnees::Donnees;
    use bs_donnees::equipement::Emplacement;

    /// Niveau, puis coffre aux bagues pleines : remplacement, retour, remplacement.
    #[test]
    fn niveau_remplacement_retour() {
        let d: &'static Donnees = Box::leak(Box::new(Donnees::embarquees().unwrap()));
        let mut p = Partie::nouvelle(d, 0, 0, 5, false);
        let mut e = EcranCartes::nouveau(true);

        p.etat.niveaux_en_attente = 1;
        e.etape = Some(Etape::Niveau);
        tirer(&mut p, &mut e);
        assert!(!e.cartes.is_empty());
        choisir(&mut p, &mut e, 0);
        assert!(e.etape.is_none() && p.etat.niveaux_en_attente == 0);

        // Bagues portées jusqu'à ce que tout soit plein ; une de plus en proposition
        let bagues: Vec<usize> = (d.equipement.objets.values().enumerate())
            .filter(|(_, o)| o.emplacement == Emplacement::Bague)
            .map(|(k, _)| k)
            .collect();
        let n = p.equipement.bagues.len();
        for &b in &bagues[..n] {
            p.appliquer_equipement(ChoixEquipement::Objet(b));
        }
        let nouvelle = bagues[n];
        assert!(p.remplacement_necessaire(nouvelle));
        p.etat.choix_coffre.restants = 1;
        e.etape = Some(Etape::Coffre { ender: false });
        p.equipement.proposees = vec![ChoixEquipement::Objet(nouvelle)];
        e.propositions.clone_from(&p.equipement.proposees);
        decrire(&p, &mut e);

        choisir(&mut p, &mut e, 0);
        assert!(e.etape == Some(Etape::Remplacement { ender: false }));
        assert_eq!(
            e.cartes.len(),
            n + 1,
            "un objet porté par carte, plus Retour"
        );
        choisir(&mut p, &mut e, n);
        assert!(e.etape == Some(Etape::Coffre { ender: false }));
        assert_eq!(e.cartes.len(), 1);
        choisir(&mut p, &mut e, 0);
        choisir(&mut p, &mut e, 0);
        assert!(e.etape.is_none() && p.etat.choix_coffre.restants == 0);
        assert!(p.equipement.portes().any(|o| o.objet == nouvelle));
    }
}
