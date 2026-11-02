//! Vérification du manifeste signé : chaque attaque du plan (section 5) est refusée.

use bs_maj::manifeste::{
    Manifeste, Paquet, Refus, cle_privee, cle_publique, nouvelle_cle, signer, verifier,
};

const MAINTENANT: u64 = 1_800_000_000;

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
    let e = signer(&manifeste(3), &privee);
    assert_eq!(
        verifier(e.as_bytes(), &[publique], 3, MAINTENANT),
        Ok(manifeste(3))
    );
}

#[test]
fn cle_de_secours_acceptee() {
    let (_, ci) = cles();
    let (secours_privee, secours) = cles();
    let e = signer(&manifeste(3), &secours_privee);
    assert!(verifier(e.as_bytes(), &[ci, secours], 0, MAINTENANT).is_ok());
}

#[test]
fn falsifie_refuse() {
    let (privee, publique) = cles();
    let e = signer(&manifeste(3), &privee).replace("6035", "6036");
    assert_eq!(
        verifier(e.as_bytes(), &[publique], 0, MAINTENANT),
        Err(Refus::Signature)
    );
}

#[test]
fn autre_cle_refusee() {
    let (_, publique) = cles();
    let (autre, _) = cles();
    let e = signer(&manifeste(3), &autre);
    assert_eq!(
        verifier(e.as_bytes(), &[publique], 0, MAINTENANT),
        Err(Refus::Signature)
    );
}

#[test]
fn ancien_refuse() {
    let (privee, publique) = cles();
    let e = signer(&manifeste(2), &privee);
    assert_eq!(
        verifier(e.as_bytes(), &[publique], 3, MAINTENANT),
        Err(Refus::Ancien {
            serie: 2,
            derniere: 3
        })
    );
}

#[test]
fn expire_refuse() {
    let (privee, publique) = cles();
    let e = signer(&manifeste(3), &privee);
    assert_eq!(
        verifier(e.as_bytes(), &[publique], 0, MAINTENANT + 86_400),
        Err(Refus::Expire)
    );
}

#[test]
fn illisible_refuse() {
    let (_, publique) = cles();
    assert_eq!(
        verifier(b"<html>404</html>", &[publique], 0, MAINTENANT),
        Err(Refus::Illisible)
    );
    assert_eq!(
        verifier(b"", &[publique], 0, MAINTENANT),
        Err(Refus::Illisible)
    );
}
