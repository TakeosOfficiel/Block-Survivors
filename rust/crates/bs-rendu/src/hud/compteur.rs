//! Compteur de dégâts en jeu (src/ui/compteurDegats.ts), au style des menus : en haut à
//! droite sur une lueur de nuit sans cadre, une ligne par arme (icône, nom, dégâts, dégâts
//! par seconde depuis son obtention, part du total, rail vert arrondi relatif à la
//! meilleure), et le total en pied. Tab ou l'option
//! « Compteur » l'affiche ou le cache. Rafraîchi deux fois par seconde (temps réel) ;
//! les lignes sont créées une fois et seulement réécrites. Le compteur n'est pas
//! déplaçable (il l'est dans le jeu TS). Masqué pendant les écrans de choix.

use bevy::prelude::*;
use bs_regles::textes::libelles::nombre;

use super::{Hud, absolu};
use crate::PartieEnCours;
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF, VERT_GAIN};
use crate::ecrans::nuit::{Style, ecrit};
use crate::options::Options;
use crate::polices::PolicesMenu;
use crate::textures::Textures;

/// Lignes au plus (les armes au-delà ne sont pas montrées).
const LIGNES: usize = 8;
const LARGEUR: f32 = 380.0;
const ICONE: f32 = 22.0;
const PERIODE: f32 = 0.5;

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

/// Entités du compteur.
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

fn texte(
    commandes: &mut Commands,
    p: &PolicesMenu,
    parent: Entity,
    (style, taille, couleur): (Style, f32, Color),
) -> Entity {
    ecrit(commandes, p, parent, "", (style, taille, couleur))
}

fn ligne(commandes: &mut Commands, p: &PolicesMenu, parent: Entity) -> Ligne {
    let racine = commandes
        .spawn((
            Node {
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
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
    // Nom et chiffres, puis le rail arrondi relatif à la meilleure arme
    let colonne = commandes
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    let haut = commandes
        .spawn((
            Node {
                align_items: AlignItems::Baseline,
                column_gap: Val::Px(8.0),
                ..default()
            },
            ChildOf(colonne),
        ))
        .id();
    let nom = texte(commandes, p, haut, (Style::Grasse, 14.0, IVOIRE));
    // Espace entre le nom et les chiffres (le texte lui-même ne s'étire pas)
    commandes.spawn((
        Node {
            flex_grow: 1.0,
            ..default()
        },
        ChildOf(haut),
    ));
    let valeur = texte(commandes, p, haut, (Style::Grasse, 14.0, IVOIRE));
    let dps = texte(commandes, p, haut, (Style::Normale, 12.0, AIDE));
    let part = texte(commandes, p, haut, (Style::Normale, 12.0, AIDE));
    let rail = commandes
        .spawn((
            Node {
                height: Val::Px(4.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
            ChildOf(colonne),
        ))
        .id();
    let barre = commandes
        .spawn((
            Node {
                width: Val::Percent(0.0),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(VERT_GAIN),
            ChildOf(rail),
        ))
        .id();
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

/// Construit le compteur (vide) dans le HUD : lueur de nuit venue du bord droit, sans
/// cadre.
pub fn construire(commandes: &mut Commands, p: &PolicesMenu, parent: Entity) -> Compteur {
    let racine = commandes
        .spawn((
            Name::new("compteur de dégâts"),
            Node {
                top: Val::Px(64.0),
                right: Val::Px(0.0),
                width: Val::Px(LARGEUR),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::new(Val::Px(40.0), Val::Px(16.0), Val::Px(10.0), Val::Px(12.0)),
                border_radius: BorderRadius::left(Val::Px(24.0)),
                ..absolu()
            },
            BackgroundGradient::from(LinearGradient::to_right(vec![
                ColorStop::new(Color::NONE, Val::Percent(0.0)),
                ColorStop::new(Color::srgba_u8(5, 8, 20, 170), Val::Percent(35.0)),
                ColorStop::new(Color::srgba_u8(5, 8, 20, 200), Val::Percent(100.0)),
            ])),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();
    let titre = commandes
        .spawn((
            Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    ecrit(commandes, p, titre, "DÉGÂTS", (Style::Grasse, 12.0, OR_VIF));
    let touche = commandes
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(1.0)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
            ChildOf(titre),
        ))
        .id();
    ecrit(commandes, p, touche, "Tab", (Style::Grasse, 11.0, AIDE));
    let lignes = std::array::from_fn(|_| ligne(commandes, p, racine));
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
    ecrit(commandes, p, pied, "Total", (Style::Normale, 13.0, AIDE));
    let total = texte(commandes, p, pied, (Style::Grasse, 13.0, IVOIRE));
    Compteur {
        racine,
        lignes,
        total,
        minuterie: 0.0,
    }
}

/// Remplace le texte s'il a changé.
fn ecrire(t: &mut Mut<Text>, texte: &str) {
    if t.0 != texte {
        t.0.clear();
        t.0.push_str(texte);
    }
}

/// Tab bascule l'option ; le compteur suit l'option et se réécrit deux fois par seconde.
#[allow(clippy::too_many_arguments)]
pub fn afficher(
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    jeu: Res<PartieEnCours>,
    hud: Option<ResMut<Hud>>,
    mut options: ResMut<Options>,
    cartes: Res<crate::ecran_cartes::EcranCartes>,
    touches: Res<ButtonInput<KeyCode>>,
    temps: Res<Time<Real>>,
    mut visibles: Query<&mut Visibility>,
    mut noeuds: Query<&mut Node>,
    mut textes: Query<&mut Text>,
    mut images: Query<&mut ImageNode>,
) {
    let Some(mut hud) = hud else {
        return;
    };
    if touches.just_pressed(KeyCode::Tab) {
        options.compteur_degats = !options.compteur_degats;
    }
    let c = &mut hud.compteur;
    // Masqué pendant les choix (cartes de niveau, coffres) : rien ne gêne la lecture
    let actif = options.compteur_degats && !cartes.ouvert();
    if let Ok(mut v) = visibles.get_mut(c.racine) {
        v.set_if_neq(if actif {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    if (options.is_changed() || cartes.is_changed()) && actif {
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
