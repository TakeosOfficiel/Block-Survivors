//! Tests des maillages de l'arène : faces cachées entre blocs pleins et entre cases
//! d'un même liquide (sans carte graphique).

use super::*;
use bs_monde::generer_monde;

fn monde_de(d: &Donnees, blocs: Vec<(&str, Vec<[f32; 3]>)>) -> MondeGenere {
    let mut m = generer_monde(d, &d.arenes.arenes[0]);
    m.blocs = blocs
        .into_iter()
        .map(|(id, p)| (id.to_owned(), p))
        .collect();
    m
}

/// Nombre de faces (quads) de chaque maillage.
fn faces(liste: &[MaillageBloc]) -> usize {
    liste.iter().map(|m| m.indices.len() / 6).sum()
}

#[test]
fn deux_blocs_pleins_cachent_leur_face_commune() {
    let d = Donnees::embarquees().expect("données");
    let monde = monde_de(
        &d,
        vec![("cobblestone", vec![[0.0, 0.5, 0.0], [1.0, 0.5, 0.0]])],
    );
    let liste = maillages_arene(&d, &monde, |_| true);
    assert_eq!(faces(&liste), 10);
    // Côtés, dessus, dessous : un maillage par groupe
    assert_eq!(liste.len(), 3);
    for m in &liste {
        assert_eq!(m.positions.len(), m.faces() * 4);
    }
}

#[test]
fn un_liquide_cache_ses_faces_internes_et_s_abaisse() {
    let d = Donnees::embarquees().expect("données");
    let monde = monde_de(
        &d,
        vec![("water", vec![[0.0, -0.5, 0.0], [0.0, -0.5, 1.0]])],
    );
    let liste = maillages_arene(&d, &monde, |_| true);
    assert_eq!(faces(&liste), 10);
    let haut = liste
        .iter()
        .flat_map(|m| m.positions.iter())
        .map(|p| p[1])
        .fold(f32::MIN, f32::max);
    assert!(
        (haut - (-1.0 + HAUTEUR_LIQUIDE)).abs() < 1e-5,
        "dessus de l'eau à {haut}"
    );
}

#[test]
fn sans_texture_la_couleur_de_secours_fait_un_damier() {
    let d = Donnees::embarquees().expect("données");
    let monde = monde_de(
        &d,
        vec![("cobblestone", vec![[0.0, 0.5, 0.0], [1.0, 0.5, 0.0]])],
    );
    let liste = maillages_arene(&d, &monde, |_| false);
    let haut = liste
        .iter()
        .find(|m| m.groupe == Some(GroupeFaces::Haut))
        .expect("dessus");
    let premiere = haut.couleurs[0][0];
    let seconde = haut.couleurs[4][0];
    assert!(
        (premiere / seconde - CONTRASTE_DAMIER).abs() < 1e-5
            || (seconde / premiere - CONTRASTE_DAMIER).abs() < 1e-5
    );
}
