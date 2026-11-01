//! Contenu du jeu : les fichiers RON de `rust/donnees/` (armes, ennemis, arènes,
//! talents…), leurs types, leur chargement et leur validation.
//!
//! Le contenu est en données, les formules en code (bs-regles). Les fichiers sont
//! produits depuis la version TypeScript par `outils/importer-donnees.sh` tant que
//! les deux versions coexistent ; ensuite, on les modifie à la main.
//!
//! Les noms des champs sont ceux de la version TypeScript (camelCase) : les
//! identifiants du contenu (parties de modèles, gerbes) ont la même forme.

pub mod arenes;
pub mod armes;
pub mod blocs;
mod communs;
pub mod effets;
pub mod entites;
pub mod equipement;
pub mod importation;
pub mod progression;
pub mod regles;
mod validation;

use std::path::Path;

use serde::de::DeserializeOwned;

pub use communs::{
    Apparence, Intervalle, ModifArme, Modifs, StatArme, StatsArme, Table, Texture, Vec3,
};
pub use validation::ErreurValidation;

/// Tout le contenu du jeu.
#[derive(Debug, Clone, PartialEq)]
pub struct Donnees {
    pub blocs: Table<blocs::Bloc>,
    pub modeles: Table<blocs::Modele>,
    pub ennemis: Table<entites::Ennemi>,
    pub projectiles: Table<entites::Projectile>,
    pub armes: Table<armes::Arme>,
    pub allies: entites::Allies,
    pub personnages: entites::Personnages,
    pub arenes: arenes::Arenes,
    pub structures: arenes::Structures,
    pub passifs: progression::Passifs,
    pub objets: Table<entites::Objet>,
    pub coffres: entites::Coffres,
    pub equipement: equipement::Equipement,
    pub armures: equipement::Armures,
    pub talents: progression::Talents,
    pub sons: effets::Sons,
    pub particules: effets::Particules,
    pub regles: regles::Regles,
    pub textes: regles::Textes,
}

/// Les fichiers de `donnees/`, sans l'extension « .ron ».
pub const FICHIERS: [&str; 19] = [
    "blocs",
    "modeles",
    "ennemis",
    "projectiles",
    "armes",
    "allies",
    "personnages",
    "arenes",
    "structures",
    "passifs",
    "objets",
    "coffres",
    "equipement",
    "armures",
    "talents",
    "sons",
    "particules",
    "regles",
    "textes",
];

#[derive(Debug, thiserror::Error)]
pub enum ErreurDonnees {
    #[error("{fichier} : lecture impossible ({source})")]
    Lecture {
        fichier: String,
        source: std::io::Error,
    },
    #[error("{fichier}:{ligne}:{colonne} : {message}")]
    Format {
        fichier: String,
        ligne: usize,
        colonne: usize,
        message: String,
    },
    #[error("données incohérentes :\n{}", .0.iter().map(|e| format!("  - {e}")).collect::<Vec<_>>().join("\n"))]
    Validation(Vec<ErreurValidation>),
}

fn lire<T: DeserializeOwned>(fichier: &str, texte: &str) -> Result<T, ErreurDonnees> {
    ron::Options::default()
        .from_str(texte)
        .map_err(|e| ErreurDonnees::Format {
            fichier: format!("{fichier}.ron"),
            ligne: e.span.start.line,
            colonne: e.span.start.col,
            message: e.code.to_string(),
        })
}

impl Donnees {
    /// Lit et valide les données à partir du texte de chaque fichier (nom sans extension).
    pub fn depuis<'a>(
        mut texte: impl FnMut(&str) -> Result<std::borrow::Cow<'a, str>, ErreurDonnees>,
    ) -> Result<Self, ErreurDonnees> {
        let mut t = |nom: &str| texte(nom).map(|s| (nom.to_owned(), s));
        macro_rules! table {
            ($nom:literal) => {{
                let (fichier, s) = t($nom)?;
                lire(&fichier, &s)?
            }};
        }
        let donnees = Self {
            blocs: table!("blocs"),
            modeles: table!("modeles"),
            ennemis: table!("ennemis"),
            projectiles: table!("projectiles"),
            armes: table!("armes"),
            allies: table!("allies"),
            personnages: table!("personnages"),
            arenes: table!("arenes"),
            structures: table!("structures"),
            passifs: table!("passifs"),
            objets: table!("objets"),
            coffres: table!("coffres"),
            equipement: table!("equipement"),
            armures: table!("armures"),
            talents: table!("talents"),
            sons: table!("sons"),
            particules: table!("particules"),
            regles: table!("regles"),
            textes: table!("textes"),
        };
        let erreurs = validation::valider(&donnees);
        if erreurs.is_empty() {
            Ok(donnees)
        } else {
            Err(ErreurDonnees::Validation(erreurs))
        }
    }

    /// Lit et valide les données d'un dossier (relues au lancement, et à chaud en développement).
    pub fn charger(dossier: &Path) -> Result<Self, ErreurDonnees> {
        Self::depuis(|nom| {
            let chemin = dossier.join(format!("{nom}.ron"));
            std::fs::read_to_string(&chemin)
                .map(Into::into)
                .map_err(|source| ErreurDonnees::Lecture {
                    fichier: chemin.display().to_string(),
                    source,
                })
        })
    }

    /// Les données embarquées dans l'exécutable (jeu, tests).
    pub fn embarquees() -> Result<Self, ErreurDonnees> {
        Self::depuis(|nom| Ok(embarque(nom).into()))
    }
}

/// Texte d'un fichier de `donnees/`, embarqué à la compilation.
fn embarque(nom: &str) -> &'static str {
    macro_rules! fichiers {
        ($($n:literal),*) => {
            match nom {
                $($n => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../donnees/", $n, ".ron")),)*
                _ => "",
            }
        };
    }
    fichiers!(
        "blocs",
        "modeles",
        "ennemis",
        "projectiles",
        "armes",
        "allies",
        "personnages",
        "arenes",
        "structures",
        "passifs",
        "objets",
        "coffres",
        "equipement",
        "armures",
        "talents",
        "sons",
        "particules",
        "regles",
        "textes"
    )
}
