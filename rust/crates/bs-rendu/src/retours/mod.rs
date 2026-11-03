//! Retours visuels de la simulation (src/effets/retours.ts) : les gerbes de particules,
//! chiffres de dégâts et secousses que chaque pas a empilés sont recueillis après le
//! pas, puis vivent et se dessinent à chaque image. S'y ajoutent les barres de vie des
//! alliés blessés et des mini-boss (src/effets/barresVie.ts), demandées par `entites`.

mod chiffres;
mod maillage;
mod particules;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bs_donnees::Donnees;
use bs_donnees::armes::Mecanique;
use bs_regles::evenements::{Evenement, StyleChiffre};

use crate::arene::couleur_lineaire;
use crate::camera::Secousse;
use crate::effets::Effets;
use crate::effets::pixels;
use crate::materiau::ParametresBloc;
use crate::materiau_entite::MateriauEntite;
use crate::textures::Textures;
use crate::{PAS, PartieEnCours};
use chiffres::Chiffres;
use maillage::MaillageDynamique;
use particules::{Particules, construire_atlas, images_atlas};

/// Barres de vie (barresVie.ts) : longueur et hauteur de base, bord du fond.
const BARRE_LONGUEUR: f32 = 0.8;
const BARRE_HAUTEUR: f32 = 0.09;
const BARRE_BORD: f32 = 0.02;
const BARRE_FOND: u32 = 0x1a1a1a;
const BARRE_PLEINE: u32 = 0x55ff55;
const BARRE_VIDE: u32 = 0xff5555;

/// Barres à dessiner cette image : (x, y, z, part de vie, longueur relative).
#[derive(Resource, Default)]
pub struct Barres(pub Vec<[f32; 5]>);

#[derive(Resource)]
pub struct Retours {
    particules: Particules,
    chiffres: Chiffres,
    barres: MaillageDynamique,
    /// Vitrine : temps avant la prochaine démonstration.
    minuteur: f32,
}

/// Vitrine : gerbes montrées à tour de rôle devant le joueur.
const GERBES_VITRINE: [&str; 6] = [
    "mort",
    "explosion",
    "critique",
    "niveau",
    "ramassage",
    "eboulis",
];

/// Icônes dont les particules « morceau » font des éclats (objets, éboulis du sol et
/// des armes), comme les appels `emettre(…, icone)` de la version TS.
fn icones(d: &'static Donnees, sol: &'static str) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = d.objets.values().map(|o| o.icone.as_str()).collect();
    v.extend([
        sol,
        "block/magma",
        "block/packed_ice",
        "block/sweet_berry_bush_stage3",
    ]);
    for a in d.armes.values() {
        match &a.mecanique {
            Mecanique::Meteore { texture, .. } | Mecanique::Nova { texture, .. } => v.push(texture),
            Mecanique::Piege { image, .. } => v.push(image),
            _ => {}
        }
    }
    v
}

/// Textures des particules et des éclats.
pub fn demander_textures(jeu: &PartieEnCours, serveur: &AssetServer, t: &mut Textures) {
    let d = jeu.partie.d;
    for chemin in images_atlas(d, &icones(d, jeu.partie.texture_sol())) {
        t.demander(serveur, chemin);
    }
}

