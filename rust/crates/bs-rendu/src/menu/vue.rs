//! Construction du menu principal (src/ui/ecranMenu.ts) : classes à gauche avec les
//! détails de la choisie, héros au centre (pseudo, maîtrise, talents), arènes à droite
//! (verrous, mobs, boss, records), puis le grand bouton « Jouer ». Refait en entier à
//! chaque changement de choix (hors de la boucle de jeu).

use bevy::prelude::*;
use bs_donnees::arenes::Arene;
use bs_donnees::progression::TypeDeblocage;
use bs_regles::textes::libelles::{lignes_bonus, nombre};

use super::ActionMenu;
use crate::ecrans::style::{self, GRIS, OR, OR_CLAIR, TEXTE, Teinte};
use crate::hud::formater_temps;
use crate::hud::police::TextePixel;
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::sauvegarde::Sauvegarde;

const FOND: Color = Color::srgb_u8(0x14, 0x17, 0x1f);
const LARGEUR_COTE: f32 = 320.0;
const JAUNE: Color = Color::srgb_u8(0xff, 0xe0, 0x66);
const ROUGE_VERROU: Color = Color::srgb_u8(0xd0, 0x80, 0x70);
const CHOISI: Teinte = Teinte {
    bord: OR,
    fond: Color::srgba_u8(60, 45, 10, 153),
};

fn texte(commandes: &mut Commands, parent: Entity, t: &str, couleur: Color, taille: f32) -> Entity {
    commandes
        .spawn((TextePixel::nouveau(t, couleur, taille), ChildOf(parent)))
        .id()
}

/// Texte coupé à la largeur d'une colonne.
fn paragraphe(commandes: &mut Commands, parent: Entity, t: &str, couleur: Color, taille: f32) {
    commandes.spawn((
        TextePixel::nouveau(t, couleur, taille).bloc(LARGEUR_COTE),
        ChildOf(parent),
    ));
}

fn colonne(commandes: &mut Commands, parent: Entity, largeur: Val, ecart: f32) -> Entity {
    commandes
        .spawn((
            Node {
                width: largeur,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(ecart),
                ..default()
            },
            ChildOf(parent),
        ))
        .id()
}

/// Bouton de liste (classe ou arène) : deux lignes, mis en avant s'il est choisi.
fn bouton_liste(
    commandes: &mut Commands,
    parent: Entity,
    (titre, sous_titre): (&str, &str),
    choisi: bool,
    action: Option<ActionMenu>,
) {
    let b = style::bouton(commandes, parent, titre, LARGEUR_COTE, false, ());
    let mut e = commandes.entity(b);
    e.entry::<Node>().and_modify(|mut n| {
        n.flex_direction = FlexDirection::Column;
        n.height = Val::Auto;
        n.padding = UiRect::axes(Val::Px(8.0), Val::Px(6.0));
        n.row_gap = Val::Px(4.0);
    });
    match action {
        Some(a) => {
            e.insert(a);
        }
        None => {
            e.remove::<(Button, style::BoutonPage)>();
        }
    }
    if choisi {
        e.insert((
            CHOISI,
            BorderColor::all(CHOISI.bord),
            BackgroundColor(CHOISI.fond),
        ));
    }
    let couleur = if action.is_some() { GRIS } else { ROUGE_VERROU };
    commandes.spawn((
        TextePixel::nouveau(sous_titre, couleur, 1.5).bloc(LARGEUR_COTE - 16.0),
        ChildOf(b),
    ));
}

