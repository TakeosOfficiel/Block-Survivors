//! Essais de la découpe d'un monde entier (région écrite à la main).

use super::*;
use crate::minecraft::schematique::lire_schematique;
use flate2::write::ZlibEncoder;

fn ecrire_gz(c: &Compound) -> Vec<u8> {
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(&ecrire_nbt("", c)).unwrap();
    gz.finish().unwrap()
}

/// Tronçon (0, 0) de la 1.18 : sol de pierre en y = 64, panneau « depart » au milieu.
fn region() -> Vec<u8> {
    let etat = |n: &str| Nbt::Compound(Compound(vec![("Name".into(), Nbt::Chaine(n.into()))]));
    // 4 bits par bloc : couche y = 0 de la section (256 blocs) en pierre (1)
    let mut longs = vec![0i64; 256];
    for l in longs.iter_mut().take(16) {
        *l = 0x1111_1111_1111_1111;
    }
    let section = Compound(vec![
        ("Y".into(), Nbt::Octet(4)),
        (
            "block_states".into(),
            Nbt::Compound(Compound(vec![
                (
                    "palette".into(),
                    Nbt::Liste(vec![etat("minecraft:air"), etat("minecraft:stone")]),
                ),
                ("data".into(), Nbt::Longs(longs)),
            ])),
        ),
        (
            "biomes".into(),
            Nbt::Compound(Compound(vec![(
                "palette".into(),
                Nbt::Liste(vec![Nbt::Chaine("minecraft:desert".into())]),
            )])),
        ),
    ]);
    let panneau = Compound(vec![
        ("id".into(), Nbt::Chaine("minecraft:sign".into())),
        ("x".into(), Nbt::Entier(8)),
        ("y".into(), Nbt::Entier(65)),
        ("z".into(), Nbt::Entier(8)),
        ("front_text".into(), Nbt::Chaine("depart".into())),
    ]);
    let troncon = Compound(vec![
        ("DataVersion".into(), Nbt::Entier(3953)),
        ("sections".into(), Nbt::Liste(vec![Nbt::Compound(section)])),
        (
            "block_entities".into(),
            Nbt::Liste(vec![Nbt::Compound(panneau)]),
        ),
    ]);
    let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
    z.write_all(&ecrire_nbt("", &troncon)).unwrap();
    let donnees = z.finish().unwrap();
    let mut r = vec![0u8; 8192];
    r[0..4].copy_from_slice(&[0, 0, 2, 1]);
    r.extend((donnees.len() as u32 + 1).to_be_bytes());
    r.push(2);
    r.extend(donnees);
    r.resize(r.len().div_ceil(4096) * 4096, 0);
    r
}

#[test]
fn monde_decoupe_autour_du_depart() {
    let dossier = std::env::temp_dir().join(format!("bs-monde-{}", std::process::id()));
    std::fs::create_dir_all(dossier.join("region")).unwrap();
    std::fs::write(dossier.join("level.dat"), ecrire_gz(&Compound::default())).unwrap();
    std::fs::write(dossier.join("region/r.0.0.mca"), region()).unwrap();
    assert!(est_un_monde(&dossier));
    let s = lire_schematique(&extraire_monde(&dossier).unwrap()).unwrap();
    std::fs::remove_dir_all(&dossier).unwrap();
    assert_eq!((s.largeur, s.hauteur, s.profondeur), (96, 64, 96));
    // Départ en (8, 65, 8) → (48, 16, 48) ; sol de pierre juste dessous
    let nom = |x, y, z| s.palette[s.bloc(x, y, z).unwrap() as usize].as_str();
    assert_eq!(nom(48, 15, 48), "stone");
    assert_eq!(nom(40, 15, 40), "stone");
    assert_eq!(nom(20, 15, 20), "air");
    assert_eq!(s.panneaux.len(), 1);
    assert_eq!((s.panneaux[0].x, s.panneaux[0].y), (48, 16));
    assert_eq!(s.biome.as_deref(), Some("desert"));
    assert_eq!(s.version_donnees, 3953);
}