fn image(pixels: Vec<u8>, largeur: u32, hauteur: u32) -> Image {
    let mut i = Image::new(
        Extent3d {
            width: largeur,
            height: hauteur,
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
    // Non éclairés ; les chiffres et barres passent devant le décor (`animation.x`)
    let mut base = ParametresBloc::pour_entites(jeu.partie.etat.arene);
    base.brouillard.w = 1.0;
    let mut decoupe = base;
    decoupe.brouillard.z = 0.1;
    let mut devant = base;
    devant.brouillard.z = 0.01;
    devant.animation = Vec4::new(1.0, 0.0, 1.0, 1.0);
    // Les chiffres et barres ne s'effacent pas dans le brouillard
    devant.brouillard.x = 1.0e6;
    devant.brouillard.y = 1.0e6 + 1.0;

    let liste = images_atlas(d, &icones(d, jeu.partie.texture_sol()));
    let poignees: Vec<Option<Handle<Image>>> = liste
        .iter()
        .map(|c| textures.obtenir(&serveur, c))
        .collect();
    let (atlas, lignes) = {
        let lire = |chemin: &str| {
            let k = liste.iter().position(|c| *c == chemin)?;
            pixels(&images, &poignees[k])
        };
        construire_atlas(&liste, lire)
    };
    let largeur = (particules::COLONNES * particules::CASE) as u32;
    let hauteur = (lignes * particules::CASE) as u32;
    let tex_particules = images.add(image(atlas, largeur, hauteur));
    let (px, l, h) = chiffres::atlas();
    let tex_chiffres = images.add(image(px, l, h));

    let mut materiau = |parametres, texture| {
        materiaux.add(MateriauEntite {
            parametres,
            texture,
            lumiere: None,
        })
    };
    let m_particules = materiau(decoupe, Some(tex_particules));
    let m_chiffres = materiau(devant, Some(tex_chiffres));
    let m_barres = materiau(devant, None);
    let mut dynamique = |m, nom| MaillageDynamique::nouveau(&mut commandes, &mut maillages, m, nom);
    let particules = Particules::nouvelles(
        &liste,
        lignes,
        jeu.graine,
        dynamique(m_particules, "particules"),
    );
    let chiffres = Chiffres::nouveaux(jeu.graine, dynamique(m_chiffres, "chiffres"));
    let barres = dynamique(m_barres, "barres-vie");
    commandes.insert_resource(Retours {
        particules,
        chiffres,
        barres,
        minuteur: 0.0,
    });
}

/// Après chaque pas : gerbes, chiffres et secousses empilés par la simulation.
pub fn recevoir(
    jeu: Res<PartieEnCours>,
    mut retours: ResMut<Retours>,
    mut secousse: ResMut<Secousse>,
    effets: Res<Effets>,
    options: Res<crate::options::Options>,
) {
    let d = jeu.partie.d;
    if effets.vitrine {
        vitrine(&jeu, &mut retours);
    }
    for ev in &jeu.partie.ev.liste {
        match *ev {
            Evenement::Gerbe {
                gerbe,
                icone,
                x,
                y,
                z,
            } => retours
                .particules
                .emettre(d, gerbe, icone, Vec3::new(x, y, z)),
            Evenement::Degats {
                x,
                y,
                z,
                valeur,
                style,
            } => {
                if options.chiffres_degats && crate::scene::scene().is_none() {
                    retours.chiffres.ajouter(Vec3::new(x, y, z), valeur, style);
                }
            }
            Evenement::Secousse(force) => {
                if options.secousses {
                    secousse.ajouter(force);
                }
            }
            Evenement::Son { .. }
            | Evenement::Geste(_)
            | Evenement::Effet { .. }
            | Evenement::AnnonceBoss { .. }
            | Evenement::ButinBoss { .. }
            | Evenement::Decouverte(_) => {}
        }
    }
}

/// `BS_VITRINE` : une gerbe de chaque sorte et un chiffre de chaque style toutes les
/// demi-secondes, derrière le joueur.
fn vitrine(jeu: &PartieEnCours, retours: &mut Retours) {
    let p = &jeu.partie;
    let j = &p.etat.joueur;
    let n = GERBES_VITRINE.len();
    let x = |k: usize| j.x + (k as f32 - (n as f32 - 1.0) / 2.0) * 1.5;
    retours.minuteur -= PAS;
    if retours.minuteur > 0.0 {
        return;
    }
    retours.minuteur = 0.5;
    let icone = p.d.objets.values().next().map(|o| o.icone.as_str());
    let sol = p.texture_sol();
    for (k, g) in GERBES_VITRINE.iter().enumerate() {
        let i = match *g {
            "ramassage" => icone,
            "eboulis" => Some(sol),
            _ => None,
        };
        retours
            .particules
            .emettre(p.d, g, i, Vec3::new(x(k), 0.6, j.z - 4.0));
    }
    let styles = [
        StyleChiffre::Normal,
        StyleChiffre::Critique,
        StyleChiffre::Feu,
        StyleChiffre::Poison,
    ];
    for (k, s) in styles.into_iter().enumerate() {
        let v = [7.0, 1234.0, 3.0, 42.0][k];
        retours
            .chiffres
            .ajouter(Vec3::new(x(k + 1), 1.2, j.z - 1.5), v, s);
    }
}

/// Vie et dessin de l'image, face à la caméra.
pub fn dessiner(
    temps: Res<Time>,
    mut retours: ResMut<Retours>,
    mut barres: ResMut<Barres>,
    effets: Res<Effets>,
    jeu: Res<PartieEnCours>,
    camera: Query<&Transform, With<Camera3d>>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut visibilites: Query<&mut Visibility>,
) {
    let dt = temps.delta_secs();
    let rotation = camera.single().map_or(Quat::IDENTITY, |t| t.rotation);
    let (droite, haut) = (rotation * Vec3::X, rotation * Vec3::Y);
    let Retours {
        particules,
        chiffres,
        barres: maillage_barres,
        ..
    } = &mut *retours;
    particules.avancer(dt);
    particules.dessiner(droite, haut);
    particules
        .maillage
        .terminer(&mut maillages, &mut visibilites);
    chiffres.avancer(dt);
    chiffres.dessiner(droite, haut);
    chiffres.maillage.terminer(&mut maillages, &mut visibilites);

    // Barres : fond sombre, puis remplissage aligné à gauche, du rouge au vert
    let fond = couleur_lineaire(BARRE_FOND);
    let (vide, pleine) = (
        Vec4::from_array(couleur_lineaire(BARRE_VIDE)),
        Vec4::from_array(couleur_lineaire(BARRE_PLEINE)),
    );
    if effets.vitrine {
        let j = &jeu.partie.etat.joueur;
        barres.0.push([j.x - 1.0, 1.5, j.z - 2.5, 0.3, 1.0]);
        barres.0.push([j.x + 1.0, 1.5, j.z - 2.5, 0.8, 2.0]);
    }
    maillage_barres.commencer();
    for &[x, y, z, part, taille] in &barres.0 {
        let part = part.clamp(0.0, 1.0);
        let centre = Vec3::new(x, y, z);
        let longueur = BARRE_LONGUEUR * taille;
        let h = BARRE_HAUTEUR * taille.sqrt();
        let uv = [0.0, 0.0, 1.0, 1.0];
        maillage_barres.carre(
            centre,
            droite * (longueur / 2.0 + BARRE_BORD),
            haut * (h / 2.0 + BARRE_BORD),
            uv,
            fond,
        );
        let largeur = (longueur * part).max(1e-3);
        maillage_barres.carre(
            centre + droite * ((largeur - longueur) / 2.0) - rotation * Vec3::Z * 0.001,
            droite * (largeur / 2.0),
            haut * (h / 2.0),
            uv,
            vide.lerp(pleine, part).to_array(),
        );
    }
    maillage_barres.terminer(&mut maillages, &mut visibilites);
}
