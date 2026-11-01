//! Police pixel de Minecraft (src/assets/police.ts), sans fichier de police : les
//! planches vanilla (`font/ascii.png`…) et leur table de caractères
//! (`font/include/default.json`) sont lues au lancement. Chaque texte du HUD
//! (`TextePixel`) est dessiné en pixels dans sa propre image, redessinée seulement quand
//! le texte change : ombre du jeu (décalée d'un pixel, couleur au quart) ou contour noir.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use serde_json::Value;

use crate::fichiers::Json;
use crate::textures::Textures;

/// Table des caractères du jeu.
const TABLE: &str = "mcj://font/include/default.json";
/// Avance de l'espace, en pixels (fournisseur « space » du jeu).
const AVANCE_ESPACE: i32 = 4;
/// Ligne de base : 7 pixels sous le haut des majuscules.
const ASCENT: i32 = 7;

/// Un caractère d'une planche.
#[derive(Clone, Copy)]
struct Glyphe {
    planche: u8,
    x: u16,
    y: u16,
    largeur: u8,
    hauteur: u8,
    ascent: i8,
}

struct Planche {
    largeur: usize,
    rgba: Vec<u8>,
}

/// Fournisseur bitmap de la table, en attente de sa planche.
struct Fournisseur {
    texture: String,
    ascent: i32,
    lignes: Vec<Vec<char>>,
}

/// Police du jeu ; `version` change quand elle devient prête (tout est redessiné).
#[derive(Resource, Default)]
pub struct Police {
    table: Option<Handle<Json>>,
    attente: Vec<Fournisseur>,
    glyphes: HashMap<char, Glyphe>,
    planches: Vec<Planche>,
    pub version: u32,
}

/// Bord du texte.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Bord {
    /// Ombre du jeu, un pixel en bas à droite.
    Ombre,
    /// Contour noir d'un pixel (niveau sur la barre d'XP).
    Contour,
    /// Rien (texte sombre sur fond clair).
    Aucun,
}

/// Texte affiché dans l'`ImageNode` de son entité ; `taille` : pixels d'écran par pixel
/// de police (2 = police de 16 px).
#[derive(Component)]
#[require(ImageNode, Node, ImageTexte)]
pub struct TextePixel {
    pub texte: String,
    pub couleur: Color,
    pub bord: Bord,
    pub taille: f32,
    /// Largeur maximale en pixels d'écran : coupé entre les mots, lignes centrées.
    pub largeur_max: Option<f32>,
}

/// Image propre au texte (réécrite à chaque changement).
#[derive(Component, Default)]
pub struct ImageTexte(Option<Handle<Image>>);

impl TextePixel {
    pub fn nouveau(texte: &str, couleur: Color, taille: f32) -> Self {
        Self {
            texte: texte.to_owned(),
            couleur,
            bord: Bord::Ombre,
            taille,
            largeur_max: None,
        }
    }

    /// Texte coupé à `largeur` pixels d'écran, lignes centrées.
    pub fn bloc(mut self, largeur: f32) -> Self {
        self.largeur_max = Some(largeur);
        self
    }

    pub fn contour(mut self) -> Self {
        self.bord = Bord::Contour;
        self
    }

    pub fn sans_bord(mut self) -> Self {
        self.bord = Bord::Aucun;
        self
    }
}

/// Remplace le texte s'il a changé (pas de redessin sinon).
pub fn ecrire(t: &mut Mut<TextePixel>, texte: &str) {
    if t.texte != texte {
        t.texte.clear();
        t.texte.push_str(texte);
    }
}

/// Change la couleur si elle a changé.
pub fn colorer(t: &mut Mut<TextePixel>, couleur: Color) {
    if t.couleur != couleur {
        t.couleur = couleur;
    }
}

/// Lit la table, demande les planches, puis découpe les caractères.
pub fn charger(
    mut police: ResMut<Police>,
    serveur: Res<AssetServer>,
    json: Res<Assets<Json>>,
    mut textures: ResMut<Textures>,
    images: Res<Assets<Image>>,
) {
    if police.version > 0 {
        return;
    }
    let Some(table) = police.table.clone() else {
        police.table = Some(serveur.load(TABLE));
        return;
    };
    if police.attente.is_empty() {
        let Some(Json(v)) = json.get(&table) else {
            if serveur.load_state(&table).is_failed() {
                warn!("table de la police introuvable : texte invisible");
                police.version = 1;
            }
            return;
        };
        police.attente = fournisseurs(v);
        for f in &police.attente {
            textures.demander(&serveur, &f.texture);
        }
        if police.attente.is_empty() {
            police.version = 1;
        }
        return;
    }
    if !police
        .attente
        .iter()
        .all(|f| textures.termine(&serveur, &f.texture))
    {
        return;
    }
    let attente = std::mem::take(&mut police.attente);
    for f in &attente {
        let image = textures
            .obtenir(&serveur, &f.texture)
            .and_then(|p| images.get(&p));
        if let Some(i) = image {
            police.ajouter(f, i);
        }
    }
    police.version = 1;
}

