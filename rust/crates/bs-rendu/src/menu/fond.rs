//! Fond de l'accueil (styles.css `#menu-fond`, `.menu-voile`) : la photo de l'arène
//! choisie en plein écran, recadrée comme `background-size: cover`, qui apparaît en
//! fondu à chaque changement d'arène, sous un voile qui assombrit les bords. Elle est
//! dessinée par une caméra 2D sous celle du héros (ordre −1), sur sa propre couche.
//! `Couvrir` recadre de même une image d'interface (vignettes des arènes).

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{ElementMenu, Menu, Page};
use crate::chargement::{Photos, photo};
use crate::lancement::DonneesJeu;

/// Couche de rendu de la photo (les autres caméras ne la voient pas).
const COUCHE: usize = 1;
/// Durée du fondu d'apparition (s).
const FONDU: f32 = 0.4;

/// La photo de l'arène, en sprite.
#[derive(Component)]
pub struct PhotoFond;

/// Image d'interface recadrée pour remplir son nœud sans se déformer.
#[derive(Component)]
pub struct Couvrir;

/// Caméra du fond, photo (cachée jusqu'au premier `suivre`) et voile.
pub fn creer(commandes: &mut Commands, fond: Color) {
    let camera = commandes
        .spawn((
            Name::new("caméra du fond du menu"),
            Camera2d,
            Camera {
                order: -1,
                clear_color: ClearColorConfig::Custom(fond),
                ..default()
            },
            RenderLayers::layer(COUCHE),
            ElementMenu,
        ))
        .id();
    commandes.spawn((
        Name::new("photo du menu"),
        Sprite::default(),
        Visibility::Hidden,
        RenderLayers::layer(COUCHE),
        PhotoFond,
        ElementMenu,
    ));
    let sombre = |a: f32| Color::srgba_u8(6, 8, 14, (a * 255.0) as u8);
    let arret = |a: f32, p: f32| ColorStop::new(sombre(a), Val::Percent(p));
    let cotes = LinearGradient::to_right(vec![
        arret(0.92, 0.0),
        arret(0.55, 26.0),
        arret(0.05, 40.0),
        arret(0.05, 60.0),
        arret(0.55, 74.0),
        arret(0.92, 100.0),
    ]);
    let haut_bas = LinearGradient::to_bottom(vec![
        arret(0.35, 0.0),
        arret(0.0, 18.0),
        arret(0.0, 60.0),
        arret(0.85, 100.0),
    ]);
    commandes.spawn((
        Name::new("voile du menu"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundGradient(vec![cotes.into(), haut_bas.into()]),
        UiTargetCamera(camera),
        ElementMenu,
    ));
}

/// Rectangle de l'image (`taille_image`) qui remplit `cible` sans se déformer, centré.
fn recadrage(taille_image: Vec2, cible: Vec2) -> Rect {
    let (l, h) = (taille_image.x, taille_image.y);
    let r = cible.x / cible.y;
    if l / h > r {
        let lr = h * r;
        Rect::new((l - lr) / 2.0, 0.0, (l + lr) / 2.0, h)
    } else {
        let hr = l / r;
        Rect::new(0.0, (h - hr) / 2.0, l, (h + hr) / 2.0)
    }
}

/// Montre la photo de l'arène choisie sur l'accueil, à la taille de la fenêtre.
pub fn suivre(
    menu: Res<Menu>,
    dj: Res<DonneesJeu>,
    (mut photos, mut images): (ResMut<Photos>, ResMut<Assets<Image>>),
    temps: Res<Time>,
    fenetres: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&mut Sprite, &mut Visibility), With<PhotoFond>>,
) {
    let Ok((mut sprite, mut visible)) = q.single_mut() else {
        return;
    };
    let voulue = (menu.page == Page::Accueil)
        .then(|| {
            photo(
                &mut images,
                &mut photos,
                &dj.arenes[menu.choix.arene].def.id,
            )
        })
        .flatten();
    let Some(voulue) = voulue else {
        visible.set_if_neq(Visibility::Hidden);
        return;
    };
    visible.set_if_neq(Visibility::Visible);
    if sprite.image != voulue {
        sprite.image = voulue;
        sprite.color = Color::WHITE.with_alpha(0.0);
    }
    let alpha = sprite.color.alpha();
    if alpha < 1.0 {
        let a = (alpha + temps.delta_secs() / FONDU).min(1.0);
        sprite.color.set_alpha(a);
    }
    let (Ok(f), Some(image)) = (fenetres.single(), images.get(&sprite.image)) else {
        return;
    };
    let ecran = Vec2::new(f.width(), f.height());
    let taille = image.size_f32();
    let echelle = (ecran.x / taille.x).max(ecran.y / taille.y);
    let voulue = Some(taille * echelle);
    if sprite.custom_size != voulue {
        sprite.custom_size = voulue;
    }
}

/// Recadre les images `Couvrir` à la taille de leur nœud.
pub fn couvrir(
    images: Res<Assets<Image>>,
    mut q: Query<(&ComputedNode, &mut ImageNode), With<Couvrir>>,
) {
    for (n, mut i) in &mut q {
        let cible = n.size();
        if cible.x <= 0.0 || cible.y <= 0.0 {
            continue;
        }
        let Some(image) = images.get(&i.image) else {
            continue;
        };
        let r = Some(recadrage(image.size_f32(), cible));
        if i.rect != r {
            i.rect = r;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recadrage_centre() {
        // Photo 16:9 dans une vignette très large : bande horizontale au milieu
        let r = recadrage(Vec2::new(1920.0, 1080.0), Vec2::new(300.0, 100.0));
        assert_eq!((r.min.x, r.max.x), (0.0, 1920.0));
        assert!((r.height() - 640.0).abs() < 0.01 && (r.min.y - 220.0).abs() < 0.01);
        // Dans un carré : colonne centrale
        let r = recadrage(Vec2::new(1920.0, 1080.0), Vec2::splat(50.0));
        assert!((r.width() - 1080.0).abs() < 0.01 && (r.min.x - 420.0).abs() < 0.01);
    }
}
