//! Construction des objets du jeu à l'entrée en partie (textures chargées) : un lot
//! par type de projectile, palier d'orbe et objet au sol, plus les ombres. Matériaux :
//! éclairé et découpé, brillant (boules de feu), teinté par instance (orbes) ou
//! translucide (ombres).

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bs_donnees::Donnees;
use bs_donnees::entites::FormeProjectile;

use super::geometrie::{
    croisee, disque, modele_couche, peindre_fusee, plan, plan_sol, rectangle_uv,
};
use super::{
    Effets, Lot, OPACITE_OMBRE, ORIENTATION_ICONE, TAILLE_ORBE, TEXTURE_OMBRE, TEXTURE_ORBES,
    TypeProjectile,
};
use crate::PartieEnCours;
use crate::arene::couleur_lineaire;
use crate::materiau::ParametresBloc;
use crate::materiau_entite::MateriauEntite;
use crate::modeles::geometrie::{Maillage, objet};
use crate::textures::Textures;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

/// Textures des projectiles, orbes, objets et ombres.
pub fn demander_textures(d: &Donnees, serveur: &AssetServer, t: &mut Textures) {
    for p in d.projectiles.values() {
        t.demander(serveur, &p.texture);
    }
    for o in d.objets.values() {
        t.demander(serveur, &o.icone);
    }
    t.demander(serveur, TEXTURE_ORBES);
    t.demander(serveur, TEXTURE_OMBRE);
}

