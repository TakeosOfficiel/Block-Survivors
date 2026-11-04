//! Page des Talents du menu, plein écran sans cadres : un mur de blocs (la texture de
//! l'arbre, `fond` dans les données) assombri sur les bords ; en haut
//! le titre, les classes en liens soulignés et les points disponibles (`entete`) ;
//! sous eux les deux spécialisations, puis l'arbre au centre (`schema`, défilant) ; la
//! fiche du talent consulté flotte à droite (`fiche`). « Retour » et « Réinitialiser »
//! sont de simples liens en bas. Échap ramène au menu.

mod entete;
mod fiche;
mod schema;

pub use schema::survoler_cases;

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
    Arbre(usize),
    Noeud(usize),
    Acheter,
    Utiliser,
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
            let id =
                dj.d.personnages
                    .personnages
                    .get_index(k)
                    .map_or("", |(i, _)| i);
            etat.arbre = prog.arbre_actif(id);
            etat.noeud = 0;
        }
        ActionTalent::Arbre(k) => {
            etat.arbre = k;
            etat.noeud = 0;
        }
        ActionTalent::Noeud(k) => etat.noeud = k,
        ActionTalent::Acheter => {
            prog.acheter(p, a, n);
        }
        ActionTalent::Utiliser => prog.choisir_arbre(p, a),
        ActionTalent::Reinitialiser if confirmee => prog.reinitialiser(p, a),
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

/// Fond de la page : la texture de bloc de l'arbre en mur (tuiles de 64), assombrie
/// vers les bords et sous l'en-tête.
fn mur(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, racine: Entity) {
    let plein = || Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    };
    commandes.spawn((
        ImageNode {
            image: ic.icone(&c.arbre().fond),
            color: Color::srgb(0.62, 0.62, 0.62),
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
        UiPosition::TOP_LEFT.at_percent(38.0, 55.0),
        RadialGradientShape::FarthestCorner,
        vec![
            ColorStop::new(noir(0.25), Val::Percent(0.0)),
            ColorStop::new(noir(0.55), Val::Percent(60.0)),
            ColorStop::new(noir(0.85), Val::Percent(100.0)),
        ],
    );
    let haut = LinearGradient::to_bottom(vec![
        ColorStop::new(noir(0.7), Val::Percent(0.0)),
        ColorStop::new(noir(0.0), Val::Percent(22.0)),
        ColorStop::new(noir(0.0), Val::Percent(85.0)),
        ColorStop::new(noir(0.6), Val::Percent(100.0)),
    ]);
    commandes.spawn((
        plein(),
        BackgroundGradient(vec![halo.into(), haut.into()]),
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
    mur(commandes, c, ic, racine);
    entete::construire(commandes, c, ic, racine);
    let corps = commandes
        .spawn((
            Node {
                flex_grow: 1.0,
                min_height: Val::ZERO,
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    let centre = commandes
        .spawn((
            Node {
                flex_grow: 1.0,
                min_width: Val::ZERO,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            ChildOf(corps),
        ))
        .id();
    entete::specialisations(commandes, c, ic, centre);
    schema::construire(commandes, c, ic, centre);
    fiche::construire(commandes, c, ic, corps);
    entete::pied(commandes, c, ic, racine);
    racine
}
