//! Instantané de la partie, écrit par l'hôte pour un invité (src/reseau/hote.ts
//! `ecrireInstantane`) : l'en-tête (temps, niveau commun, émeraudes, issue), toute
//! l'équipe, puis ce qui est à moins de `RAYON_ENVOI` de lui (ennemis, boss toujours,
//! orbes, objets, coffres, projectiles ennemis, chiffres de dégâts). Écrit dans un
//! tampon réutilisé, sans allocation.

use bs_regles::Partie;
use bs_regles::etat::Issue;
use bs_regles::evenements::StyleChiffre;
use bs_regles::joueur::Commandes;

use super::octets::Ecrivain;

/// Premier octet des messages binaires.
pub const TYPE_ENTREE: u8 = 1;
pub const TYPE_INSTANTANE: u8 = 2;
/// Instantanés par seconde.
pub const INSTANTANES_PAR_SECONDE: f32 = 15.0;
/// Rayon autour de l'invité dans lequel on lui envoie ennemis et butin (blocs).
pub const RAYON_ENVOI: f32 = 34.0;
/// Chiffres de dégâts au plus par instantané.
pub const CHIFFRES_MAX: usize = 48;

/// Drapeaux d'un joueur.
pub const JOUEUR_MORT: u8 = 1;
pub const JOUEUR_PROTEGE: u8 = 2;
pub const JOUEUR_EN_CHOIX: u8 = 4;
pub const JOUEUR_PARTI: u8 = 8;
/// Drapeaux d'un ennemi.
pub const ENNEMI_MORT: u8 = 1;
pub const ENNEMI_BLESSE: u8 = 2;
pub const ENNEMI_RALENTI: u8 = 4;

/// Bits des touches dans une entrée.
pub fn bits(c: &Commandes) -> u8 {
    u8::from(c.haut) | u8::from(c.bas) << 1 | u8::from(c.gauche) << 2 | u8::from(c.droite) << 3
}

pub fn commandes(bits: u8, lacet: f32) -> Commandes {
    Commandes {
        haut: bits & 1 != 0,
        bas: bits & 2 != 0,
        gauche: bits & 4 != 0,
        droite: bits & 8 != 0,
        lacet,
    }
}

/// Chiffre de dégâts de l'hôte à montrer aux invités.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chiffre {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub valeur: f32,
    pub style: StyleChiffre,
}

pub const STYLES: [StyleChiffre; 4] = [
    StyleChiffre::Normal,
    StyleChiffre::Critique,
    StyleChiffre::Feu,
    StyleChiffre::Poison,
];

/// Entrée d'un invité : son numéro, ses touches et le lacet de sa caméra.
pub fn ecrire_entree(numero: u32, c: &Commandes, sortie: &mut Vec<u8>) {
    let mut w = Ecrivain::nouveau(sortie);
    w.u8(TYPE_ENTREE);
    w.u32(numero);
    w.u8(bits(c));
    w.f32(c.lacet);
}

/// (numéro, commandes) d'une entrée reçue.
pub fn lire_entree(octets: &[u8]) -> Option<(u32, Commandes)> {
    let mut l = super::octets::Lecteur::nouveau(octets);
    if l.u8() != TYPE_ENTREE {
        return None;
    }
    let numero = l.u32();
    let bits = l.u8();
    let lacet = l.f32();
    (!l.erreur).then(|| (numero, commandes(bits & 15, lacet)))
}

/// Ce qui ne se lit pas dans la partie : numéros, rang réseau de chaque joueur.
pub struct Entete<'a> {
    pub numero: u32,
    /// Dernière entrée de l'invité prise en compte (il corrige sa prédiction d'après elle).
    pub derniere_entree: u32,
    /// Par joueur de la partie (0 : l'hôte) : sa place dans le salon.
    pub rangs: &'a [u8],
}

