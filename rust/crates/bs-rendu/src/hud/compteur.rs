//! Compteur de dégâts en jeu (src/ui/compteurDegats.ts) : un panneau compact en haut à
//! droite, une ligne par arme (icône, barre relative à la meilleure, nom, dégâts, dégâts
//! par seconde depuis son obtention, part du total), et le total en pied. Tab ou l'option
//! « Compteur » l'affiche ou le cache. Rafraîchi deux fois par seconde (temps réel) ;
//! les lignes sont créées une fois et seulement réécrites. Le panneau n'est pas
//! déplaçable (il l'est dans le jeu TS).

use bevy::prelude::*;
use bs_regles::textes::libelles::nombre;

use super::police::{TextePixel, ecrire};
use super::{BLANC, GRIS, Hud, absolu};
use crate::PartieEnCours;
use crate::options::Options;
use crate::textures::Textures;

/// Lignes au plus (les armes au-delà ne sont pas montrées).
const LIGNES: usize = 8;
const LARGEUR: f32 = 340.0;
const ICONE: f32 = 22.0;
const PERIODE: f32 = 0.5;
const VERT_BARRE: Color = Color::srgba(0.33, 0.8, 0.33, 0.38);

struct Ligne {
    racine: Entity,
    icone: Entity,
    barre: Entity,
    nom: Entity,
    valeur: Entity,
    dps: Entity,
    part: Entity,
    /// Arme montrée (index dans la table des armes).
    arme: Option<usize>,
}

/// Entités du panneau.
pub struct Compteur {
    racine: Entity,
    lignes: [Ligne; LIGNES],
    total: Entity,
    minuterie: f32,
}

/// « 845 », « 1,8 k », « 2,4 M »
fn abreger(v: f64) -> String {
    let decimal = |x: f64| nombre(((x * 10.0).round() / 10.0) as f32);
    if v >= 1e6 {
        format!("{} M", decimal(v / 1e6))
    } else if v >= 1e3 {
        format!("{} k", decimal(v / 1e3))
    } else {
        nombre(v.round() as f32)
    }
}

fn texte(commandes: &mut Commands, parent: Entity, t: &str, couleur: Color) -> Entity {
    commandes
        .spawn((
            TextePixel::nouveau(t, couleur, 1.5).contour(),
            ChildOf(parent),
        ))
        .id()
}

