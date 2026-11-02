//! Panneau des options (src/ui/panneauOptions.ts), par-dessus le menu ou la pause : un
//! curseur pour le volume général et un pour chaque famille de sons (clic ou glissé), des
//! bascules pour le reste, « Terminé » ou Échap pour fermer. Construit une fois à
//! l'ouverture ; les libellés et les curseurs suivent les options sans le refaire (un
//! curseur reste saisi pendant qu'on le glisse).

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use super::{CATEGORIES, Options};
use crate::ecrans::style::{self, GRIS, OR, OR_CLAIR};
use crate::hud::police::{TextePixel, ecrire};

const LARGEUR: f32 = 560.0;
const LARGEUR_CURSEUR: f32 = 256.0;
const LIBELLES_CATEGORIES: [&str; 4] = ["Joueur", "Armes", "Créatures", "Interface"];
const LIBELLES_OMBRES: [&str; 3] = ["NON", "OUI", "OUI (fines)"];

/// Panneau ouvert (sa racine) ; `ouvrir` le demande pour l'image suivante.
#[derive(Resource, Default)]
pub struct PanneauOptions {
    racine: Option<Entity>,
    demande: bool,
}

impl PanneauOptions {
    pub fn ouvrir(&mut self) {
        self.demande = true;
    }

    /// Ouvert (ou sur le point de l'être) : Échap et Entrée sont pour lui.
    pub fn ouvert(&self) -> bool {
        self.racine.is_some() || self.demande
    }
}

/// Réglage d'une ligne du panneau.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Reglage {
    /// 0 : général, puis les familles de sons.
    Volume(usize),
    Fps,
    Secousses,
    Ombres,
    Chiffres,
    Compteur,
    Vue,
    Beta,
    /// Effets des autres joueurs (coop).
    Allies,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum ActionOptions {
    Basculer(Reglage),
    Fermer,
}

/// Curseur de volume (sa barre suit la valeur).
#[derive(Component)]
pub(super) struct Curseur(usize);

#[derive(Component)]
pub(super) struct Barre(usize);

/// Texte d'un réglage.
#[derive(Component)]
pub(super) struct Libelle(Reglage);

fn libelle(o: &Options, r: Reglage) -> String {
    let oui = |b: bool| if b { "OUI" } else { "NON" };
    match r {
        Reglage::Volume(0) => match (o.volume * 100.0).round() as u32 {
            0 => "Volume général : désactivé".to_owned(),
            p => format!("Volume général : {p} %"),
        },
        Reglage::Volume(k) => match (o.volumes[k - 1] * 100.0).round() as u32 {
            0 => format!("{} : NON", LIBELLES_CATEGORIES[k - 1]),
            p => format!("{} : {p} %", LIBELLES_CATEGORIES[k - 1]),
        },
        Reglage::Fps => format!("Compteur de FPS : {}", oui(o.afficher_fps)),
        Reglage::Secousses => format!("Secousses de l'écran : {}", oui(o.secousses)),
        Reglage::Ombres => format!("Ombres et vent : {}", LIBELLES_OMBRES[o.ombres]),
        Reglage::Chiffres => format!("Chiffres des dégâts : {}", oui(o.chiffres_degats)),
        Reglage::Compteur => format!("Compteur (Tab) : {}", oui(o.compteur_degats)),
        Reglage::Vue => {
            let vue = if o.troisieme_personne {
                "3E PERSONNE"
            } else {
                "DE DESSUS"
            };
            format!("Vue (F5) : {vue}")
        }
        Reglage::Beta => format!("Versions bêta : {}", oui(o.beta)),
        Reglage::Allies => format!(
            "Effets des autres joueurs (multi) : {}",
            oui(o.effets_allies)
        ),
    }
}

fn volume(o: &Options, k: usize) -> f32 {
    if k == 0 { o.volume } else { o.volumes[k - 1] }
}

/// Ouvre (à la demande), ferme (Terminé, Échap) et applique les clics.
#[allow(clippy::type_complexity)]
pub(super) fn gerer(
    mut commandes: Commands,
    mut panneau: ResMut<PanneauOptions>,
    mut options: ResMut<Options>,
    touches: Res<ButtonInput<KeyCode>>,
    boutons: Query<(&Interaction, &ActionOptions), Changed<Interaction>>,
    curseurs: Query<(&Interaction, &Curseur, &RelativeCursorPosition)>,
) {
    if std::mem::take(&mut panneau.demande) {
        if let Some(r) = panneau.racine.take() {
            commandes.entity(r).despawn();
        }
        panneau.racine = Some(construire(&mut commandes, &options));
        return;
    }
    let Some(racine) = panneau.racine else {
        return;
    };
    let clic = boutons
        .iter()
        .find(|(i, _)| **i == Interaction::Pressed)
        .map(|(_, a)| *a);
    let echap = touches.just_pressed(KeyCode::Escape);
    match clic {
        _ if echap => {}
        Some(ActionOptions::Fermer) => {}
        Some(ActionOptions::Basculer(r)) => {
            let o = &mut *options;
            match r {
                Reglage::Fps => o.afficher_fps = !o.afficher_fps,
                Reglage::Secousses => o.secousses = !o.secousses,
                Reglage::Ombres => o.ombres = (o.ombres + 1) % LIBELLES_OMBRES.len(),
                Reglage::Chiffres => o.chiffres_degats = !o.chiffres_degats,
                Reglage::Compteur => o.compteur_degats = !o.compteur_degats,
                Reglage::Vue => o.troisieme_personne = !o.troisieme_personne,
                Reglage::Beta => o.beta = !o.beta,
                Reglage::Allies => o.effets_allies = !o.effets_allies,
                Reglage::Volume(_) => {}
            }
            return;
        }
        None => {
            // Curseurs : la valeur suit le pointeur tant qu'il est appuyé (pas de 5 %)
            for (i, c, position) in &curseurs {
                let Some(n) = position.normalized.filter(|_| *i == Interaction::Pressed) else {
                    continue;
                };
                let v = ((n.x + 0.5).clamp(0.0, 1.0) * 20.0).round() / 20.0;
                if volume(&options, c.0) != v {
                    match c.0 {
                        0 => options.volume = v,
                        k => options.volumes[k - 1] = v,
                    }
                }
            }
            return;
        }
    }
    commandes.entity(racine).despawn();
    panneau.racine = None;
}

