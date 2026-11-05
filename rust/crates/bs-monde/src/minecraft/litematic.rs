//! Schématiques Litematica (.litematic) : une ou plusieurs régions, chacune avec sa
//! palette (`BlockStatePalette`) et ses blocs tassés dans des longs (`BlockStates`,
//! au moins 2 bits par bloc, une valeur pouvant chevaucher deux longs). Les régions
//! sont réunies dans un seul volume, du coin le plus bas au plus haut ; les panneaux
//! viennent des `TileEntities` posées sur un bloc de panneau.

use super::nbt::{Compound, Nbt};
use super::schematique::{Palette, Panneau, Schematique, etat_de, texte_de};

/// Volume maximal accepté (blocs), pour refuser un fichier aberrant avant d'allouer.
const VOLUME_MAX: usize = 1 << 27;

/// Région lue : coin minimal (coordonnées du fichier), taille et contenu.
struct Region<'a> {
    min: [i64; 3],
    taille: [usize; 3],
    nbt: &'a Compound,
}

fn vecteur(c: Option<&Compound>, quoi: &str) -> Result<[i64; 3], String> {
    let c = c.ok_or_else(|| format!("Litematica invalide : {quoi} manquant."))?;
    let v = ["x", "y", "z"].map(|k| c.get(k).and_then(Nbt::entier));
    match v {
        [Some(x), Some(y), Some(z)] => Ok([x, y, z]),
        _ => Err(format!("Litematica invalide : {quoi} incomplet.")),
    }
}

fn regions(racine: &Compound) -> Result<Vec<Region<'_>>, String> {
    let mut r = Vec::new();
    for (_, n) in &racine
        .compound("Regions")
        .ok_or("Litematica sans région.")?
        .0
    {
        let Nbt::Compound(nbt) = n else { continue };
        let pos = vecteur(nbt.compound("Position"), "Position")?;
        let taille = vecteur(nbt.compound("Size"), "Size")?;
        // Taille négative : la région s'étend vers les coordonnées décroissantes
        let min = [0, 1, 2].map(|k| pos[k] + if taille[k] < 0 { taille[k] + 1 } else { 0 });
        let taille = taille.map(|t| t.unsigned_abs() as usize);
        if taille.contains(&0) {
            continue;
        }
        r.push(Region { min, taille, nbt });
    }
    if r.is_empty() {
        return Err("Litematica sans région.".into());
    }
    Ok(r)
}

/// Valeur n° `i` de `bits` bits dans les longs tassés.
fn valeur_tassee(longs: &[i64], i: usize, bits: usize) -> usize {
    let bit = i * bits;
    let (k, decalage) = (bit / 64, bit % 64);
    let Some(&bas) = longs.get(k) else { return 0 };
    let mut v = (bas as u64) >> decalage;
    if decalage + bits > 64 {
        v |= (longs.get(k + 1).copied().unwrap_or(0) as u64) << (64 - decalage);
    }
    (v & ((1u64 << bits) - 1)) as usize
}

