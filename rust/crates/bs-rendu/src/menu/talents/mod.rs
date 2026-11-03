//! Page des Talents du menu (docs/CHARTE-UI.md) : en-tête (titre, points disponibles
//! bien visibles), onglets des classes, les deux spécialisations (arbres) avec celle
//! équipée en partie, puis l'arbre (`schema`, 65 %, défilant) et la fiche du talent
//! consulté (`fiche`, 35 %) ; « Retour » en bas à gauche, « Réinitialiser » en bas à
//! droite, avec confirmation. Échap ramène au menu.

mod fiche;
mod schema;

use bevy::prelude::*;
use bs_donnees::progression::TypeDeblocage;
use bs_donnees::progression::{Arbre, Noeud};

use super::{ActionMenu, Icones};
use crate::ecrans::charte::{
    self, AIDE, BORD, IVOIRE, MARGE, OR_VIF, ROUGE, Relief, T_AIDE, T_CORPS, T_SECTION, VERT_GAIN,
};
use crate::ecrans::style::{OR, Teinte};
use crate::hud::police::TextePixel;
use crate::lancement::DonneesJeu;
use crate::sauvegarde::Sauvegarde;

/// Bouton choisi des autres pages (Collection, Profil).
pub(in crate::menu) const CHOISI: Teinte = Teinte {
    bord: OR,
    fond: Color::srgba_u8(60, 45, 10, 153),
};

/// Classe, arbre et talent affichés ; réinitialisation en attente de confirmation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EtatTalents {
    pub personnage: usize,
    pub arbre: usize,
    pub noeud: usize,
    pub confirmation: bool,
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
    Annuler,
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

/// Applique une action ; vrai si la page est à refaire. Toute autre action qu'une
/// réinitialisation confirmée annule la confirmation en attente.
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
    let confirmee = std::mem::take(&mut etat.confirmation);
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
        ActionTalent::Acheter => {
            prog.acheter(p, a, n);
        }
        ActionTalent::Utiliser => prog.choisir_arbre(p, a),
        ActionTalent::Reinitialiser if confirmee => prog.reinitialiser(p, a),
        ActionTalent::Reinitialiser => etat.confirmation = true,
        ActionTalent::Annuler => {}
        ActionTalent::Option(i) => prog.choisir_option(p, a, n, i),
        ActionTalent::ActiverFinal => prog.choisir_final(p, a, n),
        ActionTalent::BasculerArme => {
            let actif = prog.actif(p, a, n);
            prog.choisir_arme_depart(p, a, (!actif).then_some(n));
        }
    }
    true
}

/// Texte Inter de la page.
pub(super) fn ecrire(
    commandes: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    (gras, taille, couleur): (bool, f32, Color),
) -> Entity {
    let t = ic.polices.texte(t, gras, taille, couleur);
    commandes.spawn((t, ChildOf(parent))).id()
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

/// Bouton en relief de la page ; sans action, il est éteint (fond creusé, texte d'aide).
pub(super) fn bouton(
    commandes: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    relief: Relief,
    action: Option<ActionTalent>,
) -> Entity {
    let h = charte::H_SECONDAIRE;
    let b = charte::bouton_relief(commandes, parent, relief, h, ());
    let couleur = match action {
        Some(a) => {
            commandes.entity(b).insert(ActionMenu::Talent(a));
            IVOIRE
        }
        None => {
            commandes
                .entity(b)
                .remove::<(Button, Relief)>()
                .insert((BackgroundColor(charte::CREUX), BorderColor::all(BORD)));
            AIDE
        }
    };
    ecrire(commandes, ic, b, t, (true, T_CORPS, couleur));
    b
}

/// Liseré d'or de 3 au bas d'un onglet choisi.
fn lisere_bas(commandes: &mut Commands, parent: Entity) {
    commandes.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(-2.0),
            right: Val::Px(-2.0),
            bottom: Val::Px(-4.0),
            height: Val::Px(3.0),
            ..default()
        },
        BackgroundColor(OR_VIF),
        ChildOf(parent),
    ));
}