fn ligne(commandes: &mut Commands, parent: Entity) -> Ligne {
    let racine = commandes
        .spawn((
            Node {
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                display: Display::None,
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    let icone = commandes
        .spawn((
            ImageNode::default().with_color(Color::NONE),
            Node {
                width: Val::Px(ICONE),
                height: Val::Px(ICONE),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    let piste = commandes
        .spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Px(ICONE),
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                padding: UiRect::horizontal(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
            ChildOf(racine),
        ))
        .id();
    let barre = commandes
        .spawn((
            Node {
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(0.0),
                ..absolu()
            },
            BackgroundColor(VERT_BARRE),
            ChildOf(piste),
        ))
        .id();
    let nom = texte(commandes, piste, "", BLANC);
    // Espace entre le nom et les chiffres (le texte lui-même ne s'étire pas)
    commandes.spawn((
        Node {
            flex_grow: 1.0,
            ..default()
        },
        ChildOf(piste),
    ));
    let valeur = texte(commandes, piste, "", BLANC);
    let dps = texte(commandes, piste, "", GRIS);
    let part = texte(commandes, piste, "", GRIS);
    Ligne {
        racine,
        icone,
        barre,
        nom,
        valeur,
        dps,
        part,
        arme: None,
    }
}

/// Construit le panneau (vide) dans le HUD.
pub fn construire(commandes: &mut Commands, parent: Entity) -> Compteur {
    let racine = commandes
        .spawn((
            Name::new("compteur de dégâts"),
            Node {
                top: Val::Px(64.0),
                right: Val::Px(10.0),
                width: Val::Px(LARGEUR),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                padding: UiRect::all(Val::Px(6.0)),
                ..absolu()
            },
            BackgroundColor(Color::srgba(16.0 / 255.0, 0.0, 16.0 / 255.0, 0.82)),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();
    let titre = commandes
        .spawn((
            Node {
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    texte(commandes, titre, "Dégâts", super::OR);
    texte(commandes, titre, "Tab", GRIS);
    let lignes = std::array::from_fn(|_| ligne(commandes, racine));
    let pied = commandes
        .spawn((
            Node {
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    texte(commandes, pied, "Total", GRIS);
    let total = texte(commandes, pied, "", BLANC);
    Compteur {
        racine,
        lignes,
        total,
        minuterie: 0.0,
    }
}

/// Tab bascule l'option ; le panneau suit l'option et se réécrit deux fois par seconde.
#[allow(clippy::too_many_arguments)]
pub fn afficher(
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    jeu: Res<PartieEnCours>,
    hud: Option<ResMut<Hud>>,
    mut options: ResMut<Options>,
    touches: Res<ButtonInput<KeyCode>>,
    temps: Res<Time<Real>>,
    mut visibles: Query<&mut Visibility>,
    mut noeuds: Query<&mut Node>,
    mut textes: Query<&mut TextePixel>,
    mut images: Query<&mut ImageNode>,
) {
    let Some(mut hud) = hud else {
        return;
    };
    if touches.just_pressed(KeyCode::Tab) {
        options.compteur_degats = !options.compteur_degats;
    }
    let c = &mut hud.compteur;
    let actif = options.compteur_degats;
    if let Ok(mut v) = visibles.get_mut(c.racine) {
        v.set_if_neq(if actif {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    if options.is_changed() && actif {
        c.minuterie = 0.0;
    }
    c.minuterie -= temps.delta_secs();
    if !actif || c.minuterie > 0.0 {
        return;
    }
    c.minuterie = PERIODE;

    // Armes triées de la plus forte à la plus faible (tableau fixe, sans allocation)
    let p = &jeu.partie;
    let (e, s) = (&p.etat, &p.etat.stats);
    let mut tri = [(0usize, 0.0f64); LIGNES];
    let mut n = 0;
    for a in p.arsenal.armes.iter().take(LIGNES) {
        tri[n] = (a.index, s.degats_par_arme[a.index]);
        n += 1;
    }
    tri[..n].sort_by(|a, b| b.1.total_cmp(&a.1));
    let total: f64 = tri[..n].iter().map(|t| t.1).sum();
    let maximum = tri.first().map_or(1.0, |t| t.1).max(1.0);

    for (k, l) in c.lignes.iter_mut().enumerate() {
        let montree = k < n;
        if let Ok(mut node) = noeuds.get_mut(l.racine) {
            let display = if montree {
                Display::Flex
            } else {
                Display::None
            };
            if node.display != display {
                node.display = display;
            }
        }
        if !montree {
            l.arme = None;
            continue;
        }
        let (i, degats) = tri[k];
        let Some(arme) = p.arsenal.armes.iter().find(|a| a.index == i) else {
            continue;
        };
        if l.arme != Some(i) {
            let image = textures.obtenir(&serveur, &arme.def.icone);
            if let Ok(mut img) = images.get_mut(l.icone) {
                *img = image.map_or_else(
                    || ImageNode::default().with_color(Color::NONE),
                    ImageNode::new,
                );
            }
            if let Ok(mut t) = textes.get_mut(l.nom) {
                ecrire(&mut t, &arme.def.nom);
            }
            // Icône pas encore chargée : on réessaiera
            l.arme = textures.obtenir(&serveur, &arme.def.icone).map(|_| i);
        }
        if let Ok(mut node) = noeuds.get_mut(l.barre) {
            let largeur = Val::Percent((degats / maximum * 100.0) as f32);
            if node.width != largeur {
                node.width = largeur;
            }
        }
        let duree = (e.temps_jeu - s.acquisition_arme[i].max(0.0)).max(1.0);
        let part = if total > 0.0 { degats / total } else { 0.0 };
        for (entite, t) in [
            (l.valeur, abreger(degats)),
            (l.dps, format!("{}/s", abreger(degats / duree))),
            (l.part, format!("{} %", (part * 100.0).round())),
        ] {
            if let Ok(mut texte) = textes.get_mut(entite) {
                ecrire(&mut texte, &t);
            }
        }
    }
    let dps = total / e.temps_jeu.max(1.0);
    if let Ok(mut t) = textes.get_mut(c.total) {
        ecrire(&mut t, &format!("{} · {}/s", abreger(total), abreger(dps)));
    }
}

#[cfg(test)]
mod tests {
    use super::abreger;

    #[test]
    fn abreviations() {
        assert_eq!(abreger(845.4), "845");
        assert_eq!(abreger(1840.0), "1,8 k");
        assert_eq!(abreger(2_400_000.0), "2,4 M");
    }
}
