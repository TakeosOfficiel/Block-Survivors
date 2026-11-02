//! Vérification du manifeste signé : chaque attaque du plan (section 5) est refusée.

use bs_maj::delegation::{Contenu, Delegation, deleguer};
use bs_maj::manifeste::{
    Manifeste, Paquet, Refus, Series, cle_privee, cle_publique, nouvelle_cle, signer, verifier,
};

const MAINTENANT: u64 = 1_800_000_000;

fn vues(manifeste: u64) -> Series {
    Series {
        manifeste,
        delegation: 0,
    }
}

fn manifeste(serie: u64) -> Manifeste {
    Manifeste {
        serie,
        expire: MAINTENANT + 86_400,
        voie: "stable".into(),
        version: "0.2.0".into(),
        version_min_coop: "0.1.0".into(),
        installeurs: Vec::new(),
        paquets: vec![Paquet {
            id: "BlockSurvivors".into(),
            version: "0.2.0".into(),
            genre: "Delta".into(),
            fichier: "BlockSurvivors-0.2.0-stable-delta.nupkg".into(),
            sha1: "AA".into(),
            sha256: "bb".into(),
            taille: 6035,
            notes: "Corrections".into(),
        }],
    }
}

fn cles() -> (ed25519_dalek::SigningKey, ed25519_dalek::VerifyingKey) {
    let (privee, publique) = nouvelle_cle();
    (
        cle_privee(&privee).unwrap(),
        cle_publique(&publique).unwrap(),
    )
}

#[test]
fn manifeste_valide_accepte() {
    let (privee, publique) = cles();
    let e = signer(&manifeste(3), &privee, None);
    assert_eq!(
        verifier(e.as_bytes(), &[publique], vues(3), MAINTENANT),
        Ok((manifeste(3), vues(3)))
    );
}

#[test]
fn cle_de_secours_acceptee() {
    let (_, ci) = cles();
    let (secours_privee, secours) = cles();
    let e = signer(&manifeste(3), &secours_privee, None);
    assert!(verifier(e.as_bytes(), &[ci, secours], vues(0), MAINTENANT).is_ok());
}

#[test]
fn falsifie_refuse() {
    let (privee, publique) = cles();
    let e = signer(&manifeste(3), &privee, None).replace("6035", "6036");
    assert_eq!(
        verifier(e.as_bytes(), &[publique], vues(0), MAINTENANT),
        Err(Refus::Signature)
    );
}

#[test]
fn autre_cle_refusee() {
    let (_, publique) = cles();
    let (autre, _) = cles();
    let e = signer(&manifeste(3), &autre, None);
    assert_eq!(
        verifier(e.as_bytes(), &[publique], vues(0), MAINTENANT),
        Err(Refus::Signature)
    );
}

#[test]
fn ancien_refuse() {
    let (privee, publique) = cles();
    let e = signer(&manifeste(2), &privee, None);
    assert_eq!(
        verifier(e.as_bytes(), &[publique], vues(3), MAINTENANT),
        Err(Refus::Ancien {
            serie: 2,
            derniere: 3
        })
    );
}

#[test]
fn expire_refuse() {
    let (privee, publique) = cles();
    let e = signer(&manifeste(3), &privee, None);
    assert_eq!(
        verifier(e.as_bytes(), &[publique], vues(0), MAINTENANT + 86_400),
        Err(Refus::Expire)
    );
}

#[test]
fn illisible_refuse() {
    let (_, publique) = cles();
    assert_eq!(
        verifier(b"<html>404</html>", &[publique], vues(0), MAINTENANT),
        Err(Refus::Illisible)
    );
    assert_eq!(
        verifier(b"", &[publique], vues(0), MAINTENANT),
        Err(Refus::Illisible)
    );
}

fn delegation(
    racine: &ed25519_dalek::SigningKey,
    ci: &ed25519_dalek::VerifyingKey,
    serie: u64,
) -> Delegation {
    use base64::Engine;
    deleguer(
        &Contenu {
            serie,
            expire: MAINTENANT + 30 * 86_400,
            cles: vec![base64::engine::general_purpose::STANDARD.encode(ci.to_bytes())],
        },
        racine,
    )
}

#[test]
fn cle_deleguee_acceptee() {
    let (racine_privee, racine) = cles();
    let (ci_privee, ci) = cles();
    let e = signer(
        &manifeste(3),
        &ci_privee,
        Some(&delegation(&racine_privee, &ci, 5)),
    );
    let (m, series) = verifier(e.as_bytes(), &[racine], vues(0), MAINTENANT).unwrap();
    assert_eq!(m, manifeste(3));
    assert_eq!(
        series,
        Series {
            manifeste: 3,
            delegation: 5
        }
    );
}

#[test]
fn cle_de_ci_sans_delegation_refusee() {
    let (_, racine) = cles();
    let (ci_privee, _) = cles();
    let e = signer(&manifeste(3), &ci_privee, None);
    assert_eq!(
        verifier(e.as_bytes(), &[racine], vues(0), MAINTENANT),
        Err(Refus::Signature)
    );
}

#[test]
fn delegation_par_une_autre_racine_refusee() {
    let (_, racine) = cles();
    let (pirate_privee, _) = cles();
    let (ci_privee, ci) = cles();
    let e = signer(
        &manifeste(3),
        &ci_privee,
        Some(&delegation(&pirate_privee, &ci, 5)),
    );
    assert_eq!(
        verifier(e.as_bytes(), &[racine], vues(0), MAINTENANT),
        Err(Refus::Signature)
    );
}

#[test]
fn delegation_revoquee_refusee() {
    // La délégation 2 a remplacé la 1 (clé de CI volée) : la 1 ne passe plus
    let (racine_privee, racine) = cles();
    let (volee_privee, volee) = cles();
    let e = signer(
        &manifeste(9),
        &volee_privee,
        Some(&delegation(&racine_privee, &volee, 1)),
    );
    let deja_vues = Series {
        manifeste: 3,
        delegation: 2,
    };
    assert_eq!(
        verifier(e.as_bytes(), &[racine], deja_vues, MAINTENANT),
        Err(Refus::DelegationAncienne {
            serie: 1,
            derniere: 2
        })
    );
}

#[test]
fn delegation_expiree_refusee() {
    let (racine_privee, racine) = cles();
    let (ci_privee, ci) = cles();
    let e = signer(
        &manifeste(3),
        &ci_privee,
        Some(&delegation(&racine_privee, &ci, 1)),
    );
    assert_eq!(
        verifier(e.as_bytes(), &[racine], vues(0), MAINTENANT + 30 * 86_400),
        Err(Refus::DelegationExpiree)
    );
}

#[test]
fn cle_deleguee_ne_signe_pas_une_delegation() {
    // Une clé de CI ne peut pas se déléguer elle-même une nouvelle clé
    let (_, racine) = cles();
    let (ci_privee, ci) = cles();
    let (autre_privee, autre) = cles();
    let _ = ci;
    let e = signer(
        &manifeste(3),
        &autre_privee,
        Some(&delegation(&ci_privee, &autre, 9)),
    );
    assert_eq!(
        verifier(e.as_bytes(), &[racine], vues(0), MAINTENANT),
        Err(Refus::Signature)
    );
}
