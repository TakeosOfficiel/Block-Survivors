//! Maillages de l'arène générée (`bs-monde`) : pour chaque type de bloc, un maillage
//! fusionné par groupe de faces (côtés, dessus, dessous), donc trois appels de dessin par
//! type de bloc, comme les `InstancedMesh` de src/monde/arene.ts. Les faces collées à un
//! bloc plein opaque ne sont pas émises (jamais visibles).
//!
//! Mêmes règles qu'en TS : rotation aléatoire par quart de tour (`hachage3`), liquides
//! abaissés de 2 pixels, croix en deux plans diagonaux, teintes de biome, et damier de la
//! couleur de secours quand la texture manque. Fonctions pures, testées sans carte
//! graphique ; la conversion en `Mesh` Bevy est à part.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bs_commun::hachage3;
use bs_donnees::Donnees;
use bs_donnees::arenes::Arene;
use bs_donnees::blocs::{Bloc, FormeBloc, Ondulation};
use bs_monde::MondeGenere;
use bs_monde::minecraft::vent::{VENT_FEUILLAGE, VENT_PLANTE};

use crate::materiau::ATTRIBUTE_VENT;

/// Un liquide est un cube abaissé de 2 pixels, comme en jeu.
const HAUTEUR_LIQUIDE: f32 = 14.0 / 16.0;
/// Demi-diagonale des plans d'une croix, légèrement rentrée comme en jeu.
const DEMI_CROIX: f32 = 0.45;
/// Couleurs de secours : un bloc sur deux assombri pour lire la grille.
const CONTRASTE_DAMIER: f32 = 0.9;

/// Groupe de faces d'un bloc, chacun avec sa texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupeFaces {
    Cotes,
    Haut,
    Bas,
}

impl GroupeFaces {
    pub fn texture(self, b: &Bloc) -> &str {
        match self {
            Self::Cotes => &b.faces.cotes,
            Self::Haut => &b.faces.haut,
            Self::Bas => &b.faces.bas,
        }
    }

    /// Teinte de biome de ce groupe (0xRRGGBB), si la texture est grise en jeu.
    pub fn teinte(self, b: &Bloc) -> Option<u32> {
        match self {
            Self::Haut => b.teinte_haut.or(b.teinte),
            _ => b.teinte,
        }
    }
}

/// Maillage d'un groupe de faces d'un type de bloc, en tableaux à plat.
#[derive(Debug, Default, Clone)]
pub struct MaillageBloc {
    pub bloc: usize,
    pub groupe: Option<GroupeFaces>,
    pub positions: Vec<[f32; 3]>,
    pub normales: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// Couleur (linéaire) multipliée à la texture : teinte, ou damier de secours.
    pub couleurs: Vec<[f32; 4]>,
    /// Vent : centre x, pied y, centre z, mode (`bs_monde::minecraft::vent`).
    pub vents: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl MaillageBloc {
    pub fn faces(&self) -> usize {
        self.indices.len() / 6
    }

    pub fn en_mesh(&self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normales.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.couleurs.clone())
        .with_inserted_attribute(ATTRIBUTE_VENT, self.vents.clone())
        .with_inserted_indices(Indices::U32(self.indices.clone()))
    }

    /// Ajoute un quadrilatère (sommets dans l'ordre des UV : haut-gauche, haut-droite,
    /// bas-droite, bas-gauche, vus de face).
    fn quad(
        &mut self,
        coins: [Vec3; 4],
        normale: Vec3,
        couleur: [f32; 4],
        uvs: [[f32; 2]; 4],
        vent: [f32; 4],
    ) {
        let base = self.positions.len() as u32;
        for (c, uv) in coins.iter().zip(uvs) {
            self.positions.push(c.to_array());
            self.normales.push(normale.to_array());
            self.uvs.push(uv);
            self.couleurs.push(couleur);
            self.vents.push(vent);
        }
        self.indices
            .extend([base, base + 3, base + 2, base, base + 2, base + 1]);
    }
}

/// Une face d'un cube unitaire centré : normale, coins (UV standard), groupe.
struct FaceCube {
    normale: Vec3,
    coins: [Vec3; 4],
    groupe: GroupeFaces,
}

