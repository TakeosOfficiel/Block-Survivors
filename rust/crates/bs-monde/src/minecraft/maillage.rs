//! Maillages d'une schématique (src/minecraft/maillage.ts), sans carte graphique : un
//! maillage par texture, en coordonnées du jeu. Les faces cachées par un bloc plein
//! opaque voisin ne sont pas créées, comme dans Minecraft ; eau et lave sont des cubes
//! abaissés de 2 pixels (sauf sous le même liquide), plus bas encore là où elles
//! coulent (`level` 1 à 7), avec une marche visible contre un liquide plus bas.

use super::biomes::Teintes;
use super::cuisson::{Direction, ModeleCuit, Quad, V3};
use super::modeles::{BlocCuit, Liquide, TEXTURE_EAU, TEXTURE_LAVE, Variante};
use super::proprietes::{Teinte, liquide_affiche, teinte_de};
use super::schematique::Schematique;
use super::vent::{VENT_AUCUN, mode_vent, vent_du_bloc};

/// Rendu d'une texture, d'après ses pixels : opaque, découpée (feuilles, plantes) ou
/// translucide (verre teinté, eau).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenduTexture {
    Opaque,
    Decoupe,
    Translucide,
}

/// Classe une texture d'après ses pixels RGBA (première image seulement).
pub fn classer(rgba: &[u8], largeur: usize) -> RenduTexture {
    let (mut partiels, mut transparents) = (0, 0);
    for a in rgba.iter().skip(3).step_by(4).take(largeur * largeur) {
        if *a < 128 {
            transparents += 1;
        }
        if *a > 20 && *a < 235 {
            partiels += 1;
        }
    }
    if partiels > 0 {
        RenduTexture::Translucide
    } else if transparents > 0 {
        RenduTexture::Decoupe
    } else {
        RenduTexture::Opaque
    }
}

