//! Bilan d'une partie (src/ui/bilan.ts, compteurDegats.ts), pour la pause et l'écran de
//! fin : chiffres clés sur deux colonnes, équipement porté, ennemis vaincus par type,
//! tableau des dégâts de chaque arme avec sa jauge. Aussi le résumé en tête de ces
//! écrans (temps, ennemis, niveau, émeraudes) : chiffres en ligne pour la pause, grands
//! chiffres pour la fin. En Inter, sans case ni cadre (`nuit`). Construit à l'ouverture
//! de l'écran seulement.

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_regles::equipement::niveau_max_objet;
use bs_regles::partie::Partie;
use bs_regles::textes::Famille;
use bs_regles::textes::libelles::nombre;

use super::charte::{AIDE, IVOIRE, VERT_GAIN};
use super::nuit::{self, Style, TERNE, ecrit};
use crate::dessins::Dessins;
use crate::ecran_cartes::couleur_famille;
use crate::polices::PolicesMenu;
use crate::textures::Textures;

const ICONE_TEMPS: &str = "item/clock_00";
const ICONE_TUES: &str = "item/bone";
const ICONE_NIVEAU: &str = "item/experience_bottle";
const TAILLE: f32 = 16.0;

pub fn demander_textures(d: &Donnees, demander: &mut impl FnMut(&str)) {
    demander(&d.talents.maitrise.icone);
    for t in [ICONE_TEMPS, ICONE_TUES, ICONE_NIVEAU] {
        demander(t);
    }
}

/// Icônes et textures pour construire.
pub struct Contexte<'a> {
    pub serveur: &'a AssetServer,
    pub textures: &'a Textures,
    pub dessins: &'a Dessins,
    pub polices: &'a PolicesMenu,
}

impl Contexte<'_> {
    pub(super) fn image(&self, chemin: &str, cote: f32) -> impl Bundle {
        let image = self
            .textures
            .obtenir(self.serveur, chemin)
            .map(ImageNode::new)
            .unwrap_or_else(|| ImageNode::default().with_color(Color::NONE));
        (
            image,
            Node {
                width: Val::Px(cote),
                height: Val::Px(cote),
                ..default()
            },
        )
    }

    /// Texte Inter (normal ou gras) ou en police du logo.
    pub fn ecrit(
        &self,
        commandes: &mut Commands,
        parent: Entity,
        t: &str,
        style: (Style, f32, Color),
    ) -> Entity {
        ecrit(commandes, self.polices, parent, t, style)
    }
}

/// Titre de section du bilan (surtitre doré, de l'air au-dessus).
fn section(commandes: &mut Commands, c: &Contexte, parent: Entity, t: &str) {
    let e = nuit::surtitre(commandes, c.polices, parent, t);
    commandes.entity(e).insert(Node {
        margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(18.0), Val::Px(4.0)),
        ..default()
    });
}

/// « 05:07 »
pub fn temps(secondes: f64) -> String {
    let s = secondes.max(0.0).floor() as u64;
    format!("{:02}:{:02}", s / 60, s % 60)
}

fn entier(v: f64) -> String {
    nombre(v.round() as f32)
}

fn decimal(v: f64) -> String {
    nombre(((v * 10.0).round() / 10.0) as f32)
}

/// « 845 », « 12,4 k », « 1,2 M »
fn degats(v: f64) -> String {
    if v >= 1e6 {
        format!("{} M", decimal(v / 1e6))
    } else if v >= 1e4 {
        format!("{} k", decimal(v / 1e3))
    } else {
        entier(v)
    }
}

/// Les quatre chiffres du résumé : icône, valeur, libellé.
fn resume(p: &Partie) -> [(&'static str, String, &'static str); 4] {
    let e = &p.etat;
    let emeraude = p.d.objets.get("emeraude").map_or("", |o| o.icone.as_str());
    [
        (ICONE_TEMPS, temps(e.temps_jeu), "Survécu"),
        (ICONE_TUES, entier(f64::from(e.ennemis_tues)), "Ennemis"),
        (ICONE_NIVEAU, e.joueur.niveau.to_string(), "Niveau"),
        (
            emeraude,
            entier(f64::from(e.stats.emeraudes_ramassees)),
            "Émeraudes",
        ),
    ]
}