/// Fournisseurs bitmap de la table (le premier qui a un caractère l'emporte, comme en jeu).
fn fournisseurs(v: &Value) -> Vec<Fournisseur> {
    let liste = v["providers"].as_array().map_or(&[][..], Vec::as_slice);
    liste
        .iter()
        .filter(|f| f["type"] == "bitmap")
        .filter_map(|f| {
            let fichier = f["file"].as_str()?;
            let texture = fichier
                .trim_start_matches("minecraft:")
                .trim_end_matches(".png")
                .to_owned();
            let lignes = f["chars"]
                .as_array()?
                .iter()
                .filter_map(|l| Some(l.as_str()?.chars().collect()))
                .collect();
            Some(Fournisseur {
                texture,
                ascent: f["ascent"].as_i64().unwrap_or(7) as i32,
                lignes,
            })
        })
        .collect()
}

impl Police {
    fn ajouter(&mut self, f: &Fournisseur, image: &Image) {
        let Some(rgba) = image.data.as_ref() else {
            return;
        };
        let colonnes = f.lignes.first().map_or(0, Vec::len);
        if colonnes == 0 || f.lignes.is_empty() {
            return;
        }
        let largeur = image.width() as usize;
        let (lc, hc) = (largeur / colonnes, image.height() as usize / f.lignes.len());
        let planche = self.planches.len() as u8;
        let allume = |x: usize, y: usize| rgba[(y * largeur + x) * 4 + 3] > 0;
        for (rangee, ligne) in f.lignes.iter().enumerate() {
            for (colonne, &c) in ligne.iter().enumerate() {
                if c == '\0' || c == ' ' || self.glyphes.contains_key(&c) {
                    continue;
                }
                let (x0, y0) = (colonne * lc, rangee * hc);
                // Largeur : dernière colonne allumée + 1
                let l = (0..lc)
                    .rev()
                    .find(|&x| (0..hc).any(|y| allume(x0 + x, y0 + y)))
                    .map_or(0, |x| x + 1);
                self.glyphes.insert(
                    c,
                    Glyphe {
                        planche,
                        x: x0 as u16,
                        y: y0 as u16,
                        largeur: l as u8,
                        hauteur: hc as u8,
                        ascent: f.ascent as i8,
                    },
                );
            }
        }
        self.planches.push(Planche {
            largeur,
            rgba: rgba.clone(),
        });
    }

    /// Avance d'un caractère (largeur + 1 pixel d'espacement).
    fn avance(&self, c: char) -> i32 {
        match self.glyphes.get(&c) {
            _ if c == ' ' => AVANCE_ESPACE,
            Some(g) => i32::from(g.largeur) + 1,
            None => 0,
        }
    }

    /// Largeur d'un texte en pixels de police (sans le bord).
    pub fn largeur(&self, texte: &str) -> i32 {
        texte.chars().map(|c| self.avance(c)).sum::<i32>().max(1) - 1
    }