pub(super) fn lire_litematic(racine: &Compound) -> Result<Schematique, String> {
    let regions = regions(racine)?;
    let mut min = [i64::MAX; 3];
    let mut max = [i64::MIN; 3];
    for r in &regions {
        for k in 0..3 {
            min[k] = min[k].min(r.min[k]);
            max[k] = max[k].max(r.min[k] + r.taille[k] as i64);
        }
    }
    let [largeur, hauteur, profondeur] = [0, 1, 2].map(|k| (max[k] - min[k]) as usize);
    if largeur * hauteur * profondeur > VOLUME_MAX {
        return Err(format!(
            "Litematica trop grand ({largeur} × {hauteur} × {profondeur} blocs)."
        ));
    }
    let mut palette = Palette::nouvelle();
    let mut blocs = vec![0u16; largeur * hauteur * profondeur];
    let mut panneaux = Vec::new();
    for r in &regions {
        let passage: Vec<u16> = r
            .nbt
            .get("BlockStatePalette")
            .and_then(Nbt::liste)
            .unwrap_or(&[])
            .iter()
            .map(|p| match p {
                Nbt::Compound(p) => palette.ajouter(&etat_de(p)),
                _ => 0,
            })
            .collect();
        let longs: &[i64] = match r.nbt.get("BlockStates") {
            Some(Nbt::Longs(l)) => l,
            _ => return Err("Litematica invalide : BlockStates manquant.".into()),
        };
        let bits = (usize::BITS - passage.len().saturating_sub(1).leading_zeros()).max(2) as usize;
        let [tx, ty, tz] = r.taille;
        let o = [0, 1, 2].map(|k| (r.min[k] - min[k]) as usize);
        let mut i = 0;
        for y in 0..ty {
            for z in 0..tz {
                for x in 0..tx {
                    let b = passage
                        .get(valeur_tassee(longs, i, bits))
                        .copied()
                        .unwrap_or(0);
                    i += 1;
                    // Une région posée sur une autre ne remplace que par un vrai bloc
                    if b != 0 {
                        blocs[((y + o[1]) * profondeur + z + o[2]) * largeur + x + o[0]] = b;
                    }
                }
            }
        }
        for e in r
            .nbt
            .get("TileEntities")
            .and_then(Nbt::liste)
            .unwrap_or(&[])
        {
            let Nbt::Compound(e) = e else { continue };
            let [Some(x), Some(y), Some(z)] =
                ["x", "y", "z"].map(|k| e.get(k).and_then(Nbt::entier).map(|v| v as usize))
            else {
                continue;
            };
            if x >= tx || y >= ty || z >= tz {
                continue;
            }
            let (x, y, z) = (x + o[0], y + o[1], z + o[2]);
            let b = blocs[(y * profondeur + z) * largeur + x] as usize;
            if !palette.etats[b].contains("sign") {
                continue;
            }
            panneaux.push(Panneau {
                x: x as i32,
                y: y as i32,
                z: z as i32,
                texte: texte_de(&Nbt::Compound(e.clone())),
            });
        }
    }
    Ok(Schematique {
        largeur,
        hauteur,
        profondeur,
        palette: palette.etats,
        blocs,
        panneaux,
        version_donnees: racine
            .get("MinecraftDataVersion")
            .and_then(Nbt::entier)
            .unwrap_or(0),
        biome: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tasser(valeurs: &[usize], bits: usize) -> Vec<i64> {
        let mut longs = vec![0u64; (valeurs.len() * bits).div_ceil(64)];
        for (i, &v) in valeurs.iter().enumerate() {
            let bit = i * bits;
            longs[bit / 64] |= (v as u64) << (bit % 64);
            if bit % 64 + bits > 64 {
                longs[bit / 64 + 1] |= (v as u64) >> (64 - bit % 64);
            }
        }
        longs.into_iter().map(|l| l as i64).collect()
    }

    fn vec3(x: i32, y: i32, z: i32) -> Nbt {
        let c = |v| Nbt::Entier(v);
        Nbt::Compound(Compound(vec![
            ("x".into(), c(x)),
            ("y".into(), c(y)),
            ("z".into(), c(z)),
        ]))
    }

    fn etat(nom: &str) -> Nbt {
        Nbt::Compound(Compound(vec![("Name".into(), Nbt::Chaine(nom.into()))]))
    }

    fn region(pos: Nbt, taille: Nbt, noms: &[&str], valeurs: &[usize], bits: usize) -> Nbt {
        Nbt::Compound(Compound(vec![
            ("Position".into(), pos),
            ("Size".into(), taille),
            (
                "BlockStatePalette".into(),
                Nbt::Liste(noms.iter().map(|n| etat(n)).collect()),
            ),
            ("BlockStates".into(), Nbt::Longs(tasser(valeurs, bits))),
        ]))
    }

    /// Deux régions (l'une de taille négative), 3 bits par bloc avec une valeur à cheval
    /// sur deux longs, et un panneau.
    #[test]
    fn deux_regions_reunies() {
        let noms = [
            "minecraft:air",
            "minecraft:stone",
            "minecraft:dirt",
            "minecraft:oak_sign",
            "minecraft:glass",
        ];
        // Région A : 6 × 2 × 2 = 24 blocs, le 22e (bits 63 à 65) est du verre
        let mut a = vec![1; 24];
        a[21] = 4;
        a[0] = 3;
        let mut ra = region(vec3(0, 0, 0), vec3(6, 2, 2), &noms, &a, 3);
        if let Nbt::Compound(c) = &mut ra {
            let mut texte = Compound(vec![("x".into(), Nbt::Entier(0))]);
            texte.0.push(("y".into(), Nbt::Entier(0)));
            texte.0.push(("z".into(), Nbt::Entier(0)));
            texte
                .0
                .push(("front_text".into(), Nbt::Chaine("depart".into())));
            c.0.push((
                "TileEntities".into(),
                Nbt::Liste(vec![Nbt::Compound(texte)]),
            ));
        }
        // Région B : Position x = 8, Size x = -2 → occupe x = 7 et 8
        let rb = region(vec3(8, 0, 0), vec3(-2, 1, 1), &noms[..3], &[2, 2], 2);
        let racine = Compound(vec![
            ("MinecraftDataVersion".into(), Nbt::Entier(3953)),
            (
                "Regions".into(),
                Nbt::Compound(Compound(vec![("a".into(), ra), ("b".into(), rb)])),
            ),
        ]);
        let s = lire_litematic(&racine).unwrap();
        assert_eq!((s.largeur, s.hauteur, s.profondeur), (9, 2, 2));
        let nom = |x, y, z| s.palette[s.bloc(x, y, z).unwrap() as usize].as_str();
        assert_eq!(nom(0, 0, 0), "oak_sign");
        assert_eq!(nom(1, 0, 0), "stone");
        // Index 21 = (y 1, z 1, x 3)
        assert_eq!(nom(3, 1, 1), "glass");
        assert_eq!(nom(6, 0, 0), "air");
        assert_eq!(nom(7, 0, 0), "dirt");
        assert_eq!(nom(8, 0, 0), "dirt");
        assert_eq!(s.panneaux.len(), 1);
        assert_eq!(s.panneaux[0].texte, "depart");
        assert_eq!(s.version_donnees, 3953);
    }
}
