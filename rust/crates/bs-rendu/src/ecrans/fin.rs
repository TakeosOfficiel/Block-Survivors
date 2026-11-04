//! Écran de fin, au style des menus (`nuit`) : voile de nuit teinté d'or (victoire) ou
//! de rouge (défaite), grand titre, personnage et arène, grands chiffres du résumé,
//! maîtrise gagnée sur un rail vert, bilan défilant sans cadre, « REJOUER » en pilule
//! verte avec le focus (Entrée), « CONTINUER SANS FIN » en violet après une victoire
//! hors mode sans fin, « Menu principal » en lien.

use bevy::prelude::*;
use bs_regles::etat::Issue;
use bs_regles::partie::Partie;
use bs_sauvegarde::Recompense;

use super::Action;
use super::bilan::{self, Contexte};
use super::charte::{self, IVOIRE, OR_VIF, Relief, VERT_GAIN};
use super::nuit::{self, Style, nuit, pilule, voile};
use super::vues::panneau;

/// Fin de partie : titre, lieu, chiffres, maîtrise, bilan, actions.
pub fn fin(
    commandes: &mut Commands,
    c: &Contexte,
    p: &Partie,
    recompense: Option<&Recompense>,
    coop: bool,
) -> Entity {
    let e = &p.etat;
    let victoire = e.issue == Some(Issue::Victoire);
    let haut = if victoire {
        Color::srgba_u8(90, 66, 0, 150)
    } else {
        Color::srgba_u8(80, 6, 10, 160)
    };
    let r = voile(commandes, (haut, nuit(0.95)));
    commandes
        .entity(r)
        .entry::<Node>()
        .and_modify(|mut n| n.row_gap = Val::Px(14.0));
    let (titre, couleur) = if victoire {
        ("VICTOIRE !", OR_VIF)
    } else {
        ("VOUS ÊTES MORT !", charte::ROUGE)
    };
    nuit::titre(commandes, c.polices, r, titre, couleur);
    let lieu = format!("{} · {}", e.personnage.nom, e.arene.nom);
    let record = recompense.is_some_and(|r| r.record_sans_fin);
    let lieu = if record {
        let t = bilan::temps(e.temps_jeu);
        format!("Mode sans fin · {lieu} · nouveau record : {t} !")
    } else if e.sans_fin {
        format!("Mode sans fin · {lieu}")
    } else {
        lieu
    };
    let couleur = if record {
        OR_VIF
    } else {
        IVOIRE.with_alpha(0.8)
    };
    c.ecrit(commandes, r, &lieu, (Style::Grasse, 18.0, couleur));
    bilan::tuiles(commandes, c, r, p);
    if let Some(rec) = recompense {
        maitrise(commandes, c, r, p, rec);
    }
    panneau(commandes, c, r, p);
    let actions = nuit::rangee(commandes, r, 28.0);
    if !coop {
        let vert = (Relief::PRINCIPAL, true);
        pilule(
            commandes,
            c.polices,
            actions,
            "REJOUER",
            vert,
            Action::Rejouer,
        );
    }
    if victoire && !e.sans_fin {
        let t = "CONTINUER SANS FIN";
        let violet = (nuit::VIOLET, coop);
        pilule(commandes, c.polices, actions, t, violet, Action::Continuer);
    }
    let menu = (20.0, IVOIRE);
    nuit::lien(
        commandes,
        c.polices,
        actions,
        "Menu principal",
        menu,
        Action::Menu,
    );
    if !coop {
        nuit::aide(commandes, c.polices, r, "Entrée : rejouer");
    }
    r
}

/// Maîtrise gagnée : fiole, « Classe · Maîtrise N » et XP gagnée, rail vert du niveau,
/// puis points de talent, déblocages et succès en or.
fn maitrise(commandes: &mut Commands, c: &Contexte, parent: Entity, p: &Partie, r: &Recompense) {
    let bloc = nuit::rangee(commandes, parent, 18.0);
    commandes.entity(bloc).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(620.0);
        n.max_width = Val::Percent(96.0);
        n.margin = UiRect::vertical(Val::Px(6.0));
    });
    let icone = c.image(&p.d.talents.maitrise.icone, 36.0);
    commandes.spawn((icone, ChildOf(bloc)));
    let textes = nuit::colonne(commandes, bloc, 8.0);
    commandes
        .entity(textes)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.flex_grow = 1.0;
            n.flex_shrink = 1.0;
        });
    let a = r.apres;
    let entete = nuit::rangee(commandes, textes, 12.0);
    commandes
        .entity(entete)
        .entry::<Node>()
        .and_modify(|mut n| n.justify_content = JustifyContent::SpaceBetween);
    let nom = format!("{} · Maîtrise {}", p.etat.personnage.nom, a.niveau);
    c.ecrit(commandes, entete, &nom, (Style::Grasse, 18.0, IVOIRE));
    let xp = bs_regles::textes::libelles::nombre(r.xp as f32);
    let xp = format!("+{xp} XP");
    c.ecrit(commandes, entete, &xp, (Style::Logo, 22.0, VERT_GAIN));
    let rail = commandes
        .spawn((
            Node {
                height: Val::Px(6.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
            ChildOf(textes),
        ))
        .id();
    let part = (a.xp - a.debut_niveau) as f32 / (a.fin_niveau - a.debut_niveau).max(1) as f32;
    commandes.spawn((
        Node {
            width: Val::Percent(part.clamp(0.0, 1.0) * 100.0),
            height: Val::Percent(100.0),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(VERT_GAIN),
        ChildOf(rail),
    ));
    let or = (Style::Grasse, 15.0, OR_VIF);
    let niveaux = a.niveau - r.avant.niveau;
    if niveaux > 0 {
        let s = if niveaux > 1 { "s" } else { "" };
        let t = format!(
            "★ Niveau {} → {} : +{niveaux} point{s} de talent !",
            r.avant.niveau, a.niveau
        );
        c.ecrit(commandes, textes, &t, or);
    }
    for &i in &r.deblocages {
        let t = format!("★ Débloqué : {} !", p.d.talents.deblocages[i].nom);
        c.ecrit(commandes, textes, &t, or);
    }
    for &i in &r.succes {
        let s = &p.d.succes[i];
        let t = format!("★ Succès : {} ({})", s.nom, s.texte);
        c.ecrit(commandes, textes, &t, or);
    }
}
