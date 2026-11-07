//! Les autres joueurs de la coop à l'écran (src/entites/joueursDistants.ts) : chacun avec
//! son personnage et sa skin (celle de sa classe, une vanilla, ou l'importée reçue dans sa
//! présentation), animé comme le joueur, avec ombre, chute à la mort et teinte des coups ;
//! son pseudo flotte au-dessus de lui (`hud::etiquette`). Le coéquipier `n` est le
//! combattant `k = n + 1` de la partie (ordre de la liste figée, sans soi).

use bevy::prelude::*;
use bs_donnees::{Apparence, Donnees};
use bs_regles::Partie;
use bs_sauvegarde::profil_joueur::{ChoixSkin, chemin_skin_vanilla};

use crate::coop::Coop;
use crate::effets::Ombres;
use crate::modeles::Rendus;
use crate::modeles::animation::{
    EtatAnimation, angle_chute, calculer_rotations, hauteur_flottement,
};
use crate::modeles::rendu::Composants;
use crate::textures::Textures;

/// Durée de la chute d'un joueur mort (config.ts `joueur.dureeMort`).
const DUREE_MORT: f32 = 0.6;
/// Teinte rouge d'un joueur touché ou mort (config.ts `blessure`).
const ROUGE: [f32; 4] = [1.0, 0.0, 0.0, 0.35];
/// Fenêtre de teinte rouge après un coup (s).
const DUREE_BLESSURE: f32 = 0.3;

pub struct Coequipier {
    pub pseudo: String,
    pub apparence: &'static Apparence,
}

/// Coéquipiers de la partie en cours (vide en solo).
#[derive(Resource, Default)]
pub struct Coequipiers(pub Vec<Coequipier>);

/// Positions au pas précédent et animations, par coéquipier.
#[derive(Resource, Default)]
pub struct Suivi {
    precedentes: Vec<Vec2>,
    animations: Vec<EtatAnimation>,
}

/// Apparence d'un joueur d'après sa présentation (`place` : nom de sa skin importée).
fn apparence(
    d: &'static Donnees,
    j: &bs_protocole::coop::InfoJoueur,
    (serveur, textures, images): (&AssetServer, &mut Textures, &mut Assets<Image>),
) -> &'static Apparence {
    let p = &j.presentation;
    let index = bs_reseau::coop::index_personnage(d, &p.personnage);
    let classe = &d.personnages.personnages[index].apparence;
    let skin = match ChoixSkin::depuis_id(&p.skin) {
        ChoixSkin::Classe => None,
        ChoixSkin::Vanilla(k) => Some(chemin_skin_vanilla(k, p.bras_fins)),
        ChoixSkin::Importe => p
            .skin_importe
            .as_deref()
            .and_then(crate::skins::image_skin)
            .map(|image| {
                let chemin = format!("coop/skin-{}", j.index);
                textures.inserer(&chemin, images.add(image));
                chemin
            }),
    };
    let apparence = match skin {
        None => classe,
        Some(skin) => {
            let modele = if p.bras_fins {
                "humanoide_fin"
            } else {
                "humanoide"
            };
            let a = Apparence {
                modele: modele.into(),
                skin,
                ..classe.clone()
            };
            &*Box::leak(Box::new(a))
        }
    };
    textures.demander(serveur, &apparence.skin);
    if let Some(o) = &apparence.objet_en_main {
        textures.demander(serveur, o);
    }
    apparence
}

/// Entrée au chargement : les coéquipiers de la coop et leurs textures.
pub fn preparer(
    mut commandes: Commands,
    coop: Option<Res<Coop>>,
    dj: Res<crate::lancement::DonneesJeu>,
    serveur: Res<AssetServer>,
    mut textures: ResMut<Textures>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut liste = Vec::new();
    if let Some(coop) = coop {
        let place = coop.place();
        for j in coop.joueurs().iter().filter(|j| j.index != place) {
            let a = apparence(dj.d, j, (&serveur, &mut textures, &mut images));
            liste.push(Coequipier {
                pseudo: j.presentation.pseudo.clone(),
                apparence: a,
            });
        }
    }
    commandes.insert_resource(Coequipiers(liste));
    commandes.insert_resource(Suivi::default());
}

/// Coéquipiers affichables : présents dans la partie et dans les rendus.
fn nombre(p: &Partie, rendus: &Rendus) -> usize {
    (p.nombre_joueurs() - 1).min(rendus.nombre_autres())
}

/// Avant chaque pas : positions courantes, qui deviennent les précédentes.
pub fn noter(p: &Partie, rendus: &Rendus, s: &mut Suivi) {
    let n = nombre(p, rendus);
    s.precedentes.resize(n, Vec2::ZERO);
    for (k, prec) in s.precedentes.iter_mut().enumerate() {
        let j = p.etat_joueur(k + 1);
        *prec = Vec2::new(j.x, j.z);
    }
}

/// Après chaque pas : marche ou repos.
pub fn animer(p: &Partie, rendus: &Rendus, s: &mut Suivi, pas: f32) {
    let n = nombre(p, rendus);
    while s.animations.len() < n {
        s.animations
            .push(EtatAnimation::nouveau(0.25 * s.animations.len() as f32));
    }
    for (k, etat) in s.animations.iter_mut().enumerate().take(n) {
        let j = p.etat_joueur(k + 1);
        etat.avancer(rendus.modele_autre(k), pas, Vec2::new(j.vx, j.vz).length());
    }
}

/// Pose chaque coéquipier entre ses deux dernières positions (`t` : fraction du pas).
#[allow(clippy::too_many_arguments)]
pub fn placer(
    p: &Partie,
    (s, c): (&Suivi, &Coequipiers),
    t: f32,
    invulnerabilite: f32,
    rendus: &mut Rendus,
    commandes: &mut Commands,
    q: &mut Composants,
    ombres: &mut Ombres,
) {
    let n = nombre(p, rendus).min(s.animations.len()).min(c.0.len());
    for k in 0..n {
        let j = p.etat_joueur(k + 1);
        let maintenant = Vec2::new(j.x, j.z);
        let ici = s
            .precedentes
            .get(k)
            .map_or(maintenant, |a| a.lerp(maintenant, t));
        let etat = &s.animations[k];
        let r = rendus.autre(k);
        calculer_rotations(&r.modele, etat, &mut r.pose.rotations);
        let pose = &mut r.pose;
        let y_sol = p.carte.hauteur_lissee(ici.x, ici.y);
        pose.position = Vec3::new(ici.x, y_sol + hauteur_flottement(&r.modele, etat), ici.y);
        pose.angle = j.angle;
        pose.inclinaison = if j.mort {
            angle_chute(j.temps_mort, DUREE_MORT)
        } else {
            0.0
        };
        pose.echelle_xz = 1.0;
        pose.echelle_y = 1.0;
        let touche = j.invulnerabilite > invulnerabilite - DUREE_BLESSURE;
        pose.surcouche = if j.mort || touche { ROUGE } else { [0.0; 4] };
        r.ajouter(commandes, q);
        ombres
            .0
            .push([ici.x, ici.y, c.0[k].apparence.rayon_ombre, y_sol]);
    }
}