/// Les six faces, orientées comme la `BoxGeometry` de three.js (dessus : haut de la
/// texture vers −Z ; côtés : texture debout).
fn faces_cube() -> [FaceCube; 6] {
    let v = Vec3::new;
    let h = 0.5;
    [
        // +X : u vers −Z
        FaceCube {
            normale: Vec3::X,
            coins: [v(h, h, h), v(h, h, -h), v(h, -h, -h), v(h, -h, h)],
            groupe: GroupeFaces::Cotes,
        },
        // −X : u vers +Z
        FaceCube {
            normale: Vec3::NEG_X,
            coins: [v(-h, h, -h), v(-h, h, h), v(-h, -h, h), v(-h, -h, -h)],
            groupe: GroupeFaces::Cotes,
        },
        // +Z : u vers +X
        FaceCube {
            normale: Vec3::Z,
            coins: [v(-h, h, h), v(h, h, h), v(h, -h, h), v(-h, -h, h)],
            groupe: GroupeFaces::Cotes,
        },
        // −Z : u vers −X
        FaceCube {
            normale: Vec3::NEG_Z,
            coins: [v(h, h, -h), v(-h, h, -h), v(-h, -h, -h), v(h, -h, -h)],
            groupe: GroupeFaces::Cotes,
        },
        // Dessus : u vers +X, haut de la texture vers −Z
        FaceCube {
            normale: Vec3::Y,
            coins: [v(-h, h, -h), v(h, h, -h), v(h, h, h), v(-h, h, h)],
            groupe: GroupeFaces::Haut,
        },
        // Dessous : u vers +X, haut de la texture vers +Z
        FaceCube {
            normale: Vec3::NEG_Y,
            coins: [v(-h, -h, h), v(h, -h, h), v(h, -h, -h), v(-h, -h, -h)],
            groupe: GroupeFaces::Bas,
        },
    ]
}

const UV_PLEIN: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

/// Couleur linéaire d'une couleur 0xRRGGBB (sRGB, comme les couleurs three.js).
pub fn couleur_lineaire(hex: u32) -> [f32; 4] {
    let c = Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8).to_linear();
    [c.red, c.green, c.blue, 1.0]
}

fn case(p: [f32; 3]) -> IVec3 {
    IVec3::new(
        p[0].floor() as i32,
        p[1].floor() as i32,
        p[2].floor() as i32,
    )
}

/// Maillages de toute l'arène. `a_texture(chemin)` dit si la texture a pu être chargée.
pub fn maillages_arene(
    d: &Donnees,
    monde: &MondeGenere,
    a_texture: impl Fn(&str) -> bool,
) -> Vec<MaillageBloc> {
    // Blocs pleins opaques : ils cachent les faces qui les touchent
    let mut opaques = HashSet::new();
    for (id, positions) in &monde.blocs {
        let b = &d.blocs[id.as_str()];
        if b.forme.unwrap_or(FormeBloc::Cube) == FormeBloc::Cube && !b.decoupe {
            opaques.extend(positions.iter().map(|p| case(*p)));
        }
    }
    let faces = faces_cube();
    let mut sortie = Vec::new();
    for (id, positions) in &monde.blocs {
        let index = d.blocs.get_index_of(id.as_str()).expect("bloc validé");
        let b = &d.blocs[index];
        let forme = b.forme.unwrap_or(FormeBloc::Cube);
        let mode = match b.ondule {
            Some(Ondulation::Feuillage) => VENT_FEUILLAGE,
            Some(Ondulation::Plante) => VENT_PLANTE,
            None => 0.0,
        };
        // Même liquide autour : face commune invisible
        let memes: HashSet<IVec3> = if forme == FormeBloc::Liquide {
            positions.iter().map(|p| case(*p)).collect()
        } else {
            HashSet::new()
        };
        let groupes: &[GroupeFaces] = if forme == FormeBloc::Croix {
            &[GroupeFaces::Cotes]
        } else {
            &[GroupeFaces::Cotes, GroupeFaces::Haut, GroupeFaces::Bas]
        };
        for &groupe in groupes {
            let texture = a_texture(groupe.texture(b));
            let teinte = if texture {
                groupe.teinte(b).map_or([1.0; 4], couleur_lineaire)
            } else {
                couleur_lineaire(b.couleur_secours)
            };
            let mut m = MaillageBloc {
                bloc: index,
                groupe: Some(groupe),
                ..default()
            };
            for p in positions {
                let centre = Vec3::from_array(*p);
                let c = case(*p);
                let quarts = if b.rotation_aleatoire {
                    (hachage3(c.z, c.x, c.y) * 4.0).floor() as i32
                } else {
                    0
                };
                let rotation = Quat::from_rotation_y(quarts as f32 * std::f32::consts::FRAC_PI_2);
                let vent = [centre.x, centre.y - 0.5, centre.z, mode];
                let mut couleur = teinte;
                if !texture && (c.x + c.y + c.z) & 1 == 0 {
                    for k in &mut couleur[..3] {
                        *k *= CONTRASTE_DAMIER;
                    }
                }
                if forme == FormeBloc::Croix {
                    ajouter_croix(&mut m, centre, rotation, couleur, vent);
                    continue;
                }
                // Liquide : abaissé, posé sur le bas de sa case
                let (echelle, decalage) = if forme == FormeBloc::Liquide {
                    (HAUTEUR_LIQUIDE, -(1.0 - HAUTEUR_LIQUIDE) / 2.0)
                } else {
                    (1.0, 0.0)
                };
                for f in faces.iter().filter(|f| f.groupe == groupe) {
                    let normale = rotation * f.normale;
                    let voisin = c + normale.round().as_ivec3();
                    if opaques.contains(&voisin) || memes.contains(&voisin) {
                        continue;
                    }
                    let coins = f.coins.map(|k| {
                        let k = rotation * Vec3::new(k.x, k.y * echelle, k.z);
                        centre + k + Vec3::Y * decalage
                    });
                    m.quad(coins, normale, couleur, UV_PLEIN, vent);
                }
            }
            if !m.indices.is_empty() {
                sortie.push(m);
            }
        }
    }
    sortie
}