/// Construit la page ; renvoie sa racine.
pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones) -> Entity {
    let racine = commandes
        .spawn((
            Name::new("talents"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(MARGE), Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(charte::FOND),
        ))
        .id();
    entete(commandes, c, ic, racine);
    classes(commandes, c, ic, racine);
    specialisations(commandes, c, ic, racine);
    let corps = commandes
        .spawn((
            Node {
                flex_grow: 1.0,
                min_height: Val::ZERO,
                column_gap: Val::Px(20.0),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    schema::construire(commandes, c, ic, corps);
    fiche::construire(commandes, c, ic, corps);
    pied(commandes, c, ic, racine);
    racine
}

/// Titre à gauche ; maîtrise et points disponibles à droite.
fn entete(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, racine: Entity) {
    let d = c.dj.d;
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let e = rangee(commandes, racine, 16.0);
    commandes.entity(e).entry::<Node>().and_modify(|mut n| {
        n.justify_content = JustifyContent::SpaceBetween;
        n.min_height = Val::Px(48.0);
    });
    commandes.spawn((TextePixel::nouveau("Talents", IVOIRE, 4.0), ChildOf(e)));
    let droite = rangee(commandes, e, 12.0);
    commandes.spawn((
        ImageNode::new(ic.icone(&d.talents.maitrise.icone)),
        Node {
            width: Val::Px(32.0),
            height: Val::Px(32.0),
            ..default()
        },
        ChildOf(droite),
    ));
    let perso = &d.personnages.personnages[c.etat.personnage];
    let niveau = format!("{} · Maîtrise {}", perso.nom, prog.maitrise(p).niveau);
    ecrire(commandes, ic, droite, &niveau, (false, T_CORPS, AIDE));
    let libres = prog.points_libres(p, a);
    let s = if libres > 1 { "s" } else { "" };
    let t = format!("{libres} point{s} disponible{s}");
    let couleur = if libres > 0 { OR_VIF } else { IVOIRE };
    ecrire(commandes, ic, droite, &t, (true, T_SECTION, couleur));
}

/// Onglets des classes (icône d'arme, nom ; soulignés d'or si choisis).
fn classes(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, racine: Entity) {
    let d = c.dj.d;
    let prog = &c.sauvegarde.progression;
    let r = rangee(commandes, racine, 8.0);
    for (k, (id, perso)) in d.personnages.personnages.iter().enumerate() {
        let libre = prog.verrou(TypeDeblocage::Personnage, id).is_none();
        let choisie = k == c.etat.personnage;
        let action = libre.then_some(ActionTalent::Classe(k));
        let relief = if choisie {
            Relief::CHOISI
        } else {
            Relief::SECONDAIRE
        };
        let b = bouton(commandes, ic, r, &perso.nom, relief, action);
        commandes
            .entity(b)
            .entry::<Node>()
            .and_modify(|mut n| n.min_width = Val::Px(150.0));
        let icone = perso.apparence.objet_en_main.as_deref().or_else(|| {
            let arme = perso.armes_depart.first().and_then(|a| d.armes.get(a));
            arme.map(|a| a.icone.as_str())
        });
        if let Some(i) = icone {
            let e = commandes
                .spawn((
                    ImageNode::new(ic.icone(i)),
                    Node {
                        width: Val::Px(24.0),
                        height: Val::Px(24.0),
                        ..default()
                    },
                ))
                .id();
            commandes.entity(b).insert_children(0, &[e]);
        }
        if !libre {
            charte::cadenas(commandes, b, 12.0, AIDE);
        }
        if choisie {
            lisere_bas(commandes, b);
        }
    }
}

/// Les deux spécialisations (arbres) : icône, nom, rangs, « Équipée » sur celle jouée ;
/// à droite, « Équiper » quand on consulte l'autre.
fn specialisations(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, racine: Entity) {
    let d = c.dj.d;
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let actif = prog.arbre_actif(p);
    let r = rangee(commandes, racine, 8.0);
    for (k, def) in d.talents.arbres[p].iter().enumerate() {
        let relief = if k == a {
            Relief::CHOISI
        } else {
            Relief::SECONDAIRE
        };
        let b = charte::bouton_relief(
            commandes,
            r,
            relief,
            52.0,
            ActionMenu::Talent(ActionTalent::Arbre(k)),
        );
        commandes.entity(b).entry::<Node>().and_modify(|mut n| {
            n.min_width = Val::Px(260.0);
            n.justify_content = JustifyContent::FlexStart;
            n.column_gap = Val::Px(10.0);
        });
        commandes.spawn((
            ImageNode::new(ic.icone(&def.icone)),
            Node {
                width: Val::Px(28.0),
                height: Val::Px(28.0),
                ..default()
            },
            ChildOf(b),
        ));
        ecrire(commandes, ic, b, &def.nom, (true, T_CORPS + 1.0, IVOIRE));
        let rangs = prog.points(p, k, None);
        let s = if rangs > 1 { "s" } else { "" };
        ecrire(
            commandes,
            ic,
            b,
            &format!("{rangs} rang{s}"),
            (false, T_AIDE, AIDE),
        );
        if k == actif {
            let badge = commandes
                .spawn((
                    Node {
                        margin: UiRect::left(Val::Px(8.0)),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(charte::VERT_TRANCHE),
                    ChildOf(b),
                ))
                .id();
            ecrire(commandes, ic, badge, "✓ Équipée", (true, T_AIDE, VERT_GAIN));
        }
        if k == a {
            lisere_bas(commandes, b);
        }
    }
    if a != actif {
        let t = format!("Équiper {}", d.talents.arbres[p][a].nom);
        let b = bouton(
            commandes,
            ic,
            r,
            &t,
            Relief::PRINCIPAL,
            Some(ActionTalent::Utiliser),
        );
        commandes
            .entity(b)
            .entry::<Node>()
            .and_modify(|mut n| n.margin = UiRect::left(Val::Px(12.0)));
    }
}

/// « Retour » à gauche ; « Réinitialiser » à droite, en deux temps.
fn pied(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, racine: Entity) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let r = rangee(commandes, racine, 8.0);
    commandes.entity(r).entry::<Node>().and_modify(|mut n| {
        n.justify_content = JustifyContent::SpaceBetween;
    });
    let retour = charte::bouton_relief(
        commandes,
        r,
        Relief::SECONDAIRE,
        charte::H_SECONDAIRE,
        ActionMenu::Fermer,
    );
    commandes
        .entity(retour)
        .entry::<Node>()
        .and_modify(|mut n| n.min_width = Val::Px(160.0));
    ecrire(commandes, ic, retour, "Retour", (true, T_CORPS, IVOIRE));
    let droite = rangee(commandes, r, 8.0);
    let valeur = prog.points(p, a, None);
    let s = if valeur > 1 { "s" } else { "" };
    if c.etat.confirmation {
        let t = format!("Rendre {valeur} point{s} de {} ?", c.arbre().nom);
        ecrire(commandes, ic, droite, &t, (true, T_CORPS, ROUGE));
        bouton(
            commandes,
            ic,
            droite,
            "Annuler",
            Relief::SECONDAIRE,
            Some(ActionTalent::Annuler),
        );
        let rouge = Relief {
            bord: ROUGE,
            tranche: ROUGE,
            ..Relief::SECONDAIRE
        };
        bouton(
            commandes,
            ic,
            droite,
            "Confirmer",
            rouge,
            Some(ActionTalent::Reinitialiser),
        );
    } else {
        let action = (valeur > 0).then_some(ActionTalent::Reinitialiser);
        let t = if valeur > 0 {
            format!("Réinitialiser (+{valeur} point{s})")
        } else {
            "Réinitialiser".to_owned()
        };
        bouton(commandes, ic, droite, &t, Relief::SECONDAIRE, action);
    }
}
