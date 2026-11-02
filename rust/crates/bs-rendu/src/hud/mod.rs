//! HUD de la partie (src/ui/hud.ts) en interface Bevy : chrono et prochain boss, barre
//! du boss, émeraudes et ennemis vaincus, cœurs et PV, totems, expérience et niveau,
//! armes, passifs et équipement, alertes et messages, voile rouge de blessure, compteur
//! de dégâts. Mise en page de la fenêtre de référence 1280 × 820 (src/ui/echelle.ts),
//! agrandie par `UiScale` ; chaque élément n'est retouché que quand sa valeur affichée
//! change.

mod affichage;
mod barres;
mod capacites;
mod coeurs;
pub mod compteur;
pub mod etiquette;
pub mod police;

pub use affichage::{afficher, alerter, recevoir};

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;

use crate::PartieEnCours;
use crate::textures::Textures;
use police::{Police, TextePixel};

/// Couleurs du jeu (variables --mc-* de styles.css).
pub const BLANC: Color = Color::WHITE;
pub const GRIS: Color = Color::srgb(0.667, 0.667, 0.667);
pub const JAUNE: Color = Color::srgb(1.0, 1.0, 0.333);
pub const OR: Color = Color::srgb(1.0, 0.667, 0.0);
pub const VERT: Color = Color::srgb(0.333, 1.0, 0.333);
pub const ROUGE: Color = Color::srgb(1.0, 0.333, 0.333);

const LARGEUR_REFERENCE: f32 = 1280.0;
const HAUTEUR_REFERENCE: f32 = 820.0;
const ICONE_TUES: &str = "item/bone";
const ICONE_TOTEM: &str = "item/totem_of_undying";
/// Le chrono passe en or dans la dernière minute (s).
const DERNIER_COMPTE: i64 = 60;
const DUREE_ANNONCE_PROLONGATION: f32 = 4.0;
/// Voile rouge : après un coup, et pulsant quand la santé est basse.
const VOILE_BLESSURE: f32 = 0.85;
const VOILE_SANTE_BASSE: f32 = 0.5;
const SEUIL_SANTE_BASSE: f32 = 0.3;
const PULSATION: f32 = 5.0;
/// Totem qui jaillit à la résurrection (s).
const DUREE_TOTEM: f32 = 1.6;

pub struct PluginHud;

impl Plugin for PluginHud {
    fn build(&self, app: &mut App) {
        app.init_resource::<Police>()
            .insert_resource(UiScale(1.0))
            .add_systems(Update, (police::charger, echelle))
            .add_systems(
                PostUpdate,
                police::dessiner.before(bevy::ui::UiSystems::Layout),
            );
    }
}

/// Entités du HUD et minuteries des bandeaux.
#[derive(Resource)]
pub struct Hud {
    /// Racine (masquée sous l'écran de fin).
    pub racine: Entity,
    chrono: Entity,
    prochain_boss: Entity,
    emeraudes: Entity,
    tues: Entity,
    message: Entity,
    alerte: Entity,
    alerte_texte: Entity,
    voile: Entity,
    totems: Entity,
    totems_nombre: Entity,
    totem_anim: Entity,
    coeurs: coeurs::Coeurs,
    barres: barres::Barres,
    capacites: capacites::Capacites,
    compteur: compteur::Compteur,
    minuterie_message: f32,
    minuterie_alerte: f32,
    temps: f32,
    animation_totem: f32,
    vies_vues: u32,
    prolongation_vue: bool,
}

/// Textures du HUD (avant l'entrée en partie).
pub fn demander_textures(
    serveur: Res<AssetServer>,
    mut textures: ResMut<Textures>,
    jeu: Res<PartieEnCours>,
    mut fait: Local<bool>,
) {
    if std::mem::replace(&mut *fait, true) {
        return;
    }
    let d = jeu.partie.d;
    let mut demander = |t: &str| textures.demander(&serveur, t);
    for t in [ICONE_TUES, ICONE_TOTEM] {
        demander(t);
    }
    if let Some(o) = d.objets.get("emeraude") {
        demander(&o.icone);
    }
    coeurs::demander_textures(&mut demander);
    barres::demander_textures(d, &mut demander);
    capacites::demander_textures(d, &mut demander);
    crate::ecran_cartes::demander_textures(d, &mut demander);
    crate::ecrans::demander_textures(d, &mut demander);
}