/// Faces d'une même texture.
#[derive(Debug, Clone, Default)]
pub struct MaillageTexture {
    /// Index dans la liste des textures.
    pub texture: usize,
    pub positions: Vec<V3>,
    pub normales: Vec<V3>,
    /// 0 à 1 sur une image de la texture (v vers le bas).
    pub uvs: Vec<[f32; 2]>,
    /// Teinte 0xRRGGBB (blanc : aucune).
    pub couleurs: Vec<u32>,
    /// Vent : centre x, pied y, centre z, mode (voir `vent`).
    pub vents: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

/// États de la palette qui cachent leurs voisins (et arrêtent la lumière) : cubes pleins
/// aux textures toutes opaques.
pub fn opaques(blocs: &[BlocCuit], textures: &[String], rendus: &[RenduTexture]) -> Vec<bool> {
    let index_texture = |t: &str| textures.binary_search_by(|x| x.as_str().cmp(t)).ok();
    let rendu = |t: &str| index_texture(t).map_or(RenduTexture::Opaque, |k| rendus[k]);
    blocs
        .iter()
        .map(|b| {
            b.modele_unique().is_some_and(|m| {
                m.cube_plein
                    && m.quads
                        .iter()
                        .all(|q| rendu(&q.texture) == RenduTexture::Opaque)
            })
        })
        .collect()
}

/// Maillages de la schématique ; `origine` : point de la schématique placé en (0, 0, 0)
/// dans le jeu (en blocs) ; `rendus[k]` : rendu de `textures[k]`.
pub fn construire_maillages(
    s: &Schematique,
    blocs: &[BlocCuit],
    textures: &[String],
    rendus: &[RenduTexture],
    origine: V3,
) -> Vec<MaillageTexture> {
    let index_texture = |t: &str| textures.binary_search_by(|x| x.as_str().cmp(t)).ok();
    let opaque = opaques(blocs, textures, rendus);
    // Vitres, glace… : un bloc plein non opaque cache les faces d'un voisin identique
    let fusionne: Vec<bool> = blocs
        .iter()
        .map(|b| b.modele_unique().is_some_and(|m| m.cube_plein) && !b.nom.ends_with("_leaves"))
        .collect();
    let modes: Vec<f32> = blocs.iter().map(mode_vent).collect();
    let liquides: Vec<Option<Liquide>> = blocs.iter().map(liquide_affiche).collect();
    let hauteurs: Vec<f32> = blocs.iter().map(hauteur_liquide).collect();
    let biome = Teintes::du_biome(s.biome.as_deref());
    // Couleurs par colonne quand la map mêle plusieurs biomes
    let colonnes = (s.grille_biomes.as_ref()).map(|g| g.teintes(s.largeur, s.profondeur));
    let teintes_en = |x: i32, z: i32| {
        colonnes
            .as_ref()
            .map_or(&biome, |c| &c[z as usize * s.largeur + x as usize])
    };
    let genres: Vec<Teinte> = blocs
        .iter()
        .map(|b| {
            if b.liquide == Some(Liquide::Eau) {
                Teinte::Eau
            } else {
                teinte_de(&b.nom)
            }
        })
        .collect();
    let mut sortie: Vec<MaillageTexture> = (0..textures.len())
        .map(|texture| MaillageTexture {
            texture,
            ..Default::default()
        })
        .collect();

    let cache = |x: i32, y: i32, z: i32, d: Direction, ici: Option<u16>| {
        let [dx, dy, dz] = d.vecteur();
        if y + dy < 0 {
            return true; // dessous de la schématique : jamais vu
        }
        match s.bloc(x + dx, y + dy, z + dz) {
            None => false,
            Some(v) => opaque[v as usize] || (Some(v) == ici && fusionne[v as usize]),
        }
    };

    for y in 0..s.hauteur as i32 {
        for z in 0..s.profondeur as i32 {
            for x in 0..s.largeur as i32 {
                let i = s.blocs[s.index(x as usize, y as usize, z as usize)];
                if i == 0 {
                    continue;
                }
                let b = &blocs[i as usize];
                let o = [
                    x as f32 - origine[0],
                    y as f32 - origine[1],
                    z as f32 - origine[2],
                ];
                let vent = vent_du_bloc(b, modes[i as usize], o);
                for partie in &b.parties {
                    let modele = choisir(partie, x, y, z);
                    for q in &modele.quads {
                        if q.cullface.is_some_and(|d| cache(x, y, z, d, Some(i))) {
                            continue;
                        }
                        let Some(k) = index_texture(&q.texture) else {
                            continue;
                        };
                        let couleur = if q.teinte {
                            genres[i as usize].couleur(teintes_en(x, z))
                        } else {
                            q.couleur
                        };
                        ajouter(
                            &mut sortie[k],
                            q,
                            o,
                            couleur,
                            vent,
                            rendus[k] != RenduTexture::Opaque,
                        );
                    }
                }
                let Some(liquide) = liquides[i as usize] else {
                    continue;
                };
                let meme = |nx: i32, ny: i32, nz: i32| {
                    s.bloc(nx, ny, nz)
                        .is_some_and(|v| liquides[v as usize] == Some(liquide))
                };
                // Hauteur de la surface d'un bloc de ce liquide (plein sous le même liquide)
                let hauteur = |nx: i32, ny: i32, nz: i32| match s.bloc(nx, ny, nz) {
                    Some(v) if liquides[v as usize] == Some(liquide) => {
                        Some(if meme(nx, ny + 1, nz) {
                            1.0
                        } else {
                            hauteurs[v as usize]
                        })
                    }
                    _ => None,
                };
                let haut = hauteur(x, y, z).unwrap_or(1.0);
                let texture = if liquide == Liquide::Eau {
                    TEXTURE_EAU
                } else {
                    TEXTURE_LAVE
                };
                let Some(k) = index_texture(texture) else {
                    continue;
                };
                let couleur = if liquide == Liquide::Eau {
                    teintes_en(x, z).eau
                } else {
                    0xffffff
                };
                for d in Direction::TOUTES {
                    let [dx, dy, dz] = d.vecteur();
                    // Contre le même liquide : seulement la marche qui dépasse du voisin
                    let bas = match hauteur(x + dx, y + dy, z + dz) {
                        Some(_) if dy != 0 => continue,
                        Some(hv) if hv >= haut => continue,
                        Some(hv) => hv,
                        None if cache(x, y, z, d, None) => continue,
                        None => 0.0,
                    };
                    ajouter(
                        &mut sortie[k],
                        &quad_liquide(d, bas, haut, texture),
                        o,
                        couleur,
                        [0.0, 0.0, 0.0, VENT_AUCUN],
                        false,
                    );
                }
            }
        }
    }
    sortie.retain(|m| !m.indices.is_empty());
    sortie
}

/// Variante pondérée, toujours la même pour une position donnée.
fn choisir(partie: &[Variante], x: i32, y: i32, z: i32) -> &ModeleCuit {
    if partie.len() == 1 {
        return &partie[0].modele;
    }
    let mut h =
        x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663) ^ z.wrapping_mul(83_492_791);
    h = (h ^ ((h as u32) >> 13) as i32).wrapping_mul(0x5bd1_e995);
    let total: f32 = partie.iter().map(|v| v.poids).sum();
    let mut seuil = (h as u32) as f32 / u32::MAX as f32 * total;
    for v in partie {
        seuil -= v.poids;
        if seuil < 0.0 {
            return &v.modele;
        }
    }
    &partie[partie.len() - 1].modele
}

