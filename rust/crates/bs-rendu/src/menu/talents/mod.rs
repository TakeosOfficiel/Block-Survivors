//! Page des Talents du menu, à la manière des jeux de rôle en ligne : en haut le titre,
//! les classes et les points restants (`entete`) ; au centre les deux arbres de la
//! classe côte à côte, en grilles (`grille`), qui se partagent les points, et la fiche
//! du talent choisi en infobulle (`fiche`). Un clic choisit une case, un second clic
//! sur la même l'apprend. « Retour » et « Réinitialiser » (les deux arbres) sont des
//! liens en bas. Échap ramène au menu.

mod entete;
mod fiche;
mod grille;

pub use grille::survoler_cases;

use bevy::prelude::*;
use bs_donnees::progression::{Arbre, Noeud};

use super::vue::Lien;
use super::{ActionMenu, Icones};
use crate::ecrans::charte::{self, IVOIRE, OR_VIF, Relief, T_CORPS};
use crate::ecrans::style::{OR, Teinte};
use crate::lancement::DonneesJeu;
use crate::sauvegarde::Sauvegarde;

/// Bouton choisi des autres pages (Collection, Profil).
pub(in crate::menu) const CHOISI: Teinte = Teinte {
    bord: OR,
    fond: Color::srgba_u8(60, 45, 10, 153),
};
/// Libellé d'un lien non choisi.
pub(super) const ETEINT: Color = Color::srgb_u8(0x93, 0x9b, 0x98);

/// Classe, arbre et talent affichés ; réinitialisation en attente de confirmation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EtatTalents {
    pub personnage: usize,
    pub arbre: usize,
    pub noeud: usize,
    pub confirmation: bool,
}

/// Bouton de la page.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionTalent {
    Classe(usize),
    /// Case d'un arbre : la choisir, ou l'apprendre si elle l'est déjà.
    Noeud(usize, usize),
    Acheter,
    Reinitialiser,
    Annuler,
    Option(usize),
    BasculerArme,
}

/// Ce que la page lit pour se construire.
#[derive(Clone, Copy)]
pub struct Contexte<'a> {
    pub dj: &'a DonneesJeu,
    pub sauvegarde: &'a Sauvegarde,
    pub etat: EtatTalents,
}

impl Contexte<'_> {
    pub(super) fn id(&self) -> &'static str {
        let d = self.dj.d;
        d.personnages
            .personnages
            .get_index(self.etat.personnage)
            .map_or("", |(id, _)| id.as_str())
    }

    pub(super) fn arbre(&self) -> &'static Arbre {
        &self.dj.d.talents.arbres[self.id()][self.etat.arbre]
    }

    pub(super) fn noeud(&self) -> &'static Noeud {
        let noeuds = &self.arbre().noeuds;
        noeuds.get(self.etat.noeud).unwrap_or(&noeuds[0])
    }
}

/// Applique une action ; vrai si la page est à refaire. Toute autre action qu'une
/// réinitialisation confirmée annule la confirmation en attente.
pub fn agir(
    dj: &DonneesJeu,
    sauvegarde: &mut Sauvegarde,
    etat: &mut EtatTalents,
    action: ActionTalent,
) -> bool {
    let c = Contexte {
        dj,
        sauvegarde,
        etat: *etat,
    };
    let (p, a, n) = (c.id(), etat.arbre, c.noeud());
    let confirmee = std::mem::take(&mut etat.confirmation);
    let prog = &mut sauvegarde.progression;
    match action {
        ActionTalent::Classe(k) => {
            etat.personnage = k;
            etat.arbre = 0;
            etat.noeud = 0;
        }
        ActionTalent::Noeud(arbre, k) if (arbre, k) == (a, etat.noeud) => {
            prog.acheter(p, a, n);
        }
        ActionTalent::Noeud(arbre, k) => (etat.arbre, etat.noeud) = (arbre, k),
        ActionTalent::Acheter => {
            prog.acheter(p, a, n);
        }
        ActionTalent::Reinitialiser if confirmee => {
            prog.reinitialiser(p, 0);
            prog.reinitialiser(p, 1);
        }
        ActionTalent::Reinitialiser => etat.confirmation = true,
        ActionTalent::Annuler => {}
        ActionTalent::Option(i) => prog.choisir_option(p, a, n, i),
        ActionTalent::BasculerArme => {
            let actif = prog.actif(p, a, n);
            prog.choisir_arme_depart(p, a, (!actif).then_some(n));
        }
    }
    true
}