/// Taille de l'interface : la fenêtre de référence tient dans la fenêtre, par quarts
/// au-delà de 1.
fn echelle(fenetre: Query<&Window, With<PrimaryWindow>>, mut ui: ResMut<UiScale>) {
    let Ok(f) = fenetre.single() else {
        return;
    };
    let brute = (f.width() / LARGEUR_REFERENCE).min(f.height() / HAUTEUR_REFERENCE);
    let e = if brute >= 1.0 {
        (brute * 4.0).floor() / 4.0
    } else {
        brute.max(0.5)
    };
    if ui.0 != e {
        ui.0 = e;
    }
}

/// Image d'un sprite, ou couleur unie s'il manque.
fn image_ou(h: &Option<Handle<Image>>, secours: Color) -> ImageNode {
    match h {
        Some(h) => ImageNode::new(h.clone()),
        None => ImageNode::solid_color(secours),
    }
}

/// Nœud placé en absolu.
pub fn absolu() -> Node {
    Node {
        position_type: PositionType::Absolute,
        ..default()
    }
}

/// Image d'une texture du jeu (ou rien si elle manque) à la taille donnée.
pub fn icone(textures: &Textures, serveur: &AssetServer, chemin: &str, taille: f32) -> impl Bundle {
    let image = textures
        .obtenir(serveur, chemin)
        .map(ImageNode::new)
        .unwrap_or_else(|| ImageNode::default().with_color(Color::NONE));
    (
        image,
        Node {
            width: Val::Px(taille),
            height: Val::Px(taille),
            ..default()
        },
    )
}