/// Deux plans verticaux croisés en diagonale du bloc (fleurs, herbes, toiles).
fn ajouter_croix(
    m: &mut MaillageBloc,
    centre: Vec3,
    rotation: Quat,
    couleur: [f32; 4],
    vent: [f32; 4],
) {
    let d = DEMI_CROIX;
    let plans = [
        [
            Vec3::new(-d, 0.5, -d),
            Vec3::new(d, 0.5, d),
            Vec3::new(d, -0.5, d),
            Vec3::new(-d, -0.5, -d),
        ],
        [
            Vec3::new(-d, 0.5, d),
            Vec3::new(d, 0.5, -d),
            Vec3::new(d, -0.5, -d),
            Vec3::new(-d, -0.5, d),
        ],
    ];
    for coins in plans {
        let normale = rotation * (coins[1] - coins[0]).cross(coins[3] - coins[0]).normalize();
        m.quad(
            coins.map(|k| centre + rotation * k),
            normale,
            couleur,
            UV_PLEIN,
            vent,
        );
    }
}

/// Terrain décoratif hors arène : le dessus de son bloc répété (une texture par bloc),
/// assombri, sur un grand carré troué à la taille de l'arène, au niveau du sol.
pub fn maillage_exterieur(
    d: &Donnees,
    def: &Arene,
    a_texture: impl Fn(&str) -> bool,
) -> Option<MaillageBloc> {
    let ext = def.exterieur.as_ref()?;
    let index = d.blocs.get_index_of(ext.bloc.as_str())?;
    let b = &d.blocs[index];
    let base = if a_texture(&b.faces.haut) {
        b.teinte_haut.map_or([1.0; 4], couleur_lineaire)
    } else {
        couleur_lineaire(b.couleur_secours)
    };
    let couleur = [
        base[0] * ext.assombrissement,
        base[1] * ext.assombrissement,
        base[2] * ext.assombrissement,
        1.0,
    ];
    let (t, ax, az) = (ext.taille / 2.0, def.largeur / 2.0, def.profondeur / 2.0);
    let mut m = MaillageBloc {
        bloc: index,
        groupe: Some(GroupeFaces::Haut),
        ..default()
    };
    // Quatre bandes autour de l'arène : (x0, z0, x1, z1)
    for [x0, z0, x1, z1] in [
        [-t, -t, t, -az],
        [-t, az, t, t],
        [-t, -az, -ax, az],
        [ax, -az, t, az],
    ] {
        let coins = [
            Vec3::new(x0, 0.0, z0),
            Vec3::new(x1, 0.0, z0),
            Vec3::new(x1, 0.0, z1),
            Vec3::new(x0, 0.0, z1),
        ];
        let uvs = coins.map(|c| [c.x, c.z]);
        m.quad(coins, Vec3::Y, couleur, uvs, [0.0; 4]);
    }
    Some(m)
}

#[cfg(test)]
mod tests;
