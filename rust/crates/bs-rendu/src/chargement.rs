//! Écran de chargement (src/ui/ecranChargement.ts), pendant la phase `Chargement` : la
//! photo de l'arène en plein écran (celles fournies avec le jeu, embarquées), son nom, une
//! astuce au hasard et une barre qui ne recule jamais. Tout disparaît à la sortie de la
//! phase. Les photos sont décodées une fois puis gardées.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;

use crate::carte_mc::Import;
use crate::dessins::Dessins;
use crate::ecrans::style::{self, GRIS, OR, OR_CLAIR, TEXTE};
use crate::hud::police::{TextePixel, ecrire};
use crate::lancement::DonneesJeu;
use crate::textures::Textures;
use crate::{PartieEnCours, Phase};

/// Photos fournies avec le jeu, par arène (src/assets/chargement).
const PHOTOS: [(&str, &[u8]); 4] = [
    (
        "prairie",
        include_bytes!("../../../../src/assets/chargement/prairie.webp"),
    ),
    (
        "mine",
        include_bytes!("../../../../src/assets/chargement/mine.webp"),
    ),
    (
        "nether",
        include_bytes!("../../../../src/assets/chargement/nether.webp"),
    ),
    (
        "map_mushroom",
        include_bytes!("../../../../src/assets/chargement/map_mushroom.webp"),
    ),
];
const LARGEUR_BARRE: f32 = 600.0;

/// Photos déjà décodées, par arène.
#[derive(Resource, Default)]
pub struct Photos(HashMap<&'static str, Handle<Image>>);

impl Photos {
    /// Photo de l'arène si elle est déjà décodée.
    pub fn decodee(&self, id: &str) -> Option<Handle<Image>> {
        self.0.get(id).cloned()
    }
}

/// Barre (largeur suivie) et libellé de l'avancement.
#[derive(Component)]
pub(crate) struct Remplissage;

#[derive(Component)]
pub(crate) struct Libelle;

/// Part déjà remplie (0 à 1) : la barre ne recule pas quand des textures s'ajoutent.
#[derive(Resource, Default)]
pub struct Avance(f32);

pub(crate) fn photo(
    images: &mut Assets<Image>,
    photos: &mut Photos,
    id: &str,
) -> Option<Handle<Image>> {
    let (cle, octets) = PHOTOS.iter().find(|(p, _)| *p == id)?;
    if let Some(h) = photos.0.get(cle) {
        return Some(h.clone());
    }
    let lue = image::load_from_memory_with_format(octets, image::ImageFormat::WebP);
    let image = match lue {
        Ok(i) => Image::from_dynamic(i, true, RenderAssetUsages::RENDER_WORLD),
        Err(e) => {
            warn!("photo de l'arène {id} illisible : {e}");
            return None;
        }
    };
    let h = images.add(image);
    photos.0.insert(cle, h.clone());
    Some(h)
}

/// Entrée en chargement : caméra, photo, nom de l'arène, astuce, barre vide.
pub fn entrer(
    mut commandes: Commands,
    dj: Res<DonneesJeu>,
    dessins: Res<Dessins>,
    jeu: Option<Res<PartieEnCours>>,
    import: Option<Res<Import>>,
    mut images: ResMut<Assets<Image>>,
    mut photos: ResMut<Photos>,
) {
    commandes.insert_resource(Avance::default());
    let fin = DespawnOnExit(Phase::Chargement);
    commandes.spawn((Name::new("caméra du chargement"), Camera2d, fin.clone()));
    let arene = match (&jeu, &import) {
        (Some(j), _) => Some(j.partie.etat.arene),
        (None, Some(i)) => i.arene(),
        _ => None,
    };
    let racine = commandes
        .spawn((
            Name::new("chargement"),
            style::plein(),
            BackgroundColor(Color::srgb_u8(0x0b, 0x0d, 0x12)),
            fin,
        ))
        .id();
    if let Some(h) = arene.and_then(|a| photo(&mut images, &mut photos, &a.id)) {
        commandes.spawn((ImageNode::new(h), style::plein(), ChildOf(racine)));
    }
    // Voile dégradé vers le bas, sous les textes
    let stops = vec![
        ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(45.0)),
        ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.85), Val::Percent(100.0)),
    ];
    commandes.spawn((
        style::plein(),
        BackgroundGradient::from(LinearGradient::to_bottom(stops)),
        ChildOf(racine),
    ));
    let bas = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                right: Val::ZERO,
                bottom: Val::Px(48.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    if let Some(a) = arene {
        commandes.spawn((TextePixel::nouveau(&a.nom, OR_CLAIR, 4.0), ChildOf(bas)));
    }
    let astuces = &dj.d.textes.astuces;
    if !astuces.is_empty() {
        let k = crate::lancement::graine_au_hasard() as usize % astuces.len();
        let t = format!("Astuce : {}", astuces[k]);
        commandes.spawn((
            TextePixel::nouveau(&t, TEXTE, 2.0).bloc(760.0),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
            ChildOf(bas),
        ));
    }
    let cadre = commandes
        .spawn((
            Node {
                width: Val::Px(LARGEUR_BARRE),
                height: Val::Px(18.0),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            ChildOf(bas),
        ))
        .id();
    style::orner(&mut commandes, &dessins, cadre);
    commandes.spawn((
        Remplissage,
        Node {
            width: Val::Percent(0.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(OR),
        ChildOf(cadre),
    ));
    commandes.spawn((
        Libelle,
        TextePixel::nouveau("Chargement… 0 %", GRIS, 1.5),
        ChildOf(bas),
    ));
}

/// Barre et libellé suivent le chargement des textures (et la construction d'une carte
/// importée avant elles).
pub fn avancer(
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    import: Option<Res<Import>>,
    mut avance: ResMut<Avance>,
    mut barre: Query<&mut Node, With<Remplissage>>,
    mut libelle: Query<&mut TextePixel, With<Libelle>>,
) {
    let a = textures.avancement(&serveur);
    let (etape, part) = match import.is_some() {
        true => ("Construction de la carte…", 0.0),
        false if a.total == 0 => ("Chargement…", 0.0),
        false => ("Chargement des textures…", a.faites as f32 / a.total as f32),
    };
    avance.0 = avance.0.max(part.min(1.0));
    let pourcent = (avance.0 * 100.0).round();
    if let Ok(mut n) = barre.single_mut()
        && n.width != Val::Percent(pourcent)
    {
        n.width = Val::Percent(pourcent);
    }
    if let Ok(mut t) = libelle.single_mut() {
        ecrire(&mut t, &format!("{etape} {pourcent} %"));
    }
}
