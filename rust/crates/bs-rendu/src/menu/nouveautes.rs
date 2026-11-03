//! Page Nouveautés du menu (src/ui/ecranNouveautes.ts) : le journal des versions, lu dans
//! CHANGELOG.md à la compilation (les plus récentes), défilant à la molette. Tant qu'il
//! n'a pas été ouvert pour la version installée, le bouton du menu porte « Nouveau ! ».
//! La dernière version est dépliée, les autres repliées (un clic les déplie) ; chaque
//! section montre ses premiers points en Inter 18 (docs/CHARTE-UI.md).

use bevy::prelude::*;
use bs_sauvegarde::Stockage;

use super::{ActionMenu, Icones};
use crate::ecrans::charte::{self, AIDE, IVOIRE, OR_VIF, Relief, T_AIDE, T_CORPS, T_SECTION};
use crate::ecrans::vues::Defilant;
use crate::hud::police::TextePixel;

const JOURNAL: &str = include_str!("../../../../../CHANGELOG.md");
const CLE: &str = "block-survivors.version-vue";
/// Versions affichées (les plus récentes).
const NOMBRE_VERSIONS: usize = 4;
const LARGEUR: f32 = 820.0;
/// Points affichés par section (sections courtes).
const LIGNES_PAR_SECTION: usize = 6;

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

/// Texte Inter à la largeur du panneau.
fn texte(
    c: &mut Commands,
    ic: &mut Icones,
    parent: Entity,
    t: &str,
    (gras, taille, couleur): (bool, f32, Color),
) -> Entity {
    let t = ic.polices.texte(t, gras, taille, couleur);
    c.spawn((t, ChildOf(parent))).id()
}

/// Construit la page, la version `ouverte` dépliée ; renvoie sa racine.
pub fn construire(c: &mut Commands, ic: &mut Icones, ouverte: usize) -> Entity {
    let racine = c
        .spawn((
            Name::new("nouveautés"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(charte::MARGE)),
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(charte::FOND),
        ))
        .id();
    c.spawn((
        TextePixel::nouveau("Nouveautés", IVOIRE, 4.0),
        ChildOf(racine),
    ));
    let panneau = charte::panneau(
        c,
        racine,
        Node {
            width: Val::Px(LARGEUR),
            max_width: Val::Percent(100.0),
            flex_grow: 1.0,
            min_height: Val::ZERO,
            padding: UiRect::new(Val::Px(16.0), Val::Px(8.0), Val::Px(16.0), Val::Px(16.0)),
            ..default()
        },
    );
    let zone = c
        .spawn((
            Defilant,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                min_height: Val::ZERO,
                padding: UiRect::right(Val::Px(12.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
            ChildOf(panneau),
        ))
        .id();
    for (k, v) in lire_journal(JOURNAL)
        .iter()
        .take(NOMBRE_VERSIONS)
        .enumerate()
    {
        let deplie = k == ouverte;
        // En-tête : bouton qui déplie la version (« ⌄ » si ouverte, « › » sinon)
        let relief = if deplie {
            Relief::CHOISI
        } else {
            Relief::SECONDAIRE
        };
        let b = charte::bouton_relief(
            c,
            zone,
            relief,
            charte::H_SECONDAIRE,
            ActionMenu::Version(k),
        );
        c.entity(b).entry::<Node>().and_modify(|mut n| {
            n.justify_content = JustifyContent::SpaceBetween;
            n.flex_shrink = 0.0;
        });
        let couleur = if deplie { OR_VIF } else { IVOIRE };
        texte(c, ic, b, &v.titre, (true, T_SECTION - 2.0, couleur));
        let resume = if deplie {
            "⌄".to_owned()
        } else {
            let n: usize = v.sections.iter().map(|(_, l)| l.len()).sum();
            format!("{n} changement{} ›", if n > 1 { "s" } else { "" })
        };
        texte(c, ic, b, &resume, (false, T_AIDE, AIDE));
        if !deplie {
            continue;
        }
        let corps = c
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.0,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::new(Val::Px(12.0), Val::Px(8.0), Val::Px(4.0), Val::Px(12.0)),
                    ..default()
                },
                ChildOf(zone),
            ))
            .id();
        for (section, lignes) in &v.sections {
            let t = texte(c, ic, corps, section, (true, T_CORPS + 1.0, OR_VIF));
            c.entity(t).insert(Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            });
            // Sections courtes : les premiers points seulement
            let principales: Vec<_> = lignes.iter().filter(|l| !l.sous).collect();
            for l in principales.iter().take(LIGNES_PAR_SECTION) {
                let propre = l.texte.replace("**", "").replace('`', "");
                let t = texte(
                    c,
                    ic,
                    corps,
                    &format!("•  {propre}"),
                    (false, T_CORPS + 1.0, IVOIRE),
                );
                c.entity(t).insert(Node {
                    margin: UiRect::left(Val::Px(8.0)),
                    ..default()
                });
            }
            let reste = principales.len().saturating_sub(LIGNES_PAR_SECTION);
            if reste > 0 {
                let t = format!("… et {reste} autre{}", if reste > 1 { "s" } else { "" });
                let t = texte(c, ic, corps, &t, (false, T_AIDE, AIDE));
                c.entity(t).insert(Node {
                    margin: UiRect::left(Val::Px(8.0)),
                    ..default()
                });
            }
        }
    }
    let retour = charte::bouton_relief(
        c,
        racine,
        Relief::SECONDAIRE,
        charte::H_SECONDAIRE,
        ActionMenu::Fermer,
    );
    c.entity(retour).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(200.0);
        n.flex_shrink = 0.0;
    });
    texte(c, ic, retour, "Retour", (true, T_CORPS, IVOIRE));
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