    /// Lignes d'un texte coupé entre les mots à `max` pixels de police.
    fn couper<'a>(&self, texte: &'a str, max: Option<i32>, lignes: &mut Vec<&'a str>) {
        lignes.clear();
        let Some(max) = max else {
            lignes.push(texte);
            return;
        };
        for paragraphe in texte.split('\n') {
            let mut debut = 0;
            let mut fin = 0;
            for (i, _) in paragraphe
                .match_indices(' ')
                .chain([(paragraphe.len(), "")])
            {
                if fin > debut && self.largeur(&paragraphe[debut..i]) > max {
                    lignes.push(&paragraphe[debut..fin]);
                    debut = fin + 1;
                }
                fin = i;
            }
            lignes.push(&paragraphe[debut.min(paragraphe.len())..]);
        }
    }

    /// Pixels RGBA du texte, avec son bord : (largeur, hauteur, pixels).
    fn dessiner(
        &self,
        texte: &str,
        couleur: [u8; 3],
        bord: Bord,
        max: Option<i32>,
    ) -> (u32, u32, Vec<u8>) {
        let mut lignes = Vec::new();
        self.couper(texte, max, &mut lignes);
        // Accents qui dépassent au-dessus des majuscules
        let dessus = texte
            .chars()
            .filter_map(|c| self.glyphes.get(&c))
            .map(|g| i32::from(g.ascent) - ASCENT)
            .max()
            .unwrap_or(0)
            .max(0);
        // Marge à gauche et en haut, place ajoutée, couleur et décalages du bord
        let (marge, extra, fond, decalages): (i32, i32, [u8; 3], &[(i32, i32)]) = match bord {
            Bord::Ombre => (0, 1, couleur.map(|v| v / 4), &[(1, 1)]),
            Bord::Contour => (1, 2, [0; 3], &[(1, 0), (-1, 0), (0, 1), (0, -1)]),
            Bord::Aucun => (0, 0, [0; 3], &[]),
        };
        let interligne = 8 + dessus + 2;
        let largeurs: Vec<i32> = lignes.iter().map(|t| self.largeur(t)).collect();
        let l = (largeurs.iter().copied().max().unwrap_or(0) + extra).max(1);
        let h = 8 + dessus + extra + interligne * (lignes.len() as i32 - 1).max(0);
        let mut px = vec![0u8; (l * h * 4) as usize];
        let mut poser = |x: i32, y: i32, c: [u8; 3]| {
            if x >= 0 && y >= 0 && x < l && y < h {
                let i = ((y * l + x) * 4) as usize;
                px[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
            }
        };
        // Deux passes : le bord d'abord, puis le texte par-dessus
        for passe in 0..2 {
            for (k, ligne) in lignes.iter().enumerate() {
                let mut x = marge + (l - extra - largeurs[k]) / 2;
                let y0 = marge + dessus + k as i32 * interligne;
                for c in ligne.chars() {
                    if let Some(g) = self.glyphes.get(&c) {
                        let p = &self.planches[g.planche as usize];
                        let haut = y0 + ASCENT - i32::from(g.ascent);
                        for gy in 0..i32::from(g.hauteur) {
                            for gx in 0..i32::from(g.largeur) {
                                let i = ((g.y as usize + gy as usize) * p.largeur
                                    + g.x as usize
                                    + gx as usize)
                                    * 4;
                                if p.rgba[i + 3] == 0 {
                                    continue;
                                }
                                if passe == 0 {
                                    for (dx, dy) in decalages {
                                        poser(x + gx + dx, haut + gy + dy, fond);
                                    }
                                } else {
                                    let teinte = [0, 1, 2].map(|k| {
                                        (u16::from(p.rgba[i + k]) * u16::from(couleur[k]) / 255)
                                            as u8
                                    });
                                    poser(x + gx, haut + gy, teinte);
                                }
                            }
                        }
                    }
                    x += self.avance(c);
                }
            }
        }
        (l as u32, h as u32, px)
    }
}

/// Redessine les textes changés (tous quand la police arrive).
pub fn dessiner(
    police: Res<Police>,
    mut vue: Local<u32>,
    mut images: ResMut<Assets<Image>>,
    mut textes: Query<(Ref<TextePixel>, &mut ImageTexte, &mut ImageNode, &mut Node)>,
) {
    if police.version == 0 {
        return;
    }
    let tout = *vue != police.version;
    *vue = police.version;
    for (t, mut propre, mut image, mut node) in &mut textes {
        if !tout && !t.is_changed() {
            continue;
        }
        let c = t.couleur.to_srgba();
        let couleur = [c.red, c.green, c.blue].map(|v| (v * 255.0).round() as u8);
        let max = t.largeur_max.map(|m| (m / t.taille) as i32);
        let (l, h, px) = police.dessiner(&t.texte, couleur, t.bord, max);
        let mut nouvelle = Image::new(
            Extent3d {
                width: l,
                height: h,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            px,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        nouvelle.sampler = ImageSampler::nearest();
        let poignee = match &propre.0 {
            Some(p) => {
                let _ = images.insert(p.id(), nouvelle);
                p.clone()
            }
            None => {
                let p = images.add(nouvelle);
                propre.0 = Some(p.clone());
                p
            }
        };
        if image.image != poignee {
            image.image = poignee.clone();
        }
        node.width = Val::Px(l as f32 * t.taille);
        node.height = Val::Px(h as f32 * t.taille);
    }
}