/// Résumé en ligne (pause) : icône et valeur, séparés de pastilles.
pub fn pastilles(commandes: &mut Commands, c: &Contexte, parent: Entity, p: &Partie) {
    let ligne = nuit::rangee(commandes, parent, 18.0);
    for (k, (icone, valeur, _)) in resume(p).into_iter().enumerate() {
        if k > 0 {
            nuit::point(commandes, ligne, 5.0, TERNE);
        }
        let r = nuit::rangee(commandes, ligne, 8.0);
        commandes.spawn((c.image(icone, 22.0), ChildOf(r)));
        c.ecrit(commandes, r, &valeur, (Style::Grasse, 20.0, IVOIRE));
    }
}

/// Résumé en grands chiffres (fin de partie) : valeur, icône et libellé dessous.
pub fn tuiles(commandes: &mut Commands, c: &Contexte, parent: Entity, p: &Partie) {
    let ligne = nuit::rangee(commandes, parent, 56.0);
    for (icone, valeur, libelle) in resume(p) {
        let tuile = nuit::colonne(commandes, ligne, 4.0);
        commandes
            .entity(tuile)
            .entry::<Node>()
            .and_modify(|mut n| n.align_items = AlignItems::Center);
        c.ecrit(commandes, tuile, &valeur, (Style::Logo, 46.0, IVOIRE));
        let bas = nuit::rangee(commandes, tuile, 6.0);
        commandes.spawn((c.image(icone, 16.0), ChildOf(bas)));
        let t = libelle.to_uppercase();
        c.ecrit(commandes, bas, &t, (Style::Grasse, 12.0, AIDE));
    }
}

fn rangee(commandes: &mut Commands, parent: Entity, gap: f32) -> Entity {
    let n = Node {
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::End,
        column_gap: Val::Px(gap),
        row_gap: Val::Px(6.0),
        ..default()
    };
    commandes.spawn((n, ChildOf(parent))).id()
}

