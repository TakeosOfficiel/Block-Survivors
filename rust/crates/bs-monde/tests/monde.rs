//! Le monde généré en Rust est celui de la version TypeScript : même empreinte de la
//! carte de collision (drapeaux, ralentissements, dégâts) pour chaque arène construite.

use bs_donnees::Donnees;
use bs_monde::{BLOQUE_MARCHE, ChampDeFlux, generer_monde};

/// Empreintes relevées avec la version TypeScript (genererMonde, FNV-1a de la carte).
const EMPREINTES: [(&str, u32); 3] = [
    ("prairie", 3_068_297_080),
    ("mine", 2_712_976_472),
    ("nether", 2_228_661_699),
];

#[test]
fn memes_cartes_qu_en_typescript() {
    let donnees = Donnees::embarquees().expect("données");
    for (id, attendue) in EMPREINTES {
        let arene = donnees.arenes.arenes.iter().find(|a| a.id == id).expect(id);
        let c = generer_monde(&donnees, arene).carte;
        let mut h: u32 = 2_166_136_261;
        let mut melanger = |v: u32| {
            h ^= v;
            h = h.wrapping_mul(16_777_619);
        };
        for i in 0..c.drapeaux.len() {
            melanger(u32::from(c.drapeaux[i]));
            melanger((f64::from(c.ralentissement[i]) * 100.0).round() as u32);
            melanger((f64::from(c.degats[i]) * 100.0).round() as u32);
        }
        assert_eq!(h, attendue, "carte de « {id} »");
    }
}

#[test]
fn le_flux_mene_au_joueur() {
    let donnees = Donnees::embarquees().expect("données");
    let arene = donnees
        .arenes
        .arenes
        .iter()
        .find(|a| a.id == "prairie")
        .unwrap();
    let carte = generer_monde(&donnees, arene).carte;
    let mut flux = ChampDeFlux::nouveau(&carte);
    flux.maj(&carte, 0.5, 0.5);
    // Depuis un coin atteignable, la descente du champ arrive sur la case du joueur
    let (mut x, mut z) = (-carte.demi_x + 2.5, -carte.demi_z + 2.5);
    assert!(carte.drapeaux[carte.cellule(x, z).unwrap()] & BLOQUE_MARCHE == 0);
    for _ in 0..2000 {
        let Some((dx, dz)) = flux.direction(&carte, x, z) else {
            break;
        };
        x += dx * 0.25;
        z += dz * 0.25;
    }
    assert_eq!(carte.cellule(x, z), carte.cellule(0.5, 0.5));
}