/// Texte de la page (Luckiest Guy si gras et grand, Inter sinon).
pub(super) fn ecrire(
    commandes: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    (gras, taille, couleur): (bool, f32, Color),
) -> Entity {
    let t = ic.polices.texte(t, gras, taille, couleur);
    commandes.spawn((t, ChildOf(parent))).id()
}

pub(super) fn rangee(commandes: &mut Commands, parent: Entity, ecart: f32) -> Entity {
    commandes
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(ecart),
                ..default()
            },
            ChildOf(parent),
        ))
        .id()
}

/// Lien sans cadre (libellé gras, or au survol) ; sans action, simple texte.
pub(super) fn lien(
    commandes: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    (taille, couleur): (f32, Color),
    action: Option<ActionMenu>,
) -> Entity {
    let b = rangee(commandes, parent, 10.0);
    commandes.entity(b).entry::<Node>().and_modify(|mut n| {
        n.padding = UiRect::vertical(Val::Px(4.0));
    });
    if let Some(a) = action {
        commandes.entity(b).insert((Button, Lien(couleur), a));
    }
    ecrire(commandes, ic, b, t, (true, taille, couleur));
    b
}

/// Trait d'or sous un onglet choisi.
pub(super) fn souligner(commandes: &mut Commands, parent: Entity) {
    commandes.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            right: Val::ZERO,
            bottom: Val::Px(-6.0),
            height: Val::Px(3.0),
            border_radius: BorderRadius::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(OR_VIF),
        BoxShadow::new(
            OR_VIF.with_alpha(0.5),
            Val::ZERO,
            Val::ZERO,
            Val::ZERO,
            Val::Px(6.0),
        ),
        ChildOf(parent),
    ));
}

/// Grand bouton vert de l'action principale (seul bouton en relief de la page).
pub(super) fn bouton(
    commandes: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    action: ActionTalent,
) -> Entity {
    let h = charte::H_PRINCIPAL;
    let b = charte::bouton_relief(commandes, parent, Relief::PRINCIPAL, h, ());
    commandes.entity(b).insert(ActionMenu::Talent(action));
    ecrire(commandes, ic, b, t, (true, T_CORPS + 2.0, IVOIRE));
    b
}

/// Fond de la page : de l'ardoise des profondeurs en tuiles, très assombrie, voilée
/// vers les bords.
fn mur(commandes: &mut Commands, ic: &mut Icones, racine: Entity) {
    let plein = || Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    };
    commandes.spawn((
        ImageNode {
            image: ic.icone("block/deepslate_tiles"),
            color: Color::srgb(0.22, 0.22, 0.24),
            image_mode: NodeImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 4.0,
            },
            ..default()
        },
        plein(),
        ChildOf(racine),
    ));
    let noir = |a: f32| Color::srgba(0.0, 0.0, 0.0, a);
    let halo = RadialGradient::new(
        UiPosition::CENTER,
        RadialGradientShape::FarthestCorner,
        vec![
            ColorStop::new(noir(0.1), Val::Percent(0.0)),
            ColorStop::new(noir(0.75), Val::Percent(100.0)),
        ],
    );
    commandes.spawn((
        plein(),
        BackgroundGradient(vec![halo.into()]),
        ChildOf(racine),
    ));
}

/// Construit la page ; renvoie sa racine.
pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones) -> Entity {
    let racine = commandes
        .spawn((
            Name::new("talents"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(charte::FOND),
        ))
        .id();
    mur(commandes, ic, racine);
    entete::construire(commandes, c, ic, racine);
    let corps = commandes
        .spawn((
            Node {
                flex_grow: 1.0,
                min_height: Val::ZERO,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    // Arbres et fiche alignés en haut, l'ensemble centré dans la page.
    let rang = commandes
        .spawn((
            Node {
                align_items: AlignItems::FlexStart,
                column_gap: Val::Px(22.0),
                ..default()
            },
            ChildOf(corps),
        ))
        .id();
    grille::construire(commandes, c, ic, rang, 0);
    grille::construire(commandes, c, ic, rang, 1);
    fiche::construire(commandes, c, ic, rang);
    entete::pied(commandes, c, ic, racine);
    racine
}
