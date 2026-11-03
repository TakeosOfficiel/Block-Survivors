//! Accueil (docs/CHARTE-UI.md) : la scène d'abord. Plein écran sur la photo de l'arène
//! vue de haut (`fond`), le héros 3D au centre, cerné de mobs ; par-dessus, à gauche
//! sur une ombre profonde, une colonne : le logo, le grand « JOUER », le résumé
//! classe · arène et les liens (`colonne`), puis Options, Nouveautés et Quitter en bas
//! (`bas`) ; le profil et « Apparence » en haut à droite. Un clic sur la classe ou
//! l'arène du résumé ouvre son volet à droite de la colonne (`volet`) ; un choix le
//! referme. Refait en entier à chaque changement (hors de la boucle de jeu).

mod arenes;
mod bas;
mod classes;
mod colonne;
mod logo;
mod volet;

pub use colonne::survoler_liens;

use bevy::prelude::*;

use super::{ActionMenu, Icones, Volet};
use crate::ecrans::charte::{self, AIDE, IVOIRE, MARGE, Relief, T_CORPS};
use crate::hud::police::TextePixel;
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::sauvegarde::Sauvegarde;

pub(super) const FOND: Color = Color::srgb_u8(0x0b, 0x0e, 0x16);
/// Marge du haut de l'écran.
const HAUT: f32 = 24.0;
const OMBRE: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);

/// Outils de construction : commandes, polices et icônes.
pub(super) struct Pinceau<'a, 'w, 's, 'i> {
    pub c: &'a mut Commands<'w, 's>,
    pub ic: &'a mut Icones<'i>,
}

impl Pinceau<'_, '_, '_, '_> {
    pub fn noeud(&mut self, parent: Entity, n: Node) -> Entity {
        self.c.spawn((n, ChildOf(parent))).id()
    }

    /// Texte Inter (`gras` : demi-gras), ombré.
    pub fn texte(&mut self, parent: Entity, t: &str, gras: bool, taille: f32, c: Color) -> Entity {
        let ombre = TextShadow {
            offset: Vec2::new(0.0, 2.0),
            color: OMBRE,
        };
        let t = self.ic.polices.texte(t, gras, taille, c);
        self.c.spawn((t, ombre, ChildOf(parent))).id()
    }

    /// Titre de panneau en police du jeu.
    pub fn section(&mut self, parent: Entity, t: &str) {
        let e = self
            .c
            .spawn((TextePixel::nouveau(t, IVOIRE, 2.5), ChildOf(parent)))
            .id();
        self.c.entity(e).insert(Node {
            margin: UiRect::bottom(Val::Px(12.0)),
            ..default()
        });
    }

    /// Icône d'un sprite du jeu, carrée.
    pub fn icone(&mut self, parent: Entity, chemin: &str, cote: f32) -> Entity {
        let image = self.ic.icone(chemin);
        let n = Node {
            width: Val::Px(cote),
            height: Val::Px(cote),
            flex_shrink: 0.0,
            ..default()
        };
        self.c
            .spawn((ImageNode::new(image), n, ChildOf(parent)))
            .id()
    }

    /// Case creusée autour d'une icône.
    pub fn case(&mut self, parent: Entity, chemin: &str, cote: f32, icone: f32) -> Entity {
        let c = self.noeud(
            parent,
            Node {
                width: Val::Px(cote),
                height: Val::Px(cote),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
        );
        self.c.entity(c).insert((
            BackgroundColor(charte::FOND),
            BorderColor::all(charte::BORD),
        ));
        self.icone(c, chemin, icone);
        c
    }

    /// Bouton en relief avec son libellé.
    pub fn bouton(
        &mut self,
        parent: Entity,
        t: &str,
        relief: Relief,
        action: ActionMenu,
    ) -> Entity {
        let b = charte::bouton_relief(self.c, parent, relief, charte::H_SECONDAIRE, action);
        self.texte(b, t, true, T_CORPS, IVOIRE);
        b
    }
}

/// Nœud absolu placé par ses bords.
fn place(gauche: Option<Val>, droite: Option<Val>, haut: Option<Val>, bas: Option<Val>) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: gauche.unwrap_or(Val::Auto),
        right: droite.unwrap_or(Val::Auto),
        top: haut.unwrap_or(Val::Auto),
        bottom: bas.unwrap_or(Val::Auto),
        ..default()
    }
}

/// Construit l'accueil ; renvoie sa racine.
pub fn construire(
    commandes: &mut Commands,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    ic: &mut Icones,
    (choix, ouvert): (ChoixPartie, Volet),
) -> Entity {
    let racine = commandes
        .spawn((
            Name::new("menu"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            // Transparent : la scène et le héros 3D sont dessous
        ))
        .id();
    let mut p = Pinceau { c: commandes, ic };
    let gauche = p.noeud(
        racine,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..place(Some(Val::Px(MARGE)), None, Some(Val::Px(HAUT + 12.0)), None)
        },
    );
    logo::construire(&mut p, gauche);
    colonne::construire(&mut p, dj, sauvegarde, choix, gauche);
    profil(&mut p, dj, sauvegarde, choix, racine);
    bas::construire(&mut p, sauvegarde, racine);
    volet::construire(&mut p, dj, sauvegarde, (choix, ouvert), racine);
    racine
}

/// En haut à droite : « Apparence », le dé du pseudo, le profil (visage, pseudo).
fn profil(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    racine: Entity,
) {
    let droite = p.noeud(
        racine,
        Node {
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            ..place(None, Some(Val::Px(MARGE)), Some(Val::Px(HAUT)), None)
        },
    );
    p.bouton(
        droite,
        "Apparence",
        Relief::SECONDAIRE,
        ActionMenu::OuvrirProfil,
    );
    bas::de(p, droite);
    let profil = charte::bouton_relief(
        p.c,
        droite,
        Relief::SECONDAIRE,
        charte::H_SECONDAIRE,
        ActionMenu::OuvrirProfils,
    );
    p.c.entity(profil).entry::<Node>().and_modify(|mut n| {
        n.padding = UiRect::new(Val::Px(8.0), Val::Px(12.0), Val::ZERO, Val::ZERO)
    });
    let apparence = sauvegarde
        .depart(dj.d, choix.personnage)
        .apparence
        .unwrap_or(&dj.d.personnages.personnages[choix.personnage].apparence);
    super::profil::visage(p.c, p.ic, profil, &apparence.skin, 28.0);
    p.texte(profil, &sauvegarde.profil.pseudo, true, T_CORPS, IVOIRE);
    p.texte(profil, "⌄", true, T_CORPS, AIDE);
}
