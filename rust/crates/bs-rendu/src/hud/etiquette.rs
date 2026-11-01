//! Étiquette du pseudo au-dessus du joueur (src/effets/etiquette.ts) : texte blanc en
//! police pixel sur un fond noir translucide, toujours lisible (par-dessus le décor),
//! cachée à la mort. Haute de 0,42 bloc à 0,35 bloc au-dessus de la tête : sa taille suit
//! le zoom, par pixels de police entiers.

use bevy::prelude::*;

use super::police::{Bord, TextePixel};
use crate::PartieEnCours;
use crate::camera::CameraJeu;
use crate::modeles::Rendus;

/// Hauteur du texte et marge au-dessus de la tête (blocs, config.ts `joueur.etiquette`).
const HAUTEUR_TEXTE: f32 = 0.42;
const MARGE: f32 = 0.35;
/// Lignes de pixels de l'étiquette par pixel de police (8 de police + 1 de fond).
const LIGNES: f32 = 9.0;

#[derive(Component)]
pub struct Etiquette {
    texte: Entity,
}

/// Crée l'étiquette (cachée tant qu'elle n'est pas placée ; aucune si le pseudo est vide).
pub fn creer(mut commandes: Commands, sauvegarde: Res<crate::sauvegarde::Sauvegarde>) {
    let pseudo = &sauvegarde.profil.pseudo;
    if pseudo.is_empty() {
        return;
    }
    let mut texte = TextePixel::nouveau(pseudo, Color::WHITE, 1.0);
    texte.bord = Bord::Aucun;
    let fond = commandes
        .spawn((
            Name::new("étiquette"),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
            Visibility::Hidden,
        ))
        .id();
    let texte = commandes.spawn((texte, ChildOf(fond))).id();
    commandes.entity(fond).insert(Etiquette { texte });
}

/// Place l'étiquette au-dessus de la tête du joueur, à l'image.
pub fn placer(
    jeu: Res<PartieEnCours>,
    rendus: Option<Res<Rendus>>,
    camera: Query<(&Camera, &GlobalTransform), With<CameraJeu>>,
    ui: Res<UiScale>,
    mut etiquettes: Query<(&Etiquette, &mut Node, &mut Visibility, &ComputedNode)>,
    mut textes: Query<&mut TextePixel>,
) {
    let (Some(rendus), Ok((camera, gt))) = (rendus, camera.single()) else {
        return;
    };
    for (e, mut noeud, mut visible, calcule) in &mut etiquettes {
        let r = &rendus.liste[0];
        let p = r.pose.position;
        let tete = Vec3::new(p.x, p.y + r.modele.hauteur + MARGE, p.z);
        let haut = tete + Vec3::Y * HAUTEUR_TEXTE;
        let ecran = (
            camera.world_to_viewport(gt, tete),
            camera.world_to_viewport(gt, haut),
        );
        let (Ok(bas), Ok(sommet)) = ecran else {
            visible.set_if_neq(Visibility::Hidden);
            continue;
        };
        let montre = !jeu.partie.etat.joueur.mort;
        visible.set_if_neq(if montre {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        // Pixels de police entiers, en pixels d'interface
        let taille = ((bas.y - sommet.y) / LIGNES / ui.0).round().max(1.0);
        if let Ok(mut t) = textes.get_mut(e.texte)
            && t.taille != taille
        {
            t.taille = taille;
        }
        let marge = Val::Px(taille);
        // Largeur mise en page (pixels d'interface), ou estimée avant la première
        let mesuree = calcule.size().x * calcule.inverse_scale_factor();
        let largeur = if mesuree > 0.0 {
            mesuree
        } else {
            largeur_estimee(&textes, e.texte, taille)
        };
        noeud.padding = UiRect::new(marge, marge, marge, Val::Px(0.0));
        noeud.left = Val::Px(bas.x / ui.0 - largeur / 2.0);
        noeud.top = Val::Px(bas.y / ui.0 - LIGNES * taille);
    }
}

/// Largeur approchée de l'étiquette (6 pixels de police par caractère, plus les marges).
fn largeur_estimee(textes: &Query<&mut TextePixel>, texte: Entity, taille: f32) -> f32 {
    let n = textes.get(texte).map_or(0, |t| t.texte.chars().count());
    (n as f32 * 6.0 + 2.0) * taille
}
