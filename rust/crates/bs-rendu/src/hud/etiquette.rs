//! Étiquettes des pseudos au-dessus du joueur et de ses coéquipiers de la coop
//! (src/effets/etiquette.ts) : texte blanc en police pixel sur un fond noir translucide,
//! toujours lisible (par-dessus le décor), cachée à la mort (et la sienne à la 3e personne). Haute de 0,42 bloc à 0,35 bloc au-dessus de la
//! tête : sa taille suit le zoom, par pixels de police entiers.

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
    /// Joueur de la partie (0 : moi).
    k: usize,
}

/// Crée les étiquettes (cachées tant qu'elles ne sont pas placées ; aucune pour un pseudo
/// vide).
pub fn creer(
    mut commandes: Commands,
    sauvegarde: Res<crate::sauvegarde::Sauvegarde>,
    coequipiers: Option<Res<crate::coequipiers::Coequipiers>>,
) {
    let autres = coequipiers
        .iter()
        .flat_map(|c| c.0.iter().map(|c| &c.pseudo));
    for (k, pseudo) in std::iter::once(&sauvegarde.profil.pseudo)
        .chain(autres)
        .enumerate()
    {
        if !pseudo.is_empty() {
            creer_une(&mut commandes, pseudo, k);
        }
    }
}

fn creer_une(commandes: &mut Commands, pseudo: &str, k: usize) {
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
    commandes.entity(fond).insert(Etiquette { texte, k });
}

/// Place l'étiquette au-dessus de la tête du joueur, à l'image.
pub fn placer(
    jeu: Res<PartieEnCours>,
    rendus: Option<Res<Rendus>>,
    camera: Query<(&Camera, &Transform), With<CameraJeu>>,
    ui: Res<UiScale>,
    options: Res<crate::options::Options>,
    mut etiquettes: Query<(&Etiquette, &mut Node, &mut Visibility, &ComputedNode)>,
    mut textes: Query<&mut TextePixel>,
) {
    let (Some(rendus), Ok((camera, t))) = (rendus, camera.single()) else {
        return;
    };
    // Transform de cette image (la GlobalTransform n'est à jour qu'après PostUpdate :
    // l'étiquette traînerait d'une image derrière le joueur)
    let gt = GlobalTransform::from(*t);
    let p = &jeu.partie;
    for (e, mut noeud, mut visible, calcule) in &mut etiquettes {
        let present = e.k < p.nombre_joueurs();
        let Some(r) = rendus.rendu_joueur(e.k).filter(|_| present) else {
            visible.set_if_neq(Visibility::Hidden);
            continue;
        };
        let pos = r.pose.position;
        let tete = Vec3::new(pos.x, pos.y + r.modele.hauteur + MARGE, pos.z);
        let haut = tete + Vec3::Y * HAUTEUR_TEXTE;
        let ecran = (
            camera.world_to_viewport(&gt, tete),
            camera.world_to_viewport(&gt, haut),
        );
        let (Ok(bas), Ok(sommet)) = ecran else {
            visible.set_if_neq(Visibility::Hidden);
            continue;
        };
        // Pas la sienne à la 3e personne (le joueur est juste devant la caméra)
        let moi_devant = e.k == 0 && options.troisieme_personne;
        let montre = !p.etat_joueur(e.k).mort && !moi_devant;
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