fn ajouter(m: &mut MaillageTexture, q: &Quad, o: V3, couleur: u32, vent: [f32; 4], decoller: bool) {
    let base = m.positions.len() as u32;
    let n = q.normale;
    // Face découpée posée sur le bord du bloc (calque d'herbe…) : décollée d'un cheveu
    let mut e = [0.0; 3];
    if decoller {
        let p = q.positions[0];
        let bord = |v: f32| v == 0.0 || v == 1.0;
        if (0..3).any(|k| n[k].abs() == 1.0 && bord(p[k])) {
            e = n.map(|v| v * 0.002);
        }
    }
    for c in 0..4 {
        let p = q.positions[c];
        m.positions.push([0, 1, 2].map(|k| p[k] + o[k] + e[k]));
        m.normales.push(if q.ombre { n } else { [0.0, 1.0, 0.0] });
        m.uvs.push([q.uv[c][0] / 16.0, q.uv[c][1] / 16.0]);
        m.couleurs.push(couleur);
        m.vents.push(vent);
    }
    // Coins TL, TR, BR, BL vus de face : deux triangles dans le sens direct
    m.indices
        .extend([base, base + 3, base + 2, base, base + 2, base + 1]);
}

/// Hauteur de la surface d'un bloc de liquide : 14/16 pour une source ou une chute,
/// (8 - level) / 9 là où il coule (`level` 1 à 7), comme dans Minecraft.
fn hauteur_liquide(b: &BlocCuit) -> f32 {
    let level = b
        .propriete("level")
        .and_then(|l| l.parse::<u32>().ok())
        .unwrap_or(0);
    if (1..8).contains(&level) {
        (8 - level) as f32 / 9.0
    } else {
        14.0 / 16.0
    }
}

/// Face d'un cube de liquide de hauteur `h` (en blocs) ; une face de côté commence à
/// `b` (marche au-dessus d'un liquide plus bas).
fn quad_liquide(d: Direction, b: f32, h: f32, texture: &str) -> Quad {
    let c: [f32; 12] = match d {
        Direction::Nord => [1.0, h, 0.0, 0.0, h, 0.0, 0.0, b, 0.0, 1.0, b, 0.0],
        Direction::Sud => [0.0, h, 1.0, 1.0, h, 1.0, 1.0, b, 1.0, 0.0, b, 1.0],
        Direction::Ouest => [0.0, h, 0.0, 0.0, h, 1.0, 0.0, b, 1.0, 0.0, b, 0.0],
        Direction::Est => [1.0, h, 1.0, 1.0, h, 0.0, 1.0, b, 0.0, 1.0, b, 1.0],
        Direction::Haut => [0.0, h, 0.0, 1.0, h, 0.0, 1.0, h, 1.0, 0.0, h, 1.0],
        Direction::Bas => [0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    };
    let (v_haut, v_bas) = if matches!(d, Direction::Haut | Direction::Bas) {
        (0.0, 16.0)
    } else {
        (16.0 - h * 16.0, 16.0 - b * 16.0)
    };
    Quad {
        positions: [0, 1, 2, 3].map(|k| [c[k * 3], c[k * 3 + 1], c[k * 3 + 2]]),
        uv: [[0.0, v_haut], [16.0, v_haut], [16.0, v_bas], [0.0, v_bas]],
        texture: texture.to_owned(),
        normale: d.vecteur().map(|v| v as f32),
        cullface: None,
        teinte: true,
        couleur: 0xffffff,
        ombre: true,
    }
}
