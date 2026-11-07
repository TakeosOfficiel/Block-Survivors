//! Lumière des blocs, comme dans Minecraft : chaque source (torche, lanterne, lave,
//! glowstone…) éclaire jusqu'à 15 blocs autour d'elle, un niveau de moins par bloc
//! traversé ; les blocs pleins opaques l'arrêtent (pas de lumière à travers les murs).
//! Calculée une fois au chargement, pour l'arène générée comme pour une map importée.

use std::collections::VecDeque;

use bs_donnees::Donnees;
use bs_donnees::blocs::FormeBloc;

use crate::MondeGenere;
use crate::minecraft::modeles::BlocCuit;
use crate::minecraft::schematique::Schematique;

/// Niveau d'une source la plus forte.
pub const NIVEAU_MAX: u8 = 15;

/// Niveau émis par un bloc (nom Minecraft ou id des données, qui le reprend), selon
/// ses propriétés (`lit`, `candles`…).
pub fn emission<'a>(nom: &str, propriete: impl Fn(&str) -> Option<&'a str>) -> u8 {
    let allume = || propriete("lit") != Some("false");
    match nom {
        "glowstone"
        | "sea_lantern"
        | "lantern"
        | "jack_o_lantern"
        | "shroomlight"
        | "lava"
        | "beacon"
        | "conduit"
        | "end_gateway"
        | "fire"
        | "ochre_froglight"
        | "verdant_froglight"
        | "pearlescent_froglight"
        | "respawn_anchor" => NIVEAU_MAX,
        "torch" | "wall_torch" | "end_rod" | "copper_torch" | "copper_wall_torch" => 14,
        "campfire" if allume() => NIVEAU_MAX,
        "redstone_lamp" if propriete("lit") == Some("true") => NIVEAU_MAX,
        "furnace" | "blast_furnace" | "smoker" if propriete("lit") == Some("true") => 13,
        "cave_vines" | "cave_vines_plant" if propriete("berries") == Some("true") => 14,
        "nether_portal" => 11,
        "soul_torch" | "soul_wall_torch" | "soul_lantern" | "soul_fire" | "crying_obsidian" => 10,
        "soul_campfire" if allume() => 10,
        "glow_lichen"
        | "enchanting_table"
        | "ender_chest"
        | "redstone_torch"
        | "redstone_wall_torch"
            if allume() =>
        {
            7
        }
        "amethyst_cluster" | "sea_pickle" => 5,
        "magma" | "magma_block" => 3,
        _ if nom.ends_with("candle") && propriete("lit") == Some("true") => {
            3 * propriete("candles")
                .and_then(|n| n.parse().ok())
                .unwrap_or(1u8)
        }
        _ => 0,
    }
}

/// Niveaux de lumière d'une boîte de blocs (x le plus rapide, puis y, puis z).
#[derive(Debug, Clone, Default)]
pub struct VolumeLumiere {
    /// Coin du bloc (0, 0, 0) de la boîte, en coordonnées du jeu.
    pub origine: [i32; 3],
    pub taille: [usize; 3],
    pub niveaux: Vec<u8>,
}

impl VolumeLumiere {
    fn index(&self, p: [i32; 3]) -> Option<usize> {
        let mut i = 0;
        for k in (0..3).rev() {
            let c = p[k] - self.origine[k];
            if c < 0 || c as usize >= self.taille[k] {
                return None;
            }
            i = i * self.taille[k] + c as usize;
        }
        Some(i)
    }

    /// Propage la lumière des `sources` (bloc, niveau) ; `opaque[i]` : le bloc d'index
    /// `i` arrête la lumière (une source opaque, comme la glowstone, garde la sienne).
    pub fn propager(
        origine: [i32; 3],
        taille: [usize; 3],
        sources: impl IntoIterator<Item = ([i32; 3], u8)>,
        opaque: &[bool],
    ) -> Self {
        let mut v = Self {
            origine,
            taille,
            niveaux: vec![0; taille[0] * taille[1] * taille[2]],
        };
        let mut file = VecDeque::new();
        for (p, n) in sources {
            if let Some(i) = v.index(p)
                && n > v.niveaux[i]
            {
                v.niveaux[i] = n.min(NIVEAU_MAX);
                file.push_back(i);
            }
        }
        let pas = [1, taille[0], taille[0] * taille[1]];
        while let Some(i) = file.pop_front() {
            let n = v.niveaux[i];
            if n <= 1 {
                continue;
            }
            let mut c = [i % taille[0], i / taille[0] % taille[1], 0];
            c[2] = i / pas[2];
            for k in 0..3 {
                for bord in [false, true] {
                    let j = match bord {
                        false if c[k] > 0 => i - pas[k],
                        true if c[k] + 1 < taille[k] => i + pas[k],
                        _ => continue,
                    };
                    if !opaque[j] && v.niveaux[j] < n - 1 {
                        v.niveaux[j] = n - 1;
                        file.push_back(j);
                    }
                }
            }
        }
        v
    }

