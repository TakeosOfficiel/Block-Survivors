//! Page Nouveautés du menu (src/ui/ecranNouveautes.ts) : le journal des versions, lu dans
//! CHANGELOG.md à la compilation (les plus récentes), défilant à la molette. Tant qu'il
//! n'a pas été ouvert pour la version installée, le bouton du menu porte « Nouveau ! ».
//! En menu de console (`console`) : à gauche les versions en grands intitulés (l'ouverte
//! dorée, un clic sur une autre l'ouvre) ; à droite ses sections et tous ses points,
//! en pastilles rondes, sans cadre.

use bevy::prelude::*;
use bs_sauvegarde::Stockage;

use super::console::{self, Police, aide, ecrit, intitule, surtitre};
use super::{ActionMenu, Icones, air};
use crate::ecrans::charte::{IVOIRE, OR_VIF};
use crate::ecrans::vues::Defilant;

const JOURNAL: &str = include_str!("../../../../../CHANGELOG.md");
const CLE: &str = "block-survivors.version-vue";
/// Versions affichées (les plus récentes).
const NOMBRE_VERSIONS: usize = 4;

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

/// Titre « Version 0.6.0 — date » coupé en numéro et date.
fn numero(titre: &str) -> (&str, &str) {
    let t = titre.strip_prefix("Version ").unwrap_or(titre);
    t.split_once(" — ").unwrap_or((t, ""))
}

/// Point du journal : pastille dorée (pâle et en retrait pour un sous-point) et texte.
fn point(c: &mut Commands, ic: &Icones, parent: Entity, l: &Ligne) {
    let r = air::rangee(c, parent, 14.0);
    let retrait = if l.sous { 28.0 } else { 0.0 };
    c.entity(r).entry::<Node>().and_modify(move |mut n| {
        n.align_items = AlignItems::FlexStart;
        n.margin = UiRect::left(Val::Px(retrait));
    });
    let (cote, couleur, taille, texte) = if l.sous {
        (5.0, OR_VIF.with_alpha(0.45), 15.0, IVOIRE.with_alpha(0.7))
    } else {
        (7.0, OR_VIF, 17.0, IVOIRE)
    };
    let p = air::point(c, r, cote, couleur);
    c.entity(p)
        .entry::<Node>()
        .and_modify(move |mut n| n.margin = UiRect::top(Val::Px(taille * 0.5)));
    let propre = l.texte.replace("**", "").replace('`', "");
    let e = ecrit(c, ic, r, &propre, (Police::Normale, taille, texte));
    c.entity(e).insert(Node {
        flex_shrink: 1.0,
        ..default()
    });
}

/// Construit la page, la version `ouverte` dépliée ; renvoie sa racine.
pub fn construire(c: &mut Commands, ic: &mut Icones, ouverte: usize) -> Entity {
    let console::Page {
        racine,
        gauche,
        scene: d,
    } = console::page(c, ic, ("nouveautés", "NOUVEAUTÉS"), (460.0, 820.0));
    let versions = lire_journal(JOURNAL);
    for (k, v) in versions.iter().take(NOMBRE_VERSIONS).enumerate() {
        let (num, date) = numero(&v.titre);
        let n: usize = v.sections.iter().map(|(_, l)| l.len()).sum();
        let pluriel = if n > 1 { "s" } else { "" };
        let aide = match (k, date.is_empty()) {
            (0, _) => format!("La plus récente · {n} changement{pluriel}"),
            (_, true) => format!("{n} changement{pluriel}"),
            (_, false) => format!("{date} · {n} changement{pluriel}"),
        };
        let action = (k != ouverte).then_some(ActionMenu::Version(k));
        intitule(c, ic, gauche, (num, &aide), (64.0, action));
    }
    let Some(v) = versions.get(ouverte) else {
        return racine;
    };
    let (num, date) = numero(&v.titre);
    let t = if date.is_empty() {
        format!("VERSION {num}")
    } else {
        format!("VERSION {num} · {date}")
    };
    surtitre(c, ic, d, &t);
    // Journal défilant à la molette
    let zone = c
        .spawn((
            Defilant,
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                min_height: Val::ZERO,
                max_height: Val::Px(520.0),
                row_gap: Val::Px(10.0),
                padding: UiRect::right(Val::Px(16.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
            ChildOf(d),
        ))
        .id();
    for (k, (section, lignes)) in v.sections.iter().enumerate() {
        let e = ecrit(c, ic, zone, section, (Police::Logo, 30.0, OR_VIF));
        c.entity(e).insert(Node {
            margin: UiRect::top(Val::Px(if k == 0 { 4.0 } else { 22.0 })),
            flex_shrink: 0.0,
            ..default()
        });
        for l in lignes {
            point(c, ic, zone, l);
        }
    }
    let p = console::pied(c, ic, racine);
    aide(c, ic, p, "Molette : faire défiler le journal.");
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
