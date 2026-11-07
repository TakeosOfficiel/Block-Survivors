//! Créatures des talents de l'Invocateur (src/combat/talents/invocateur.ts `majVisuel`) :
//! le ravageur de l'Évocation supérieure, qui charge en marchant, et les crocs de
//! l'évocateur, qui sortent de terre mâchoires ouvertes, claquent puis s'enfoncent.

use bevy::prelude::*;
use bs_commun::maths::atan2;
use bs_regles::classes::invocateur::{CROCS_CLAQUE, CROCS_VIE};
use bs_regles::partie::Partie;

use crate::PAS;
use crate::effets::Ombres;
use crate::modeles::Rendus;
use crate::modeles::animation::{EtatAnimation, calculer_rotations, hauteur_flottement};
use crate::modeles::rendu::Composants;

/// Ordre des créatures dans `allies.creatures_talents`.
const RAVAGEUR: &str = "ravageur";
const CROCS: &str = "crocs";
/// Sortie de terre, enfoncement, fermeture des mâchoires (s), profondeur (blocs) et
/// ouverture des mâchoires (rad).
const SORTIE: f32 = 0.12;
const ENFONCEMENT: f32 = 0.25;
const FERMETURE: f32 = 0.08;
const PROFONDEUR: f32 = 0.9;
const OUVERTURE: f32 = 1.1;

fn index(p: &Partie, nom: &str) -> Option<usize> {
    p.d.allies.creatures_talents.get_index_of(nom)
}

/// Après chaque pas : la marche du ravageur avance.
pub fn animer(p: &Partie, rendus: &Rendus, etat: &mut EtatAnimation) {
    let Some(v) = p.talents.classes.invocateur.as_ref() else {
        return;
    };
    let Some(k) = index(p, RAVAGEUR) else {
        return;
    };
    if v.ravageur.actif {
        etat.avancer(rendus.modele_creature(k), PAS, v.ravageur.vitesse_sol);
    }
}

/// Pose le ravageur et les crocs (entre `commencer` et `terminer` des rendus).
pub fn placer(
    p: &Partie,
    etat: &EtatAnimation,
    rendus: &mut Rendus,
    commandes: &mut Commands,
    q: &mut Composants,
    ombres: &mut Ombres,
) {
    let Some(v) = p.talents.classes.invocateur.as_ref() else {
        return;
    };
    let r = v.ravageur;
    if r.actif
        && let Some(k) = index(p, RAVAGEUR)
    {
        let rendu = rendus.creature(k);
        calculer_rotations(&rendu.modele, etat, &mut rendu.pose.rotations);
        let y = hauteur_flottement(&rendu.modele, etat) + p.carte.hauteur_lissee(r.x, r.z);
        let pose = &mut rendu.pose;
        pose.position = Vec3::new(r.x, y, r.z);
        pose.angle = atan2(r.dir_x, r.dir_z);
        pose.inclinaison = 0.0;
        pose.echelle_xz = 1.0;
        pose.echelle_y = 1.0;
        pose.surcouche = [0.0; 4];
        rendu.ajouter(commandes, q);
        let ombre = p.d.allies.creatures_talents[k].rayon_ombre;
        ombres
            .0
            .push([r.x, r.z, ombre, p.carte.hauteur_lissee(r.x, r.z)]);
    }
    let Some(k) = index(p, CROCS) else {
        return;
    };
    let rendu = rendus.creature(k);
    let partie = |nom: &str| rendu.modele.parties.iter().position(|x| x.nom == nom);
    let (machoire_a, machoire_b) = (partie("machoireA"), partie("machoireB"));
    for c in v.crocs.iter().filter(|c| c.t >= 0.0) {
        let sortie = (c.t / SORTIE).min((CROCS_VIE - c.t) / ENFONCEMENT).min(1.0);
        let ouverture = if c.t < CROCS_CLAQUE {
            1.0
        } else {
            (1.0 - (c.t - CROCS_CLAQUE) / FERMETURE).max(0.0)
        };
        let pose = &mut rendu.pose;
        pose.rotations.copy_from_slice(&rendu.modele.pose_base);
        if let Some(a) = machoire_a {
            pose.rotations[a].z = -ouverture * OUVERTURE;
        }
        if let Some(b) = machoire_b {
            pose.rotations[b].z = ouverture * OUVERTURE;
        }
        pose.position = Vec3::new(
            c.x,
            p.carte.hauteur(c.x, c.z) - PROFONDEUR * (1.0 - sortie),
            c.z,
        );
        pose.angle = c.angle;
        pose.inclinaison = 0.0;
        pose.echelle_xz = 1.0;
        pose.echelle_y = 1.0;
        pose.surcouche = [0.0; 4];
        rendu.ajouter(commandes, q);
    }
}