/// Construit le menu ; renvoie sa racine.
pub fn construire(
    commandes: &mut Commands,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
) -> Entity {
    let racine = commandes
        .spawn((
            Name::new("menu"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(FOND),
        ))
        .id();
    texte(commandes, racine, "Block Survivors", OR_CLAIR, 5.0);
    let milieu = commandes
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(24.0),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    classes(commandes, dj, sauvegarde, choix, milieu);
    heros(commandes, dj, sauvegarde, choix, milieu);
    arenes(commandes, dj, sauvegarde, choix, milieu);

    let bas = commandes
        .spawn((
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(16.0),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    style::bouton(commandes, bas, "Quitter", 160.0, false, ActionMenu::Quitter);
    let jouer = style::bouton(commandes, bas, "Jouer", 320.0, true, ActionMenu::Jouer);
    commandes.entity(jouer).entry::<Node>().and_modify(|mut n| {
        n.flex_direction = FlexDirection::Column;
        n.height = Val::Px(64.0);
    });
    let d = dj.d;
    let nom = &d.personnages.personnages[choix.personnage].nom;
    let sous_titre = format!("{nom} · {}", dj.arenes[choix.arene].def.nom);
    texte(commandes, jouer, &sous_titre, GRIS, 1.5);
    let version = format!("Version {} · Entrée : jouer", bs_commun::VERSION);
    texte(commandes, bas, &version, GRIS, 1.5);
    racine
}

fn classes(
    commandes: &mut Commands,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    parent: Entity,
) {
    let d = dj.d;
    let col = colonne(commandes, parent, Val::Px(LARGEUR_COTE), 6.0);
    texte(commandes, col, "Classes", OR, 2.0);
    for (k, (id, p)) in d.personnages.personnages.iter().enumerate() {
        let verrou = sauvegarde.progression.verrou(TypeDeblocage::Personnage, id);
        let sous_titre =
            verrou.map_or_else(|| p.role.clone(), |v| format!("Verrouillée · {}", v.texte));
        let action = verrou.is_none().then_some(ActionMenu::Classe(k));
        bouton_liste(
            commandes,
            col,
            (&p.nom, &sous_titre),
            k == choix.personnage,
            action,
        );
    }
    let p = &d.personnages.personnages[choix.personnage];
    paragraphe(commandes, col, &p.description, TEXTE, 1.5);
    let arme = p
        .armes_depart
        .first()
        .and_then(|a| d.armes.get(a))
        .map_or("", |a| a.nom.as_str());
    let tuiles = format!(
        "{} cœurs · Vitesse {} · {arme}",
        nombre(p.pv / 2.0),
        nombre(p.vitesse)
    );
    texte(commandes, col, &tuiles, JAUNE, 1.5);
    for ligne in lignes_bonus(&p.bonus) {
        texte(commandes, col, &ligne, GRIS, 1.5);
    }
}

fn heros(
    commandes: &mut Commands,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    parent: Entity,
) {
    let d = dj.d;
    let col = colonne(commandes, parent, Val::Auto, 8.0);
    commandes.entity(col).entry::<Node>().and_modify(|mut n| {
        n.flex_grow = 1.0;
        n.justify_content = JustifyContent::Center;
    });
    let (id, p) = d
        .personnages
        .personnages
        .get_index(choix.personnage)
        .expect("classe choisie");
    texte(commandes, col, &sauvegarde.profil.pseudo, TEXTE, 3.0);
    let prog = &sauvegarde.progression;
    let m = prog.maitrise(id);
    let arbre = prog.arbre_actif(id);
    let libres = prog.points_libres(id, arbre);
    let ligne = format!("{} · Maîtrise {}", p.nom, m.niveau);
    texte(commandes, col, &ligne, OR_CLAIR, 2.0);
    if libres > 0 {
        let s = if libres > 1 { "s" } else { "" };
        let t = format!("{libres} point{s} de talent à dépenser");
        texte(commandes, col, &t, JAUNE, 1.5);
    }
    if let Some(a) = d.talents.arbres.get(id).and_then(|a| a.get(arbre)) {
        let rangs = prog.points(id, arbre, None);
        let s = if rangs > 1 { "s" } else { "" };
        let t = format!("Talents : {} ({rangs} rang{s})", a.nom);
        texte(commandes, col, &t, GRIS, 1.5);
    }
}

fn arenes(
    commandes: &mut Commands,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    parent: Entity,
) {
    let d = dj.d;
    let col = colonne(commandes, parent, Val::Px(LARGEUR_COTE), 6.0);
    texte(commandes, col, "Arènes", OR, 2.0);
    let prog = &sauvegarde.progression;
    for (k, a) in dj.arenes.iter().enumerate() {
        let verrou = prog.verrou(TypeDeblocage::Arene, &a.def.id);
        let sous_titre = match verrou {
            Some(v) => format!("Verrouillée · {}", v.texte),
            None => match prog.record(&a.def.id) {
                r if r > 0.0 => format!("Record : {}", formater_temps(r as i64)),
                _ => "Jamais jouée".to_owned(),
            },
        };
        let action = verrou.is_none().then_some(ActionMenu::Arene(k));
        bouton_liste(
            commandes,
            col,
            (&a.def.nom, &sous_titre),
            k == choix.arene,
            action,
        );
    }
    let a = dj.arenes[choix.arene].def;
    paragraphe(commandes, col, &a.description, TEXTE, 1.5);
    let mobs = format!("Mobs : {}", noms_mobs(d, a));
    paragraphe(commandes, col, &mobs, GRIS, 1.5);
    for b in prochains_boss(d, a) {
        texte(commandes, col, &b, ROUGE_VERROU, 1.5);
    }
    let sans_fin = prog.record_sans_fin(&a.id);
    if sans_fin > 0.0 {
        let t = format!("Record sans fin : {}", formater_temps(sans_fin as i64));
        texte(commandes, col, &t, JAUNE, 1.5);
    }
}

/// Noms des mobs de l'arène (toutes zones), dans l'ordre d'arrivée.
fn noms_mobs(d: &bs_donnees::Donnees, a: &Arene) -> String {
    let mut tous: Vec<_> = a
        .ennemis
        .iter()
        .chain(a.zones.iter().flat_map(|z| &z.ennemis))
        .collect();
    tous.sort_by(|x, y| x.depuis.total_cmp(&y.depuis));
    let mut noms: Vec<String> = Vec::new();
    for e in tous {
        let nom = d
            .ennemis
            .get(&e.ennemi)
            .map_or(e.ennemi.clone(), |x| x.nom.to_lowercase());
        if !noms.contains(&nom) {
            noms.push(nom);
        }
    }
    noms.join(", ")
}

/// « 5:00 · Géant », « 10:00 · Seigneur wither »… jusqu'à la fin de la partie.
fn prochains_boss(d: &bs_donnees::Donnees, a: &Arene) -> Vec<String> {
    let c = &d.regles.boss;
    let duree = d.regles.progression.duree_partie;
    let mut liste = Vec::new();
    if a.boss.is_empty() || c.intervalle <= 0.0 {
        return liste;
    }
    let mut t = c.premier_passage;
    let mut i = 0;
    while t <= duree {
        let id = &a.boss[i % a.boss.len()];
        let nom = d.ennemis.get(id).map_or(id.as_str(), |e| e.nom.as_str());
        liste.push(format!("{} · {nom}", formater_temps(t as i64)));
        t += c.intervalle;
        i += 1;
    }
    liste
}