/// Image RGBA nette créée par le jeu (skin peinte, ombre de secours).
fn image(pixels: Vec<u8>, cote: u32) -> Image {
    let mut i = Image::new(
        Extent3d {
            width: cote,
            height: cote,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    i.sampler = ImageSampler::nearest();
    i
}

/// Pixels RGBA d'une texture chargée.
fn pixels<'a>(images: &'a Assets<Image>, h: &Option<Handle<Image>>) -> Option<(&'a [u8], usize)> {
    let i = images.get(h.as_ref()?)?;
    let (l, ht) = (i.width() as usize, i.height() as usize);
    let px = i.data.as_deref()?;
    (px.len() == l * ht * 4).then_some((px, l))
}

#[allow(clippy::too_many_arguments)]
pub fn construire(
    mut commandes: Commands,
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    jeu: Res<PartieEnCours>,
    mut images: ResMut<Assets<Image>>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<MateriauEntite>>,
) {
    let d = jeu.partie.d;
    let ambiance = ParametresBloc::de_l_arene(jeu.partie.etat.arene);
    // Éclairé, découpé ; brillant (non éclairé) ; teinté par instance ; translucide
    let mut eclaire = ambiance;
    eclaire.brouillard.z = 0.5;
    eclaire.brouillard.w = 0.0;
    let mut brillant = eclaire;
    brillant.brouillard.w = 1.0;
    let mut teinte = brillant;
    teinte.animation.y = 1.0;
    let mut ombre = brillant;
    ombre.brouillard.z = 0.0;
    ombre.animation.z = OPACITE_OMBRE;
    ombre.animation.w = 1.0;

    let mut lot = |nom: &str, m: Maillage, parametres: ParametresBloc, texture| {
        Lot::nouveau(
            nom,
            maillages.add(m.en_mesh()),
            materiaux.add(MateriauEntite {
                parametres,
                texture,
            }),
        )
    };

    let mut projectiles = Vec::with_capacity(d.projectiles.len());
    for (id, def) in &d.projectiles {
        let mut tex = textures.obtenir(&serveur, &def.texture);
        let secours = couleur_lineaire(def.couleur_secours);
        let couleur = if tex.is_some() { [1.0; 4] } else { secours };
        let uv = rectangle_uv(def.uv, def.taille_texture);
        let mut parametres = eclaire;
        let m = match def.forme {
            FormeProjectile::Croisee => croisee(def.longueur, def.largeur, uv, couleur),
            FormeProjectile::FaceCamera => {
                parametres = brillant;
                plan(def.longueur, def.longueur, uv, couleur)
            }
            FormeProjectile::Modele => {
                if def.peinture.as_deref() == Some("fusee") {
                    let item = pixels(&images, &tex).map(|(px, _)| px.to_vec());
                    tex = Some(images.add(image(peindre_fusee(item.as_deref()), 32)));
                }
                let couleur = if tex.is_some() { [1.0; 4] } else { secours };
                match def.modele.as_deref().and_then(|m| d.modeles.get(m)) {
                    Some(modele) => modele_couche(modele, couleur),
                    None => plan(def.longueur, def.longueur, uv, secours),
                }
            }
            FormeProjectile::Relief => match pixels(&images, &tex) {
                Some((px, l)) => {
                    let mut m = objet(px, l, px.len() / 4 / l, def.longueur / l as f32);
                    // Couché à plat, pointe vers l'avant (+Z)
                    let r =
                        Quat::from_rotation_y(ORIENTATION_ICONE) * Quat::from_rotation_z(FRAC_PI_2);
                    tourner(&mut m, r);
                    m
                }
                None => {
                    tex = None;
                    let mut m = plan(def.longueur, def.longueur, uv, secours);
                    tourner(
                        &mut m,
                        Quat::from_rotation_y(-3.0 * FRAC_PI_4) * Quat::from_rotation_x(-FRAC_PI_2),
                    );
                    m
                }
            },
        };
        projectiles.push(TypeProjectile {
            def,
            lot: lot(&format!("projectile:{id}"), m, parametres, tex),
        });
    }

    let tex_orbes = textures.obtenir(&serveur, TEXTURE_ORBES);
    let orbes = d
        .regles
        .progression
        .orbes
        .iter()
        .enumerate()
        .map(|(k, palier)| {
            let cote = TAILLE_ORBE * (1.0 + k as f32 * 0.15);
            let (x, y) = (
                (palier.image % 4) as f32 * 16.0,
                (palier.image / 4) as f32 * 16.0,
            );
            let uv = rectangle_uv([x, y, 16.0, 16.0], [64.0, 64.0]);
            let couleur = if tex_orbes.is_some() {
                [1.0; 4]
            } else {
                couleur_lineaire(0xb8ff40)
            };
            lot(
                &format!("orbes:{k}"),
                plan(cote, cote, uv, couleur),
                teinte,
                tex_orbes.clone(),
            )
        })
        .collect();

    let objets = d
        .objets
        .iter()
        .map(|(id, o)| {
            let tex = textures.obtenir(&serveur, &o.icone);
            let m = match pixels(&images, &tex) {
                Some((px, l)) => objet(px, l, px.len() / 4 / l, o.taille / l as f32),
                None => {
                    let mut m = plan(
                        o.taille,
                        o.taille,
                        [0.0, 0.0, 1.0, 1.0],
                        couleur_lineaire(o.couleur_secours),
                    );
                    tourner(&mut m, Quat::from_rotation_x(-FRAC_PI_4));
                    m
                }
            };
            lot(&format!("objet:{id}"), m, eclaire, tex)
        })
        .collect();

    let tex_ombre = textures
        .obtenir(&serveur, TEXTURE_OMBRE)
        .unwrap_or_else(|| images.add(image(disque(16), 16)));
    let mut ombre_noire = plan_sol();
    if textures.obtenir(&serveur, TEXTURE_OMBRE).is_none() {
        ombre_noire.couleurs.fill([0.0, 0.0, 0.0, 1.0]);
    }
    let ombres = lot("ombres", ombre_noire, ombre, Some(tex_ombre));

    commandes.insert_resource(Effets {
        projectiles,
        orbes,
        objets,
        ombres,
        vitrine: std::env::var_os("BS_VITRINE").is_some(),
    });
}

/// Tourne tout le maillage (positions et normales).
fn tourner(m: &mut Maillage, r: Quat) {
    for p in &mut m.positions {
        *p = (r * Vec3::from_array(*p)).to_array();
    }
    for n in &mut m.normales {
        *n = (r * Vec3::from_array(*n)).to_array();
    }
}
