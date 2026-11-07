//! Écran de chargement (src/ui/ecranChargement.ts), pendant la phase `Chargement` : la
//! photo de l'arène en plein écran (celles fournies avec le jeu, embarquées), son nom, une
//! astuce au hasard et une barre qui ne recule jamais. Tout disparaît à la sortie de la
//! phase. Les photos sont décodées une fois puis gardées. Dégradé sombre vers le bas,
//! textes groupés au-dessus de la barre, étape et pourcentage sur une ligne
//! (docs/CHARTE-UI.md).

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;

use crate::carte_mc::Import;
use crate::ecrans::charte::{self, AIDE, IVOIRE, OR_SOURD, OR_VIF};
use crate::ecrans::style;
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

/// Pourcentage, à droite de l'étape.
#[derive(Component)]
pub(crate) struct Pourcent;

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
    let h = decoder(images, octets, id)?;
    photos.0.insert(cle, h.clone());
    Some(h)
}

/// Décode une photo WebP embarquée.
pub(crate) fn decoder(
    images: &mut Assets<Image>,
    octets: &[u8],
    id: &str,
) -> Option<Handle<Image>> {
    let lue = image::load_from_memory_with_format(octets, image::ImageFormat::WebP);
    let image = match lue {
        Ok(i) => Image::from_dynamic(i, true, RenderAssetUsages::RENDER_WORLD),
        Err(e) => {
            warn!("photo de l'arène {id} illisible : {e}");
            return None;
        }
    };
    Some(images.add(image))
}

/// Entrée en chargement : caméra, photo, nom de l'arène, astuce, barre vide.
pub fn entrer(
    mut commandes: Commands,
    dj: Res<DonneesJeu>,
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
        ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(30.0)),
        ColorStop::new(Color::srgba(0.04, 0.05, 0.06, 0.75), Val::Percent(65.0)),
        ColorStop::new(Color::srgba(0.04, 0.05, 0.06, 0.95), Val::Percent(100.0)),
    ];
    commandes.spawn((
        style::plein(),
        BackgroundGradient::from(LinearGradient::to_bottom(stops)),
        ChildOf(racine),
    ));
    // Groupe en bas : nom, astuce, étape et pourcentage, barre
    let bas = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-LARGEUR_BARRE / 2.0)),
                width: Val::Px(LARGEUR_BARRE),
                bottom: Val::Px(48.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    if let Some(a) = arene {
        commandes.spawn((TextePixel::nouveau(&a.nom, IVOIRE, 4.0), ChildOf(bas)));
    }
    let astuces = &dj.d.textes.astuces;
    if !astuces.is_empty() {
        let k = crate::lancement::graine_au_hasard() as usize % astuces.len();
        let bloc = commandes
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(8.0), Val::Px(8.0)),
                    border: UiRect::left(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.063, 0.078, 0.094, 0.7)),
                BorderColor::all(OR_SOURD),
                ChildOf(bas),
            ))
            .id();
        commandes.spawn((TextePixel::nouveau("Astuce", OR_VIF, 1.5), ChildOf(bloc)));
        commandes.spawn((
            TextePixel::nouveau(&astuces[k], IVOIRE, 2.0)
                .bloc(LARGEUR_BARRE - 30.0)
                .a_gauche(),
            ChildOf(bloc),
        ));
    }
    // Étape à gauche, pourcentage à droite, sur une ligne
    let ligne = commandes
        .spawn((
            Node {
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
            ChildOf(bas),
        ))
        .id();
    commandes.spawn((
        Libelle,
        TextePixel::nouveau("Chargement…", AIDE, 1.5),
        ChildOf(ligne),
    ));
    commandes.spawn((
        Pourcent,
        TextePixel::nouveau("0 %", IVOIRE, 1.5),
        ChildOf(ligne),
    ));
    let cadre = commandes
        .spawn((
            Node {
                height: Val::Px(14.0),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                margin: UiRect::top(Val::Px(-6.0)),
                ..default()
            },
            BackgroundColor(charte::CREUX),
            BorderColor::all(charte::BORD),
            ChildOf(bas),
        ))
        .id();
    commandes.spawn((
        Remplissage,
        Node {
            width: Val::Percent(0.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(charte::VERT_GAIN),
        ChildOf(cadre),
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
    mut libelles: Query<(&mut TextePixel, Has<Pourcent>), Or<(With<Libelle>, With<Pourcent>)>>,
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
    for (mut t, pourcentage) in &mut libelles {
        if pourcentage {
            ecrire(&mut t, &format!("{pourcent} %"));
        } else {
            ecrire(&mut t, etape);
        }
    }
}
