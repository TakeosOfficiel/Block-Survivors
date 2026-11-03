//! Bilan d'une partie (src/ui/bilan.ts, compteurDegats.ts), pour la pause et l'écran de
//! fin : chiffres clés sur deux colonnes, équipement porté, ennemis vaincus par type,
//! tableau des dégâts de chaque arme avec sa jauge. Aussi le résumé en tête de ces
//! écrans (temps, ennemis, niveau, émeraudes) : pastilles de la pause, tuiles de la fin.
//! Construit à l'ouverture de l'écran seulement.

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_regles::equipement::niveau_max_objet;
use bs_regles::partie::Partie;
use bs_regles::textes::Famille;
use bs_regles::textes::libelles::nombre;

use super::style::{GRIS, OR_CLAIR};
use crate::dessins::Dessins;
use crate::ecran_cartes::couleur_famille;
use crate::hud::police::TextePixel;
use crate::textures::Textures;

const ICONE_TEMPS: &str = "item/clock_00";
const ICONE_TUES: &str = "item/bone";
const ICONE_NIVEAU: &str = "item/experience_bottle";
const CLAIR: Color = Color::srgb_u8(0xd8, 0xd8, 0xd8);
const VERT: Color = Color::srgb_u8(0x3c, 0xb8, 0x50);
const TAILLE: f32 = 2.0;
const TAILLE_TABLEAU: f32 = 1.5;

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
}

pub fn texte(commandes: &mut Commands, parent: Entity, t: &str, couleur: Color, taille: f32) {
    commandes.spawn((TextePixel::nouveau(t, couleur, taille), ChildOf(parent)));
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

/// Résumé en pastilles (pause) : icône et valeur.
pub fn pastilles(commandes: &mut Commands, c: &Contexte, parent: Entity, p: &Partie) {
    let ligne = commandes
        .spawn((
            Node {
                column_gap: Val::Px(8.0),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    for (icone, valeur, _) in resume(p) {
        let pastille = commandes
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.07)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.12)),
                ChildOf(ligne),
            ))
            .id();
        commandes.spawn((c.image(icone, 20.0), ChildOf(pastille)));
        texte(commandes, pastille, &valeur, Color::WHITE, TAILLE);
    }
}

/// Résumé en tuiles (fin de partie) : grande valeur, icône et libellé.
pub fn tuiles(commandes: &mut Commands, c: &Contexte, parent: Entity, p: &Partie) {
    let ligne = commandes
        .spawn((
            Node {
                column_gap: Val::Px(12.0),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    for (icone, valeur, libelle) in resume(p) {
        let tuile = commandes
            .spawn((
                Node {
                    min_width: Val::Px(130.0),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(6.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba_u8(10, 12, 18, 153)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.12)),
                ChildOf(ligne),
            ))
            .id();
        texte(commandes, tuile, &valeur, Color::WHITE, 3.5);
        let bas = commandes
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(5.0),
                    ..default()
                },
                ChildOf(tuile),
            ))
            .id();
        commandes.spawn((c.image(icone, 16.0), ChildOf(bas)));
        texte(commandes, bas, &libelle.to_uppercase(), GRIS, 1.5);
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
        texte(commandes, l, libelle, CLAIR, TAILLE);
        texte(commandes, l, valeur, Color::WHITE, TAILLE);
    }

    // Équipement porté : icône, nom coloré de sa rareté, niveau
    texte(commandes, parent, "Équipement", OR_CLAIR, TAILLE);
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
        texte(commandes, objet, &nom, couleur, TAILLE);
    }
    if p.equipement.portes().next().is_none() {
        let vide = "Aucun objet : ouvrez des coffres avec vos émeraudes.";
        texte(commandes, liste, vide, GRIS, TAILLE);
    }

    // Ennemis vaincus, par type rencontré
    texte(commandes, parent, "Ennemis vaincus", OR_CLAIR, TAILLE);
    let liste = rangee(commandes, parent, 14.0);
    for ((_, def), &n) in d.ennemis.iter().zip(&s.tues_par_type) {
        if n > 0 {
            let t = format!("{} ×{}", def.nom, entier(f64::from(n)));
            texte(commandes, liste, &t, CLAIR, TAILLE);
        }
    }
    if e.ennemis_tues == 0 {
        texte(commandes, liste, "Aucun pour l'instant.", GRIS, TAILLE);
    }

    texte(commandes, parent, "Dégâts par arme", OR_CLAIR, TAILLE);
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
    let cellule = |commandes: &mut Commands, t: &str, couleur: Color, gauche: bool| {
        let mut n = Node::default();
        if gauche {
            n.justify_self = JustifySelf::Start;
        }
        commandes.spawn((
            TextePixel::nouveau(t, couleur, TAILLE_TABLEAU),
            n,
            ChildOf(grille),
        ));
    };
    commandes.spawn((Node::default(), ChildOf(grille)));
    for (k, titre) in ["Arme", "Dégâts", "Part", "Par s", "Achevés"]
        .into_iter()
        .enumerate()
    {
        cellule(commandes, titre, GRIS, k == 0);
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
        cellule(
            commandes,
            &format!("{} ({rang})", a.def.nom),
            Color::WHITE,
            true,
        );
        for v in [
            degats(dg),
            format!("{} %", part.round()),
            decimal(dg / duree),
            entier(f64::from(s.tues_par_arme[i])),
        ] {
            cellule(commandes, &v, Color::WHITE, false);
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
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(VERT),
            ChildOf(jauge),
        ));
    }
}