/// Écrit l'instantané destiné au joueur `dest` (≥ 1) dans `sortie`.
pub fn ecrire_instantane_partie(
    p: &Partie,
    dest: usize,
    entete: &Entete,
    chiffres: &[Chiffre],
    sortie: &mut Vec<u8>,
) {
    let mut w = Ecrivain::nouveau(sortie);
    let e = &p.etat;
    let hote = p.etat_joueur(0);
    w.u8(TYPE_INSTANTANE);
    w.u32(entete.numero);
    w.u32(entete.derniere_entree);
    w.f64(e.temps_jeu);
    w.u32(e.ennemis_tues);
    w.u16(hote.niveau.min(u32::from(u16::MAX)) as u16);
    w.f32(hote.xp);
    w.f32(hote.xp_niveau);
    w.u32(e.emeraudes);
    w.u8(match e.issue {
        None => 0,
        Some(Issue::Victoire) => 1,
        Some(Issue::Defaite) => 2,
    });
    w.u8(u8::from(e.sans_fin) | u8::from(p.prolongation) << 1);

    // Équipe
    let n = p.nombre_joueurs();
    w.u8(n as u8);
    for k in 0..n {
        let j = p.etat_joueur(k);
        let (c, choix, parti) = if k == 0 {
            (&p.joueur.commandes, p.equipe.choix_hote, false)
        } else {
            let m = &p.equipe.membres[k - 1];
            (&m.joueur.commandes, m.en_choix, m.parti)
        };
        w.u8(entete.rangs.get(k).copied().unwrap_or(k as u8));
        w.f32(j.x);
        w.f32(j.z);
        w.angle(j.angle);
        w.f32(j.pv);
        w.f32(j.pv_max);
        w.f32(j.bouclier);
        w.u8((u8::from(j.mort) * JOUEUR_MORT)
            | (u8::from(j.protection > 0.0) * JOUEUR_PROTEGE)
            | (u8::from(choix) * JOUEUR_EN_CHOIX)
            | (u8::from(parti) * JOUEUR_PARTI));
        w.f32(j.temps_mort);
        w.f32(j.invulnerabilite);
        w.u8(bits(c));
        w.f32(c.lacet);
        let reanimation = p.equipe.reanimations.get(k).copied().unwrap_or(0.0);
        w.part(reanimation / bs_regles::equipe::regles::DUREE_REANIMATION);
        w.u8(j.vies_utilisees.min(255) as u8);
    }

    let moi = p.etat_joueur(dest);
    let (cx, cz) = (moi.x, moi.z);
    let r2 = RAYON_ENVOI * RAYON_ENVOI;
    let proche = |x: f32, z: f32| (x - cx) * (x - cx) + (z - cz) * (z - cz) <= r2;

    // Ennemis (les boss, partout : leur barre de vie)
    let compte = w.reserver16();
    let mut nombre = 0u16;
    for en in p.ennemis.actifs() {
        if !proche(en.x, en.z) && !en.boss() {
            continue;
        }
        w.u32(en.id);
        w.u16(en.index_type as u16);
        w.coordonnee(en.x);
        w.coordonnee(en.z);
        w.angle(en.angle);
        w.f32(en.pv);
        w.f32(en.pv_max);
        w.u8((u8::from(!en.vivant()) * ENNEMI_MORT)
            | (u8::from(en.blessure > 0.0) * ENNEMI_BLESSE)
            | (u8::from(en.ralenti > 0.0) * ENNEMI_RALENTI));
        w.part(en.mort.max(0.0) / bs_regles::ennemis::DUREE_MORT);
        w.u8((en.vitesse_sol * 16.0).clamp(0.0, 255.0) as u8);
        w.part(en.meche.min(1.0));
        nombre += 1;
    }
    w.ecrire16(compte, nombre);

    // Butin
    let r = &p.ramassables;
    let compte = w.reserver16();
    let mut nombre = 0u16;
    for o in r.orbes.iter().filter(|o| proche(o.x, o.z)) {
        w.coordonnee(o.x);
        w.coordonnee(o.z);
        w.f32(o.valeur);
        nombre += 1;
    }
    w.ecrire16(compte, nombre);
    let compte = w.reserver16();
    let mut nombre = 0u16;
    for o in r.objets.iter().filter(|o| proche(o.x, o.z)) {
        w.u16(o.index_type as u16);
        w.coordonnee(o.x);
        w.coordonnee(o.z);
        w.f32(o.valeur);
        nombre += 1;
    }
    w.ecrire16(compte, nombre);
    let compte = w.reserver16();
    let mut nombre = 0u16;
    for c in r.coffres.pool.iter().filter(|c| proche(c.x, c.z)) {
        w.u16(c.index_type as u16);
        w.coordonnee(c.x);
        w.coordonnee(c.z);
        w.angle(c.angle);
        w.f32(c.ouvert.unwrap_or(-1.0));
        nombre += 1;
    }
    w.ecrire16(compte, nombre);

    // Projectiles ennemis
    let compte = w.reserver16();
    let mut nombre = 0u16;
    for t in p
        .projectiles_ennemis
        .pool
        .iter()
        .filter(|t| proche(t.x, t.z))
    {
        w.u16(t.index_type as u16);
        w.coordonnee(t.x);
        w.coordonnee(t.z);
        w.f32(t.vx);
        w.f32(t.vz);
        w.f32(t.vie);
        w.u8(u8::from(t.renvoye));
        nombre += 1;
    }
    w.ecrire16(compte, nombre);

    // Chiffres de dégâts
    let compte = w.reserver16();
    let mut nombre = 0u16;
    for c in chiffres
        .iter()
        .filter(|c| proche(c.x, c.z))
        .take(CHIFFRES_MAX)
    {
        w.coordonnee(c.x);
        w.f32(c.y);
        w.coordonnee(c.z);
        w.f32(c.valeur);
        w.u8(STYLES.iter().position(|s| *s == c.style).unwrap_or(0) as u8);
        nombre += 1;
    }
    w.ecrire16(compte, nombre);
}