/// Construit le HUD à l'entrée en partie.
pub fn construire(
    mut commandes: Commands,
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    mut images: ResMut<Assets<Image>>,
    jeu: Res<PartieEnCours>,
) {
    let d = jeu.partie.d;
    let racine = commandes
        .spawn((
            Name::new("hud"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .id();
    let fils = |b: &mut EntityCommands| {
        b.insert(ChildOf(racine));
    };
    let creer = |commandes: &mut Commands, b: Node| {
        let mut e = commandes.spawn(b);
        fils(&mut e);
        e.id()
    };
    let voile = creer(
        &mut commandes,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..absolu()
        },
    );
    commandes.entity(voile).insert(
        ImageNode::new(images.add(image_voile())).with_color(Color::srgba(1.0, 1.0, 1.0, 0.0)),
    );

    // Haut : chrono, prochain boss, barre du boss
    let haut = creer(
        &mut commandes,
        Node {
            top: Val::Px(8.0),
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(6.0),
            ..absolu()
        },
    );
    let chrono = commandes
        .spawn((TextePixel::nouveau("00:00", BLANC, 3.0), ChildOf(haut)))
        .id();
    let prochain_boss = commandes
        .spawn((
            TextePixel::nouveau("", GRIS, 2.0),
            Node {
                margin: UiRect::top(Val::Px(-4.0)),
                ..default()
            },
            ChildOf(haut),
        ))
        .id();
    let barres = barres::construire(&mut commandes, &serveur, &textures, d, racine, haut);
    let coeurs = coeurs::construire(&mut commandes, &serveur, &textures, racine);

    // Droite : émeraudes et ennemis vaincus
    let droite = creer(
        &mut commandes,
        Node {
            top: Val::Px(10.0),
            right: Val::Px(12.0),
            column_gap: Val::Px(16.0),
            ..absolu()
        },
    );
    let mut compteur = |icone_chemin: &str, couleur: Color| {
        let boite = commandes
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                ChildOf(droite),
            ))
            .id();
        commandes.spawn((
            icone(&textures, &serveur, icone_chemin, 32.0),
            ChildOf(boite),
        ));
        commandes
            .spawn((TextePixel::nouveau("0", couleur, 2.0), ChildOf(boite)))
            .id()
    };
    let icone_emeraude = d.objets.get("emeraude").map_or("", |o| o.icone.as_str());
    let emeraudes = compteur(icone_emeraude, VERT);
    let tues = compteur(ICONE_TUES, BLANC);

    // Message (jaune) et alerte (bandeau rouge clignotant), centrés
    let ligne = |commandes: &mut Commands, haut: f32| {
        commandes
            .spawn((
                Node {
                    top: Val::Percent(haut),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..absolu()
                },
                ChildOf(racine),
            ))
            .id()
    };
    let ligne_message = ligne(&mut commandes, 24.0);
    let message = commandes
        .spawn((TextePixel::nouveau("", JAUNE, 2.0), ChildOf(ligne_message)))
        .id();
    let alerte = ligne(&mut commandes, 18.0);
    commandes.entity(alerte).insert(Visibility::Hidden);
    let bandeau = commandes
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(22.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.157, 0.0, 0.0, 0.55)),
            BorderColor::all(ROUGE),
            ChildOf(alerte),
        ))
        .id();
    let alerte_texte = commandes
        .spawn((TextePixel::nouveau("", ROUGE, 3.0), ChildOf(bandeau)))
        .id();

    // Totems restants, à gauche des cœurs, et totem qui jaillit à la résurrection
    let totems = creer(
        &mut commandes,
        Node {
            bottom: Val::Px(72.0),
            right: Val::Percent(50.0),
            margin: UiRect::right(Val::Px(172.0)),
            padding: UiRect::new(Val::Px(2.0), Val::Px(6.0), Val::ZERO, Val::ZERO),
            align_items: AlignItems::Center,
            column_gap: Val::Px(2.0),
            ..absolu()
        },
    );
    commandes.entity(totems).insert((
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
        Visibility::Hidden,
    ));
    commandes.spawn((
        icone(&textures, &serveur, ICONE_TOTEM, 32.0),
        ChildOf(totems),
    ));
    let totems_nombre = commandes
        .spawn((TextePixel::nouveau("", BLANC, 2.0), ChildOf(totems)))
        .id();
    let totem_anim = creer(
        &mut commandes,
        Node {
            left: Val::Percent(50.0),
            top: Val::Percent(45.0),
            margin: UiRect::new(Val::Px(-96.0), Val::ZERO, Val::Px(-96.0), Val::ZERO),
            ..absolu()
        },
    );
    commandes.entity(totem_anim).insert((
        icone(&textures, &serveur, ICONE_TOTEM, 192.0),
        Visibility::Hidden,
    ));

    let capacites = capacites::construire(&mut commandes, racine);
    let compteur = compteur::construire(&mut commandes, racine);
    commandes.insert_resource(Hud {
        racine,
        chrono,
        prochain_boss,
        emeraudes,
        tues,
        message,
        alerte,
        alerte_texte,
        voile,
        totems,
        totems_nombre,
        totem_anim,
        coeurs,
        barres,
        capacites,
        compteur,
        minuterie_message: 0.0,
        minuterie_alerte: 0.0,
        temps: 0.0,
        animation_totem: -1.0,
        vies_vues: 0,
        prolongation_vue: false,
    });
}

/// Voile rouge : transparent au centre, rouge vers les bords (ellipse jusqu'aux coins).
fn image_voile() -> Image {
    const T: u32 = 64;
    let mut px = Vec::with_capacity((T * T * 4) as usize);
    for y in 0..T {
        for x in 0..T {
            let u = (x as f32 + 0.5) / T as f32 * 2.0 - 1.0;
            let v = (y as f32 + 0.5) / T as f32 * 2.0 - 1.0;
            let r = (u * u + v * v).sqrt() / std::f32::consts::SQRT_2;
            let a = ((r - 0.45) / 0.55).clamp(0.0, 1.0) * 0.55;
            px.extend_from_slice(&[200, 0, 0, (a * 255.0) as u8]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: T,
            height: T,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        px,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}

/// « 75 » → « 01:15 »
pub fn formater_temps(secondes: i64) -> String {
    let s = secondes.max(0);
    format!("{:02}:{:02}", s / 60, s % 60)
}

/// Nombre à une décimale au plus, à la française : « 17,4 », « 24 ».
pub fn formater_dixiemes(dixiemes: i64) -> String {
    if dixiemes % 10 == 0 {
        format!("{}", dixiemes / 10)
    } else {
        format!("{},{}", dixiemes / 10, (dixiemes % 10).abs())
    }
}
