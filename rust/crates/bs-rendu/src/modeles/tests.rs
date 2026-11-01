//! Tests des modèles : géométrie des boîtes et des objets, rotations animées, sur les
//! données embarquées.

use bevy::math::Vec3;
use bs_donnees::Donnees;

use super::animation::{EtatAnimation, angle_chute, calculer_rotations};
use super::compilation::ModeleCompile;
use super::geometrie::{Maillage, cube, objet};

fn donnees() -> &'static Donnees {
    Box::leak(Box::new(Donnees::embarquees().expect("données")))
}

#[test]
fn boite_a_six_faces_et_uv_dans_la_texture() {
    let d = donnees();
    for (nom, modele) in d.modeles.iter() {
        for (partie, c) in &modele.parties {
            let mut m = Maillage::default();
            cube(&mut m, c, modele.taille_texture, [1.0; 4]);
            assert_eq!(m.positions.len(), 24, "{nom}/{partie}");
            assert_eq!(m.indices.len(), 36, "{nom}/{partie}");
            for uv in &m.uvs {
                assert!(
                    (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1]),
                    "{nom}/{partie} : uv {uv:?} hors texture"
                );
            }
            // Chaque triangle non dégénéré regarde du côté de sa normale
            for t in m.indices.chunks(3) {
                let p: Vec<Vec3> = t
                    .iter()
                    .map(|&i| Vec3::from_array(m.positions[i as usize]))
                    .collect();
                let n = Vec3::from_array(m.normales[t[0] as usize]);
                let aire = (p[1] - p[0]).cross(p[2] - p[0]);
                // Parties plates (ailes) : faces de tranche dégénérées
                if aire.length() > 1e-9 {
                    assert!(aire.dot(n) > 0.0, "{nom}/{partie}");
                }
            }
        }
    }
}

#[test]
fn objet_extrude_selon_ses_pixels() {
    // 2 × 2 pixels, un seul opaque : 2 grandes faces + 4 rebords
    let mut pixels = [0u8; 16];
    pixels[3] = 255;
    let m = objet(&pixels, 2, 2, 0.1);
    assert_eq!(m.positions.len(), 6 * 4);
    // Tout opaque : seuls les rebords extérieurs (2 par côté)
    let m = objet(&[255u8; 16], 2, 2, 0.1);
    assert_eq!(m.positions.len(), (2 + 8) * 4);
}

#[test]
fn rotations_de_base_puis_marche() {
    let d = donnees();
    let m = ModeleCompile::nouveau(&d.modeles["humanoide"]);
    let mut r = vec![Vec3::ZERO; m.parties.len()];
    let mut e = EtatAnimation::nouveau(0.0);
    calculer_rotations(&m, &e, &mut r);
    assert!(r.iter().all(|v| v.is_finite()));
    for _ in 0..60 {
        e.avancer(&m, 1.0 / 60.0, 4.0);
    }
    assert!(e.intensite > 0.5);
    let mut r2 = r.clone();
    calculer_rotations(&m, &e, &mut r2);
    assert_ne!(r, r2, "la marche balance les membres");
}

#[test]
fn chute_de_zero_a_quart_de_tour() {
    assert_eq!(angle_chute(0.0, 0.6), 0.0);
    let fin = angle_chute(10.0, 0.6);
    assert!((fin.abs() - std::f32::consts::FRAC_PI_2).abs() < 1e-4);
}
