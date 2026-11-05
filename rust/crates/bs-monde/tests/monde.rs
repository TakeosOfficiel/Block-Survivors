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

/// Un mur coupe la carte avec une fente d'une case tout près et une ouverture de trois
/// cases loin : un ennemi de taille 3 fait le détour, un petit passe par la fente ; sans
/// l'ouverture, le gros se faufile par la fente.
#[test]
fn gros_ennemi_par_le_passage_large() {
    let chemin = |taille: usize, large: bool| {
        let mut carte = bs_monde::CarteMonde::nouvelle(30, 30);
        let l = carte.largeur;
        for iz in 0..30 {
            let ouvert = iz == 15 || (large && (3..6).contains(&iz));
            if !ouvert {
                carte.drapeaux[iz * l + 15] |= BLOQUE_MARCHE;
            }
        }
        let mut flux = ChampDeFlux::nouveau(&carte);
        // Joueur à droite du mur, ennemi à gauche, à la hauteur de la fente
        flux.maj(&carte, 8.5, 0.5);
        let (mut x, mut z, mut z_min) = (-8.5_f32, 0.5_f32, 0.5_f32);
        for _ in 0..2000 {
            let Some((dx, dz)) = flux.direction_taille(&carte, x, z, taille) else {
                break;
            };
            x += dx * 0.25;
            z += dz * 0.25;
            z_min = z_min.min(z);
        }
        assert_eq!(carte.cellule(x, z), carte.cellule(8.5, 0.5));
        z_min
    };
    assert!(chemin(1, true) > -1.0, "petit : par la fente");
    assert!(chemin(3, true) < -8.0, "gros : par l'ouverture");
    assert!(
        chemin(3, false) > -1.0,
        "gros sans ouverture : par la fente"
    );
}
