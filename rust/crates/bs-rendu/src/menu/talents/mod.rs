//! Page des Talents du menu (src/ui/ecranTalents.ts) : pour chaque classe, deux arbres
//! en graphe, un seul actif en partie. Un clic sur un talent ouvre sa fiche : achat d'un
//! rang (un point de talent par niveau de maîtrise ; chaque arbre a tous les points),
//! option d'un talent à choix, talent final actif, arme de départ active. Échap ou
//! « Retour » ramène au menu.

mod fiche;
mod schema;

use bevy::prelude::*;
use bs_donnees::progression::TypeDeblocage;
use bs_donnees::progression::{Arbre, Noeud};

use super::{ActionMenu, Icones};
use crate::ecrans::style::{self, OR, OR_CLAIR, Teinte};
use crate::hud::police::TextePixel;
use crate::lancement::DonneesJeu;
use crate::sauvegarde::Sauvegarde;

pub(super) const JAUNE: Color = Color::srgb_u8(0xff, 0xff, 0x55);
pub(super) const ORANGE: Color = Color::srgb_u8(0xe0, 0xb0, 0x40);
pub(super) const LARGEUR_FICHE: f32 = 420.0;
const CHOISI: Teinte = Teinte {
    bord: OR,
    fond: Color::srgba_u8(60, 45, 10, 153),
};

/// Classe, arbre et talent affichés.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EtatTalents {
    pub personnage: usize,
    pub arbre: usize,
    pub noeud: usize,
}

/// Bouton de la page.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionTalent {
    Classe(usize),
    Arbre(usize),
    Noeud(usize),
    Acheter,
    Utiliser,
    Reinitialiser,
    Option(usize),
    ActiverFinal,
    BasculerArme,
}

/// Ce que la page lit pour se construire.
#[derive(Clone, Copy)]
pub struct Contexte<'a> {
    pub dj: &'a DonneesJeu,
    pub sauvegarde: &'a Sauvegarde,
    pub etat: EtatTalents,
}

impl Contexte<'_> {
    pub(super) fn id(&self) -> &'static str {
        let d = self.dj.d;
        d.personnages
            .personnages
            .get_index(self.etat.personnage)
            .map_or("", |(id, _)| id.as_str())
    }

    pub(super) fn arbre(&self) -> &'static Arbre {
        &self.dj.d.talents.arbres[self.id()][self.etat.arbre]
    }

    pub(super) fn noeud(&self) -> &'static Noeud {
        let noeuds = &self.arbre().noeuds;
        noeuds.get(self.etat.noeud).unwrap_or(&noeuds[0])
    }
}

/// Applique une action ; vrai si la page est à refaire.
pub fn agir(
    dj: &DonneesJeu,
    sauvegarde: &mut Sauvegarde,
    etat: &mut EtatTalents,
    action: ActionTalent,
) -> bool {
    let c = Contexte {
        dj,
        sauvegarde,
        etat: *etat,
    };
    let (p, a, n) = (c.id(), etat.arbre, c.noeud());
    let prog = &mut sauvegarde.progression;
    match action {
        ActionTalent::Classe(k) => {
            etat.personnage = k;
            let id =
                dj.d.personnages
                    .personnages
                    .get_index(k)
                    .map_or("", |(i, _)| i);
            etat.arbre = prog.arbre_actif(id);
            etat.noeud = 0;
        }
        ActionTalent::Arbre(k) => {
            etat.arbre = k;
            etat.noeud = 0;
        }
        ActionTalent::Noeud(k) => etat.noeud = k,
        ActionTalent::Acheter => return prog.acheter(p, a, n),
        ActionTalent::Utiliser => prog.choisir_arbre(p, a),
        ActionTalent::Reinitialiser => prog.reinitialiser(p, a),
        ActionTalent::Option(i) => prog.choisir_option(p, a, n, i),
        ActionTalent::ActiverFinal => prog.choisir_final(p, a, n),
        ActionTalent::BasculerArme => {
            let actif = prog.actif(p, a, n);
            prog.choisir_arme_depart(p, a, (!actif).then_some(n));
        }
    }
    true
}

pub(super) fn texte(
    commandes: &mut Commands,
    parent: Entity,
    t: &str,
    couleur: Color,
    taille: f32,
) {
    commandes.spawn((TextePixel::nouveau(t, couleur, taille), ChildOf(parent)));
}

pub(super) fn paragraphe(commandes: &mut Commands, parent: Entity, t: &str, couleur: Color) {
    commandes.spawn((
        TextePixel::nouveau(t, couleur, 1.5)
            .bloc(LARGEUR_FICHE - 28.0)
            .a_gauche(),
        ChildOf(parent),
    ));
}

