//! Page Nouveautés du menu (src/ui/ecranNouveautes.ts) : le journal des versions, lu dans
//! CHANGELOG.md à la compilation (les plus récentes), défilant à la molette. Tant qu'il
//! n'a pas été ouvert pour la version installée, le bouton du menu porte « Nouveau ! ».

use bevy::prelude::*;
use bs_sauvegarde::Stockage;

use super::ActionMenu;
use crate::ecrans::style::{self, GRIS, OR, OR_CLAIR, TEXTE};
use crate::ecrans::vues::Defilant;
use crate::hud::police::TextePixel;

const JOURNAL: &str = include_str!("../../../../../CHANGELOG.md");
const CLE: &str = "block-survivors.version-vue";
/// Versions affichées (les plus récentes).
const NOMBRE_VERSIONS: usize = 4;
const LARGEUR: f32 = 820.0;

/// Une version du journal : son titre et ses sections (titre, lignes).
#[derive(Debug, PartialEq)]
pub struct Version<'a> {
    pub titre: String,
    pub sections: Vec<(&'a str, Vec<Ligne<'a>>)>,
}

/// Ligne d'une section (`sous` : sous-point, en retrait).
#[derive(Debug, PartialEq)]
pub struct Ligne<'a> {
    pub texte: &'a str,
    pub sous: bool,
}

/// Découpe le journal : « ## [0.6.0] — date », « ### Section », « - ligne ».
pub fn lire_journal(texte: &str) -> Vec<Version<'_>> {
    let mut versions: Vec<Version> = Vec::new();
    for brute in texte.lines() {
        let ligne = brute.trim();
        if let Some(t) = ligne.strip_prefix("## ") {
            let titre = format!("Version {}", t.replace(['[', ']'], ""));
            versions.push(Version {
                titre,
                sections: Vec::new(),
            });
        } else if let Some(t) = ligne.strip_prefix("### ") {
            if let Some(v) = versions.last_mut() {
                v.sections.push((t, Vec::new()));
            }
        } else if let Some(t) = ligne.strip_prefix("- ") {
            let section = versions.last_mut().and_then(|v| v.sections.last_mut());
            if let Some((_, lignes)) = section {
                let sous = brute.starts_with(' ') || brute.starts_with('\t');
                lignes.push(Ligne { texte: t, sous });
            }
        }
    }
    versions
}

/// Les nouveautés de cette version n'ont pas encore été lues.
pub fn a_lire(stockage: &Stockage) -> bool {
    stockage.lire::<String>(CLE).as_deref() != Some(bs_commun::VERSION)
}

/// Note les nouveautés de cette version comme lues.
pub fn marquer_lues(stockage: &Stockage) {
    stockage.ecrire(CLE, &bs_commun::VERSION);
}

/// Construit la page ; renvoie sa racine.
pub fn construire(commandes: &mut Commands) -> Entity {
    let racine = commandes
        .spawn((
            Name::new("nouveautés"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(super::vue::FOND),
        ))
        .id();
    commandes.spawn((
        TextePixel::nouveau("Nouveautés", OR_CLAIR, 3.0),
        ChildOf(racine),
    ));
    let zone = commandes
        .spawn((
            Defilant,
            Node {
                width: Val::Px(LARGEUR),
                flex_grow: 1.0,
                min_height: Val::ZERO,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                overflow: Overflow {
                    x: OverflowAxis::Clip,
                    y: OverflowAxis::Scroll,
                },
                ..default()
            },
            ScrollPosition::default(),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            BorderColor::all(Color::srgb_u8(0x2a, 0x2a, 0x2a)),
            ChildOf(racine),
        ))
        .id();
    let a_gauche = |t: &str, couleur, taille, retrait: f32| {
        (
            TextePixel::nouveau(t, couleur, taille)
                .bloc(LARGEUR - 40.0 - retrait)
                .a_gauche(),
            Node {
                margin: UiRect::left(Val::Px(retrait)),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(zone),
        )
    };
    for (k, v) in lire_journal(JOURNAL)
        .iter()
        .take(NOMBRE_VERSIONS)
        .enumerate()
    {
        let couleur = if k == 0 { OR_CLAIR } else { OR };
        let mut titre = commandes.spawn(a_gauche(&v.titre, couleur, 2.5, 0.0));
        if k > 0 {
            titre.entry::<Node>().and_modify(|mut n| {
                n.margin.top = Val::Px(14.0);
            });
        }
        for (section, lignes) in &v.sections {
            commandes.spawn(a_gauche(section, GRIS, 2.0, 0.0));
            for l in lignes {
                let (puce, retrait) = if l.sous { ("◦", 36.0) } else { ("•", 14.0) };
                let t = format!("{puce} {}", l.texte.replace("**", "").replace('`', ""));
                commandes.spawn(a_gauche(&t, TEXTE, 1.5, retrait));
            }
        }
    }
    let retour = style::bouton(
        commandes,
        racine,
        "Retour",
        200.0,
        false,
        ActionMenu::Fermer,
    );
    commandes
        .entity(retour)
        .entry::<Node>()
        .and_modify(|mut n| n.flex_shrink = 0.0);
    racine
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lecture_du_journal() {
        let j = "# Titre\n- hors version\n## [0.2.0] — 2026\n### Nouveautés\n- **A** : a\n  - sous\n### Corrections\n- b\n## [0.1.0]\n- sans section\n";
        let v = lire_journal(j);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].titre, "Version 0.2.0 — 2026");
        assert_eq!(v[0].sections.len(), 2);
        let (titre, lignes) = &v[0].sections[0];
        assert_eq!(*titre, "Nouveautés");
        assert_eq!(lignes.len(), 2);
        assert!(!lignes[0].sous && lignes[1].sous);
        assert!(v[1].sections.is_empty());
        // Le vrai journal se lit
        assert!(lire_journal(JOURNAL).len() >= NOMBRE_VERSIONS);
    }
}
