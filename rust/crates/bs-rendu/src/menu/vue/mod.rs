//! Accueil, écran de préparation de partie (docs/CHARTE-UI.md) posé sur la photo de
//! l'arène (`fond`) : bandeau du haut (logo, bouton du profil), trois colonnes (la
//! classe à gauche dans `classes`, le héros 3D au centre avec « Apparence » sous ses
//! pieds, l'arène à droite dans `arenes`), pied d'écran (`bas` : liens, « Jouer »,
//! « Multijoueur », « Quitter »). Refait en entier à chaque changement de choix (hors
//! de la boucle de jeu).

mod arenes;
mod bas;
mod classes;

use bevy::prelude::*;

use super::{ActionMenu, Icones};
use crate::ecrans::charte::{self, AIDE, IVOIRE, MARGE, Relief, T_CORPS};
use crate::hud::police::TextePixel;
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::sauvegarde::Sauvegarde;

pub(super) const FOND: Color = Color::srgb_u8(0x0b, 0x0e, 0x16);
/// Hauteur du bandeau du haut et du pied d'écran.
const HAUT: f32 = 72.0;
const PIED: f32 = 88.0;
/// Largeur des colonnes de la classe et de l'arène.
const LARGEUR_CLASSE: f32 = 300.0;
const LARGEUR_ARENE: f32 = 320.0;
/// Hauteur du héros au-dessus d'« Apparence » (ses pieds sur la même ligne).
const PLACE_HEROS: f32 = 468.0;
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
    choix: ChoixPartie,
) -> Entity {
    let racine = commandes
        .spawn((
            Name::new("menu"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            // Transparent : la photo et le héros 3D sont dessous
        ))
        .id();
    let mut p = Pinceau { c: commandes, ic };
    bandeau(&mut p, dj, sauvegarde, choix, racine);
    let milieu = p.noeud(
        racine,
        Node {
            flex_grow: 1.0,
            min_height: Val::ZERO,
            padding: UiRect::horizontal(Val::Px(MARGE)),
            column_gap: Val::Px(24.0),
            align_items: AlignItems::FlexStart,
            ..default()
        },
    );
    let colonne = |largeur| Node {
        width: Val::Px(largeur),
        flex_shrink: 0.0,
        max_height: Val::Percent(100.0),
        ..default()
    };
    let gauche = charte::panneau(p.c, milieu, colonne(LARGEUR_CLASSE));
    classes::construire(&mut p, dj, sauvegarde, choix, gauche);
    centre(&mut p, milieu);
    let droite = charte::panneau(p.c, milieu, colonne(LARGEUR_ARENE));
    arenes::construire(&mut p, dj, sauvegarde, choix, droite);
    bas::construire(&mut p, dj, sauvegarde, choix, racine);
    racine
}

/// Bandeau du haut : logo à gauche, bouton du profil (visage, pseudo) et dé à droite.
fn bandeau(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    racine: Entity,
) {
    let b = p.noeud(
        racine,
        Node {
            height: Val::Px(HAUT),
            flex_shrink: 0.0,
            padding: UiRect::horizontal(Val::Px(MARGE)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
    );
    let logo = p.noeud(
        b,
        Node {
            column_gap: Val::Px(16.0),
            ..default()
        },
    );
    p.c.spawn((TextePixel::nouveau("Block", IVOIRE, 4.0), ChildOf(logo)));
    p.c.spawn((
        TextePixel::nouveau("Survivors", charte::OR_VIF, 4.0),
        ChildOf(logo),
    ));
    let droite = p.noeud(
        b,
        Node {
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            ..default()
        },
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

/// Colonne du héros (dessiné dessous par la caméra 3D) : ombre au sol, « Apparence »
/// sous ses pieds.
fn centre(p: &mut Pinceau, milieu: Entity) {
    let c = p.noeud(
        milieu,
        Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::top(Val::Px(PLACE_HEROS)),
            ..default()
        },
    );
    let sol = p.noeud(
        c,
        Node {
            width: Val::Px(220.0),
            height: Val::Px(40.0),
            margin: UiRect::left(Val::Px(-110.0)),
            ..place(
                Some(Val::Percent(50.0)),
                None,
                Some(Val::Px(PLACE_HEROS - 44.0)),
                None,
            )
        },
    );
    let ombre = RadialGradient::new(
        UiPosition::CENTER,
        RadialGradientShape::FarthestSide,
        vec![
            ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.55), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(100.0)),
        ],
    );
    p.c.entity(sol).insert(BackgroundGradient::from(ombre));
    p.bouton(c, "Apparence", Relief::SECONDAIRE, ActionMenu::OuvrirProfil);
}
