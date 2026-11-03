//! Charte commune des pages (docs/CHARTE-UI.md) : surfaces charbon, texte ivoire, or vif
//! réservé au focus et aux récompenses, vert pour l'action principale. Les boutons en
//! relief (`Relief`) ont un bord de 2, une tranche basse plus sombre de 3 et un contour
//! ivoire au survol ; enfoncés, leur tranche s'aplatit et leur contenu descend de 2.

use bevy::prelude::*;

const fn hex(c: u32) -> Color {
    Color::srgb_u8((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

/// Fond profond (derrière tout).
pub const FOND: Color = hex(0x101418);
/// Panneau posé sur le décor (opacité 92 %).
pub const PANNEAU: Color = Color::srgba(
    0x17 as f32 / 255.0,
    0x1c as f32 / 255.0,
    0x20 as f32 / 255.0,
    0.92,
);
/// Bouton secondaire et ligne de liste.
pub const SURFACE: Color = hex(0x232a2e);
pub const SURFACE_SURVOL: Color = hex(0x303a3e);
/// Emplacement fermé.
pub const CREUX: Color = hex(0x202529);
pub const BORD: Color = hex(0x485156);
/// Noms, valeurs, instructions.
pub const IVOIRE: Color = hex(0xf1ebdd);
/// Aides et métadonnées.
pub const AIDE: Color = hex(0xbec5c1);
/// Focus et récompenses.
pub const OR_VIF: Color = hex(0xf2c65b);
/// Décor (titres de section, cadres).
pub const OR_SOURD: Color = hex(0x907640);
/// Ligne choisie.
pub const CHOISI: Color = hex(0x373323);
pub const CHOISI_SURVOL: Color = hex(0x433d29);
/// Action principale : face, survol, tranche.
pub const VERT: Color = hex(0x337a48);
pub const VERT_SURVOL: Color = hex(0x419356);
pub const VERT_TRANCHE: Color = hex(0x205333);
/// Gain, prérequis rempli.
pub const VERT_GAIN: Color = hex(0x67bb78);
/// Perte, danger.
pub const ROUGE: Color = hex(0xed7777);
/// Tranche des boutons secondaires.
pub const TRANCHE: Color = hex(0x161b1e);

/// Tailles de texte (base 720p).
pub const T_AIDE: f32 = 15.0;
pub const T_CORPS: f32 = 17.0;
pub const T_SECTION: f32 = 22.0;
/// Hauteurs des boutons.
pub const H_SECONDAIRE: f32 = 44.0;
pub const H_PRINCIPAL: f32 = 52.0;
/// Marge au bord de l'écran.
pub const MARGE: f32 = 32.0;

/// Couleurs d'un bouton en relief : au repos et au survol.
#[derive(Component, Clone, Copy)]
pub struct Relief {
    pub fond: Color,
    pub bord: Color,
    pub fond_survol: Color,
    pub bord_survol: Color,
    pub tranche: Color,
}

impl Relief {
    /// Bouton secondaire.
    pub const SECONDAIRE: Self = Self {
        fond: SURFACE,
        bord: BORD,
        fond_survol: SURFACE_SURVOL,
        bord_survol: IVOIRE,
        tranche: TRANCHE,
    };
    /// Action principale verte.
    pub const PRINCIPAL: Self = Self {
        fond: VERT,
        bord: VERT_TRANCHE,
        fond_survol: VERT_SURVOL,
        bord_survol: IVOIRE,
        tranche: VERT_TRANCHE,
    };
    /// Ligne choisie (le liseré d'or est un enfant à part).
    pub const CHOISI: Self = Self {
        fond: CHOISI,
        bord: OR_SOURD,
        fond_survol: CHOISI_SURVOL,
        bord_survol: IVOIRE,
        tranche: TRANCHE,
    };

    fn bordure(&self, survol: bool) -> BorderColor {
        let c = if survol { self.bord_survol } else { self.bord };
        BorderColor {
            top: c,
            left: c,
            right: c,
            bottom: if survol { c } else { self.tranche },
        }
    }

    /// Composants d'un nœud au repos (avec son bord).
    pub fn composants(self) -> (Self, BorderColor, BackgroundColor) {
        (self, self.bordure(false), BackgroundColor(self.fond))
    }
}

/// Bord d'un bouton en relief : 2, tranche basse de 3 (posée par-dessus le bord).
pub fn bord_relief() -> UiRect {
    UiRect {
        left: Val::Px(2.0),
        right: Val::Px(2.0),
        top: Val::Px(2.0),
        bottom: Val::Px(4.0),
    }
}

/// Survol et appui des boutons en relief.
pub fn survoler_relief(
    mut boutons: Query<
        (
            &Interaction,
            &Relief,
            &mut BorderColor,
            &mut BackgroundColor,
            &mut Node,
        ),
        Changed<Interaction>,
    >,
) {
    for (i, r, mut bord, mut fond, mut n) in &mut boutons {
        let survol = *i != Interaction::None;
        *bord = r.bordure(survol);
        fond.0 = if survol { r.fond_survol } else { r.fond };
        // Enfoncé : la tranche s'aplatit, le contenu descend de 2
        let enfonce = *i == Interaction::Pressed;
        let (haut, bas) = if enfonce { (4.0, 2.0) } else { (2.0, 4.0) };
        if n.border.top != Val::Px(haut) {
            n.border.top = Val::Px(haut);
            n.border.bottom = Val::Px(bas);
        }
    }
}

/// Bouton en relief vide (contenu centré), de hauteur `hauteur`.
pub fn bouton_relief(
    commandes: &mut Commands,
    parent: Entity,
    relief: Relief,
    hauteur: f32,
    marque: impl Bundle,
) -> Entity {
    commandes
        .spawn((
            Button,
            marque,
            Node {
                height: Val::Px(hauteur),
                padding: UiRect::horizontal(Val::Px(16.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                border: bord_relief(),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            relief.composants(),
            ChildOf(parent),
        ))
        .id()
}

/// Panneau charbon posé sur le décor : bord de 2, coins de 2, marge intérieure de 16.
pub fn panneau(commandes: &mut Commands, parent: Entity, node: Node) -> Entity {
    commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..node
            },
            BackgroundColor(PANNEAU),
            BorderColor::all(BORD),
            ChildOf(parent),
        ))
        .id()
}

/// Cadenas dessiné (anse et corps), de `cote` px de large.
pub fn cadenas(commandes: &mut Commands, parent: Entity, cote: f32, couleur: Color) -> Entity {
    let u = cote / 8.0;
    let c = commandes
        .spawn((
            Node {
                width: Val::Px(cote),
                height: Val::Px(cote * 1.125),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    commandes.spawn((
        Node {
            width: Val::Px(u * 5.0),
            height: Val::Px(u * 4.0),
            border: UiRect {
                left: Val::Px(u),
                right: Val::Px(u),
                top: Val::Px(u),
                bottom: Val::ZERO,
            },
            border_radius: BorderRadius::top(Val::Px(u * 2.5)),
            ..default()
        },
        BorderColor::all(couleur),
        ChildOf(c),
    ));
    commandes.spawn((
        Node {
            width: Val::Px(cote),
            height: Val::Px(u * 5.0),
            border_radius: BorderRadius::all(Val::Px(u * 0.5)),
            ..default()
        },
        BackgroundColor(couleur),
        ChildOf(c),
    ));
    c
}
