//! Clés publiques qui signent les manifestes : celle de la CI et celle de secours (gardée
//! hors ligne, seule à pouvoir remplacer la première). Elles sont écrites dans le build par
//! la variable `BS_CLES_MAJ` (base64, séparées par des virgules), que la CI tire des
//! variables du dépôt (`bs-signer cles` en crée une paire). Sans elle, le jeu ne cherche
//! pas de mise à jour.

use ed25519_dalek::VerifyingKey;

use crate::manifeste::cle_publique;

/// Clés écrites à la compilation.
pub const CLES: Option<&str> = option_env!("BS_CLES_MAJ");

pub fn cles_du_jeu() -> Vec<VerifyingKey> {
    lire(CLES.unwrap_or(""))
}

/// Les clés valides d'une liste « clé1,clé2 » (les autres sont ignorées).
pub fn lire(liste: &str) -> Vec<VerifyingKey> {
    liste
        .split(',')
        .filter(|c| !c.trim().is_empty())
        .filter_map(cle_publique)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liste_de_cles() {
        let (_, a) = crate::manifeste::nouvelle_cle();
        let (_, b) = crate::manifeste::nouvelle_cle();
        assert_eq!(lire(&format!("{a}, {b},")).len(), 2);
        assert_eq!(lire(&format!("{a},pas-une-clé")).len(), 1);
        assert!(lire("").is_empty());
    }
}