/// Libellés et barres à jour des options.
pub(super) fn rafraichir(
    options: Res<Options>,
    mut libelles: Query<(&Libelle, &mut TextePixel)>,
    mut barres: Query<(&Barre, &mut Node)>,
) {
    if !options.is_changed() {
        return;
    }
    for (l, mut t) in &mut libelles {
        ecrire(&mut t, &libelle(&options, l.0));
    }
    for (b, mut n) in &mut barres {
        let largeur = Val::Percent(volume(&options, b.0) * 100.0);
        if n.width != largeur {
            n.width = largeur;
        }
    }
}

/// Ferme le panneau (changement de phase).
pub(super) fn fermer(mut commandes: Commands, mut panneau: ResMut<PanneauOptions>) {
    if let Some(r) = panneau.racine.take() {
        commandes.entity(r).despawn();
    }
    panneau.demande = false;
}

fn curseur(commandes: &mut Commands, parent: Entity, o: &Options, k: usize) {
    let c = commandes
        .spawn((
            Button,
            Curseur(k),
            RelativeCursorPosition::default(),
            Node {
                width: Val::Px(LARGEUR_CURSEUR),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x10, 0x10, 0x10)),
            BorderColor::all(Color::srgb_u8(0x55, 0x55, 0x55)),
            ChildOf(parent),
        ))
        .id();
    commandes.spawn((
        Barre(k),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            width: Val::Percent(volume(o, k) * 100.0),
            ..default()
        },
        BackgroundColor(Color::srgb_u8(0x5a, 0x4a, 0x20)),
        FocusPolicy::Pass,
        ChildOf(c),
    ));
    commandes.spawn((
        Libelle(Reglage::Volume(k)),
        TextePixel::nouveau(&libelle(o, Reglage::Volume(k)), Color::WHITE, 1.5).contour(),
        FocusPolicy::Pass,
        ChildOf(c),
    ));
}

fn bascule(commandes: &mut Commands, parent: Entity, o: &Options, r: Reglage) {
    let b = style::bouton(
        commandes,
        parent,
        "",
        LARGEUR_CURSEUR,
        false,
        ActionOptions::Basculer(r),
    );
    commandes.entity(b).despawn_related::<Children>();
    commandes.spawn((
        Libelle(r),
        TextePixel::nouveau(&libelle(o, r), Color::WHITE, 1.5),
        ChildOf(b),
    ));
}

fn grille(commandes: &mut Commands, parent: Entity) -> Entity {
    commandes
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::SpaceBetween,
                row_gap: Val::Px(6.0),
                ..default()
            },
            ChildOf(parent),
        ))
        .id()
}

fn construire(commandes: &mut Commands, o: &Options) -> Entity {
    let racine = commandes
        .spawn((
            Name::new("options"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            FocusPolicy::Block,
            GlobalZIndex(30),
        ))
        .id();
    let cadre = commandes
        .spawn((
            Node {
                width: Val::Px(LARGEUR),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x14, 0x17, 0x1f)),
            BorderColor::all(OR),
            ChildOf(racine),
        ))
        .id();
    style::titre(commandes, cadre, "Options", OR_CLAIR, 3.0);
    commandes.spawn((
        TextePixel::nouveau("Musique et sons", GRIS, 1.5),
        ChildOf(cadre),
    ));
    let general = grille(commandes, cadre);
    commandes
        .entity(general)
        .entry::<Node>()
        .and_modify(|mut n| n.justify_content = JustifyContent::Center);
    curseur(commandes, general, o, 0);
    let sons = grille(commandes, cadre);
    for k in 1..=CATEGORIES.len() {
        curseur(commandes, sons, o, k);
    }
    commandes.spawn((
        TextePixel::nouveau("Affichage", GRIS, 1.5),
        Node {
            margin: UiRect::top(Val::Px(6.0)),
            ..default()
        },
        ChildOf(cadre),
    ));
    let affichage = grille(commandes, cadre);
    for r in [
        Reglage::Fps,
        Reglage::Secousses,
        Reglage::Ombres,
        Reglage::Chiffres,
        Reglage::Compteur,
        Reglage::Vue,
        Reglage::Allies,
        Reglage::Beta,
    ] {
        bascule(commandes, affichage, o, r);
    }
    let fin = style::bouton(
        commandes,
        cadre,
        "Terminé",
        200.0,
        false,
        ActionOptions::Fermer,
    );
    commandes
        .entity(fin)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::top(Val::Px(8.0)));
    racine
}