pub(super) fn rangee(commandes: &mut Commands, parent: Entity, ecart: f32) -> Entity {
    commandes
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(ecart),
                ..default()
            },
            ChildOf(parent),
        ))
        .id()
}

/// Bouton compact (onglets, actions) ; `choisi` le met en avant, sans action il est
/// éteint.
pub(super) fn onglet(
    commandes: &mut Commands,
    parent: Entity,
    t: &str,
    choisi: bool,
    action: Option<ActionTalent>,
) -> Entity {
    let b = style::bouton(commandes, parent, t, 0.0, false, ());
    let mut e = commandes.entity(b);
    e.entry::<Node>().and_modify(|mut n| {
        n.width = Val::Auto;
        n.height = Val::Px(36.0);
        n.padding = UiRect::horizontal(Val::Px(12.0));
    });
    match action {
        Some(a) => {
            e.insert(ActionMenu::Talent(a));
        }
        None => {
            e.remove::<(Button, style::BoutonPage)>();
            e.insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)));
        }
    }
    if choisi {
        e.insert((
            CHOISI,
            BorderColor::all(CHOISI.bord),
            BackgroundColor(CHOISI.fond),
        ));
    }
    b
}

/// Construit la page ; renvoie sa racine.
pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones) -> Entity {
    let d = c.dj.d;
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let racine = commandes
        .spawn((
            Name::new("talents"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(super::vue::FOND),
        ))
        .id();
    // Classes et maîtrise
    let haut = rangee(commandes, racine, 24.0);
    let classes = rangee(commandes, haut, 6.0);
    for (k, (id, perso)) in d.personnages.personnages.iter().enumerate() {
        let libre = prog.verrou(TypeDeblocage::Personnage, id).is_none();
        let action = libre.then_some(ActionTalent::Classe(k));
        onglet(
            commandes,
            classes,
            &perso.nom,
            k == c.etat.personnage,
            action,
        );
    }
    let perso = &d.personnages.personnages[c.etat.personnage];
    let libres = prog.points_libres(p, a);
    let s = if libres > 1 { "s" } else { "" };
    let maitrise = format!(
        "{} · Maîtrise {} · {libres} point{s} libre{s}",
        perso.nom,
        prog.maitrise(p).niveau
    );
    texte(commandes, haut, &maitrise, OR_CLAIR, 2.0);

    // Arbres, arbre utilisé en partie, remise à zéro
    let entete = rangee(commandes, racine, 24.0);
    let arbres = rangee(commandes, entete, 6.0);
    let actif = prog.arbre_actif(p);
    for (k, def) in d.talents.arbres[p].iter().enumerate() {
        let rangs = prog.points(p, k, None);
        let s = if rangs > 1 { "s" } else { "" };
        let nom = if k == actif {
            format!("{} (en partie) · {rangs} point{s}", def.nom)
        } else {
            format!("{} · {rangs} point{s}", def.nom)
        };
        let b = onglet(
            commandes,
            arbres,
            &nom,
            k == a,
            Some(ActionTalent::Arbre(k)),
        );
        let icone = commandes
            .spawn((
                ImageNode::new(ic.icone(&def.icone)),
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    margin: UiRect::right(Val::Px(8.0)),
                    ..default()
                },
            ))
            .id();
        commandes.entity(b).insert_children(0, &[icone]);
    }
    let actions = rangee(commandes, entete, 6.0);
    if a == actif {
        onglet(commandes, actions, "Utilisé en partie", false, None);
    } else {
        let u = Some(ActionTalent::Utiliser);
        onglet(commandes, actions, "Utiliser en partie", false, u);
    }
    let valeur = prog.points(p, a, None);
    let (t, r) = if valeur > 0 {
        let s = if valeur > 1 { "s" } else { "" };
        let t = format!("Réinitialiser (+{valeur} point{s})");
        (t, Some(ActionTalent::Reinitialiser))
    } else {
        ("Réinitialiser".to_owned(), None)
    };
    onglet(commandes, actions, &t, false, r);

    let volets = rangee(commandes, racine, 16.0);
    commandes
        .entity(volets)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.align_items = AlignItems::Stretch;
        });
    schema::construire(commandes, c, ic, volets);
    fiche::construire(commandes, c, ic, volets);

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
        .and_modify(|mut n| {
            n.margin = UiRect::top(Val::Auto);
        });
    racine
}
