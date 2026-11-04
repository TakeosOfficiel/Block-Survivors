//! Page des Talents du menu, plein écran et à l'air libre (aucun panneau ni case) : à
//! gauche l'arbre choisi à plat (`colonnes`) : le tronc en bas, une colonne colorée
//! par branche au-dessus, des pastilles rondes (`case`) ; à droite
//! (`cote`) les classes, les deux arbres (qui se partagent les points), les points
//! restants et la fiche du talent choisi (`fiche`). Un clic choisit une pastille, un
//! second clic sur la même l'apprend. « ‹ Retour » et « Réinitialiser » en bas à
//! gauche ; Échap ramène à l'accueil.

mod case;
mod colonnes;
mod cote;
mod fiche;

pub use case::survoler_cases;

use bevy::prelude::*;
use bs_donnees::progression::{Arbre, Noeud};

use super::vue::Lien;
use super::{ActionMenu, Icones};
use crate::ecrans::charte::{self, IVOIRE, MARGE, Relief};
use crate::lancement::DonneesJeu;
use crate::sauvegarde::Sauvegarde;

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
        ActionTalent::Arbre(arbre) => (etat.arbre, etat.noeud) = (arbre, 0),
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

/// Grand bouton vert de l'action principale, en police du logo, comme « JOUER ».
pub(super) fn bouton(
    commandes: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    action: ActionTalent,
) -> Entity {
    let h = charte::H_PRINCIPAL + 8.0;
    let b = charte::bouton_relief(commandes, parent, Relief::PRINCIPAL, h, ());
    commandes.entity(b).insert(ActionMenu::Talent(action));
    // Arrondi : pas de bouton carré sur cette page
    commandes.entity(b).entry::<Node>().and_modify(|mut n| {
        n.border_radius = BorderRadius::MAX;
    });
    commandes.spawn((
        Text::new(t),
        TextFont {
            font: ic.polices.logo.clone(),
            font_size: 30.0,
            ..default()
        },
        TextColor(IVOIRE),
        TextShadow {
            offset: Vec2::new(0.0, 3.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.6),
        },
        ChildOf(b),
    ));
    b
}

/// Construit la page ; renvoie sa racine.
pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones) -> Entity {
    let racine = commandes
        .spawn((
            Name::new("talents"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::new(Val::Px(MARGE), Val::Px(MARGE), Val::Px(24.0), Val::Px(24.0)),
                column_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundGradient::from(RadialGradient::new(
                UiPosition::TOP_LEFT.at_percent(33.0, 80.0),
                RadialGradientShape::FarthestCorner,
                vec![
                    ColorStop::new(Color::srgb_u8(0x1b, 0x2a, 0x3c), Val::Percent(0.0)),
                    ColorStop::new(Color::srgb_u8(0x0b, 0x11, 0x19), Val::Percent(55.0)),
                    ColorStop::new(Color::srgb_u8(0x05, 0x07, 0x0a), Val::Percent(100.0)),
                ],
            )),
        ))
        .id();
    let gauche = commandes
        .spawn((
            Node {
                flex_grow: 1.0,
                min_width: Val::ZERO,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    colonnes::construire(commandes, c, ic, gauche);
    cote::construire(commandes, c, ic, racine);
    cote::pied(commandes, c, ic, racine);
    racine
}