    /// Niveau du bloc `p` (0 hors de la boîte).
    pub fn niveau(&self, p: [i32; 3]) -> u8 {
        self.index(p).map_or(0, |i| self.niveaux[i])
    }

    /// Aucune lumière nulle part (rien à envoyer au rendu).
    pub fn sombre(&self) -> bool {
        self.niveaux.iter().all(|n| *n == 0)
    }
}

/// Lumière d'une arène générée (blocs repérés par leur centre ; une case de marge
/// autour, et de quoi éclairer au-dessus du plus haut bloc).
pub fn volume_du_monde(d: &Donnees, monde: &MondeGenere) -> VolumeLumiere {
    let case = |p: &[f32; 3]| p.map(|c| c.floor() as i32);
    let (mut min, mut max) = ([i32::MAX; 3], [i32::MIN; 3]);
    for p in monde.blocs.iter().flat_map(|(_, ps)| ps) {
        let c = case(p);
        for k in 0..3 {
            min[k] = min[k].min(c[k]);
            max[k] = max[k].max(c[k]);
        }
    }
    if min[0] > max[0] {
        return VolumeLumiere::default();
    }
    let origine = [min[0] - 1, min[1] - 1, min[2] - 1];
    let taille = [
        (max[0] - origine[0] + 2) as usize,
        (max[1] - origine[1] + 4) as usize,
        (max[2] - origine[2] + 2) as usize,
    ];
    // Sert seulement à calculer les index
    let boite = VolumeLumiere {
        origine,
        taille,
        niveaux: Vec::new(),
    };
    let mut opaque = vec![false; taille[0] * taille[1] * taille[2]];
    let mut sources = Vec::new();
    for (id, positions) in &monde.blocs {
        let b = &d.blocs[id.as_str()];
        let plein = b.forme.unwrap_or(FormeBloc::Cube) == FormeBloc::Cube && !b.decoupe;
        let n = emission(id, |_| None);
        for p in positions {
            let c = case(p);
            if plein && let Some(i) = boite.index(c) {
                opaque[i] = true;
            }
            if n > 0 {
                sources.push((c, n));
            }
        }
    }
    VolumeLumiere::propager(origine, taille, sources, &opaque)
}

/// Lumière d'une map importée, en cases de la schématique ; `opaque[k]` : l'état `k`
/// de la palette arrête la lumière (voir `maillage::opaques`).
pub fn volume_de_schematique(
    s: &Schematique,
    blocs: &[BlocCuit],
    opaque: &[bool],
) -> VolumeLumiere {
    let taille = [s.largeur, s.hauteur + 2, s.profondeur];
    let niveaux: Vec<u8> = blocs
        .iter()
        .map(|b| emission(&b.nom, |k| b.propriete(k)))
        .collect();
    let mut murs = vec![false; taille[0] * taille[1] * taille[2]];
    let mut sources = Vec::new();
    for y in 0..s.hauteur {
        for z in 0..s.profondeur {
            for x in 0..s.largeur {
                let v = s.blocs[s.index(x, y, z)] as usize;
                murs[x + taille[0] * (y + taille[1] * z)] = opaque[v];
                if niveaux[v] > 0 {
                    sources.push(([x as i32, y as i32, z as i32], niveaux[v]));
                }
            }
        }
    }
    VolumeLumiere::propager([0; 3], taille, sources, &murs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_lumiere_baisse_et_les_murs_l_arretent() {
        // Couloir de 9 blocs, mur plein en x = 6
        let taille = [9, 1, 1];
        let mut opaque = vec![false; 9];
        opaque[6] = true;
        let v = VolumeLumiere::propager([0; 3], taille, [([2, 0, 0], 14)], &opaque);
        assert_eq!(v.niveau([2, 0, 0]), 14);
        assert_eq!(v.niveau([0, 0, 0]), 12);
        assert_eq!(v.niveau([5, 0, 0]), 11);
        assert_eq!(v.niveau([6, 0, 0]), 0);
        assert_eq!(v.niveau([7, 0, 0]), 0);
        assert_eq!(v.niveau([40, 0, 0]), 0);
    }

    #[test]
    fn niveaux_des_sources() {
        let aucune = |_: &str| None;
        assert_eq!(emission("torch", aucune), 14);
        assert_eq!(emission("stone", aucune), 0);
        assert_eq!(emission("redstone_lamp", aucune), 0);
        assert_eq!(
            emission("redstone_lamp", |k| (k == "lit").then_some("true")),
            15
        );
        let bougies = |k: &str| match k {
            "lit" => Some("true"),
            "candles" => Some("3"),
            _ => None,
        };
        assert_eq!(emission("red_candle", bougies), 9);
    }
}