/// Bilan complet dans `parent` (colonne).
pub fn bilan(commandes: &mut Commands, c: &Contexte, parent: Entity, p: &Partie) {
    let (d, e) = (p.d, &p.etat);
    let s = &e.stats;
    let total: f64 = s.degats_par_arme.iter().sum();
    let chiffres = [
        ("Personnage", e.personnage.nom.clone()),
        ("Arène", e.arene.nom.clone()),
        ("Temps survécu", temps(e.temps_jeu)),
        ("Niveau atteint", e.joueur.niveau.to_string()),
        ("Expérience totale", entier(f64::from(s.xp_totale))),
        ("Ennemis vaincus", entier(f64::from(e.ennemis_tues))),
        ("Boss vaincus", s.boss_vaincus.to_string()),
        ("Dégâts infligés", entier(total)),
        ("Dégâts subis", entier(f64::from(s.degats_subis))),
        ("PV soignés", entier(f64::from(s.pv_soignes))),
        ("Totems utilisés", e.joueur.vies_utilisees.to_string()),
        (
            "Émeraudes ramassées",
            entier(f64::from(s.emeraudes_ramassees)),
        ),
        ("Coffres ouverts", s.coffres_ouverts.to_string()),
    ];
    let grille = commandes
        .spawn((
            Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
                column_gap: Val::Px(24.0),
                // Lignes alignées par le bas (les accents grandissent les textes)
                align_items: AlignItems::End,
                row_gap: Val::Px(4.0),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    for (libelle, valeur) in &chiffres {
        let l = commandes
            .spawn((
                Node {
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::End,
                    ..default()
                },
                ChildOf(grille),
            ))
            .id();
        c.ecrit(commandes, l, libelle, (Style::Normale, TAILLE, AIDE));
        c.ecrit(commandes, l, valeur, (Style::Grasse, TAILLE, IVOIRE));
    }

    // Équipement porté : icône, nom coloré de sa rareté, niveau
    section(commandes, c, parent, "ÉQUIPEMENT");
    let liste = rangee(commandes, parent, 16.0);
    for o in p.equipement.portes() {
        let def = &d.equipement.objets[o.objet];
        let objet = rangee(commandes, liste, 6.0);
        if let Some(h) = c.dessins.objets.get(o.objet) {
            commandes.spawn((
                ImageNode::new(h.clone()),
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    ..default()
                },
                ChildOf(objet),
            ));
        }
        let nom = if niveau_max_objet(d, o.objet) > 1 {
            format!("{} (niv. {})", def.nom, o.niveau)
        } else {
            def.nom.clone()
        };
        let couleur = couleur_famille(Famille::Objet(def.rarete)).0;
        c.ecrit(commandes, objet, &nom, (Style::Grasse, TAILLE, couleur));
    }
    if p.equipement.portes().next().is_none() {
        let vide = "Aucun objet : ouvrez des coffres avec vos émeraudes.";
        c.ecrit(commandes, liste, vide, (Style::Normale, TAILLE, AIDE));
    }

    // Ennemis vaincus, par type rencontré
    section(commandes, c, parent, "ENNEMIS VAINCUS");
    let liste = rangee(commandes, parent, 14.0);
    for ((_, def), &n) in d.ennemis.iter().zip(&s.tues_par_type) {
        if n > 0 {
            let t = format!("{} ×{}", def.nom, entier(f64::from(n)));
            c.ecrit(commandes, liste, &t, (Style::Normale, TAILLE, IVOIRE));
        }
    }
    if e.ennemis_tues == 0 {
        let t = "Aucun pour l'instant.";
        c.ecrit(commandes, liste, t, (Style::Normale, TAILLE, AIDE));
    }

    section(commandes, c, parent, "DÉGÂTS PAR ARME");
    tableau_degats(commandes, c, parent, p);
}

/// Tableau des dégâts, simplifié : arme et rang, total, part, dégâts par seconde,
/// achevés ; armes de la plus forte à la plus faible (le détail est au compteur, Tab).
fn tableau_degats(commandes: &mut Commands, c: &Contexte, parent: Entity, p: &Partie) {
    let (e, s) = (&p.etat, &p.etat.stats);
    let mut armes: Vec<_> = p.arsenal.armes.iter().collect();
    armes.sort_by(|a, b| s.degats_par_arme[b.index].total_cmp(&s.degats_par_arme[a.index]));
    let total: f64 = armes.iter().map(|a| s.degats_par_arme[a.index]).sum();
    let maximum = armes
        .first()
        .map_or(1.0, |a| s.degats_par_arme[a.index].max(1.0));
    let mut colonnes = vec![GridTrack::px(28.0), GridTrack::flex(1.0)];
    colonnes.extend((0..4).map(|_| GridTrack::auto()));
    let grille = commandes
        .spawn((
            Node {
                display: Display::Grid,
                grid_template_columns: colonnes,
                column_gap: Val::Px(10.0),
                row_gap: Val::Px(3.0),
                align_items: AlignItems::Center,
                justify_items: JustifyItems::End,
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    let cellule = |commandes: &mut Commands, t: &str, style: (Style, f32, Color), gauche: bool| {
        let e = c.ecrit(commandes, grille, t, style);
        if gauche {
            commandes.entity(e).insert(Node {
                justify_self: JustifySelf::Start,
                ..default()
            });
        }
    };
    commandes.spawn((Node::default(), ChildOf(grille)));
    for (k, titre) in ["Arme", "Dégâts", "Part", "Par s", "Achevés"]
        .into_iter()
        .enumerate()
    {
        let t = titre.to_uppercase();
        cellule(commandes, &t, (Style::Grasse, 12.0, TERNE), k == 0);
    }
    for a in armes {
        let i = a.index;
        let dg = s.degats_par_arme[i];
        let duree = (e.temps_jeu - s.acquisition_arme[i].max(0.0)).max(1.0);
        let rang = if a.rang >= a.rang_max() {
            "MAX".to_owned()
        } else {
            format!("rang {}", a.rang)
        };
        let part = if total > 0.0 { dg / total * 100.0 } else { 0.0 };
        commandes.spawn((c.image(&a.def.icone, 24.0), ChildOf(grille)));
        let nom = format!("{} · {rang}", a.def.nom);
        cellule(commandes, &nom, (Style::Grasse, 15.0, IVOIRE), true);
        for v in [
            degats(dg),
            format!("{} %", part.round()),
            decimal(dg / duree),
            entier(f64::from(s.tues_par_arme[i])),
        ] {
            cellule(commandes, &v, (Style::Normale, 15.0, IVOIRE), false);
        }
        // Jauge sous la ligne, de la 2e colonne à la dernière
        let jauge = commandes
            .spawn((
                Node {
                    grid_column: GridPlacement::start_end(2, -1),
                    justify_self: JustifySelf::Stretch,
                    height: Val::Px(4.0),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
                ChildOf(grille),
            ))
            .id();
        commandes.spawn((
            Node {
                width: Val::Percent((dg / maximum * 100.0) as f32),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(VERT_GAIN),
            ChildOf(jauge),
        ));
    }
}
