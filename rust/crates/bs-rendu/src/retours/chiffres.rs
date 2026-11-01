//! Chiffres des dégâts (src/effets/chiffresDegats.ts) : blancs pour un coup normal,
//! dorés et plus gros (avec un « pop ») pour un critique, petits et orange ou verts
//! pour une brûlure ou un poison. Police pixel du jeu, un carré par chiffre face à la
//! caméra, par-dessus le décor ; anneau de taille fixe (le plus ancien cède sa place).

use bevy::prelude::*;
use bs_commun::Alea;
use bs_regles::evenements::StyleChiffre;

use super::maillage::MaillageDynamique;
use crate::arene::couleur_lineaire;

/// Réglages de config.ts `chiffresDegats`.
const CAPACITE: usize = 160;
const CHIFFRES_MAX: usize = 6;
const DERIVE: f32 = 0.8;
const ECART: f32 = 0.9;
const DUREE_POP: f32 = 0.15;
const FONDU: f32 = 0.35;

struct Style {
    couleur: u32,
    taille: f32,
    montee: f32,
    pop: f32,
    duree: f32,
}

const fn style(s: StyleChiffre) -> Style {
    match s {
        StyleChiffre::Normal => Style {
            couleur: 0xffffff,
            taille: 0.38,
            montee: 1.1,
            pop: 0.3,
            duree: 0.8,
        },
        StyleChiffre::Critique => Style {
            couleur: 0xffc22a,
            taille: 0.62,
            montee: 1.4,
            pop: 0.8,
            duree: 1.1,
        },
        StyleChiffre::Feu => Style {
            couleur: 0xff8a2a,
            taille: 0.3,
            montee: 0.8,
            pop: 0.0,
            duree: 0.7,
        },
        StyleChiffre::Poison => Style {
            couleur: 0x8fd45a,
            taille: 0.3,
            montee: 0.8,
            pop: 0.0,
            duree: 0.7,
        },
    }
}

/// Atlas : une case de 32 × 40 px par chiffre, 4 px par pixel de police.
const CASE_L: usize = 32;
const CASE_H: usize = 40;
/// Avance d'un chiffre (6 px de police) en largeur de case.
const AVANCE: f32 = 24.0 / CASE_L as f32;

/// Chiffres 0 à 9 de la police du jeu (5 × 7, bit de poids fort à gauche).
const GLYPHES: [[u8; 7]; 10] = [
    [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
    [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x1f],
    [0x0e, 0x11, 0x01, 0x06, 0x08, 0x11, 0x1f],
    [0x0e, 0x11, 0x01, 0x06, 0x01, 0x11, 0x0e],
    [0x03, 0x05, 0x09, 0x11, 0x1f, 0x01, 0x01],
    [0x1f, 0x10, 0x1e, 0x01, 0x01, 0x11, 0x0e],
    [0x06, 0x08, 0x10, 0x1e, 0x11, 0x11, 0x0e],
    [0x1f, 0x11, 0x01, 0x02, 0x04, 0x04, 0x04],
    [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
    [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x02, 0x0c],
];

/// Atlas RGBA (320 × 40) : chiffres blancs avec l'ombre portée du texte du jeu (quart
/// de la couleur, un pixel de police plus bas à droite).
pub fn atlas() -> (Vec<u8>, u32, u32) {
    let largeur = CASE_L * 10;
    let mut px = vec![0u8; largeur * CASE_H * 4];
    for (chiffre, glyphe) in GLYPHES.iter().enumerate() {
        for (decalage, gris) in [(4usize, 0x3fu8), (0, 0xff)] {
            for (ligne, bits) in glyphe.iter().enumerate() {
                for colonne in 0..5 {
                    if bits >> (4 - colonne) & 1 == 0 {
                        continue;
                    }
                    let x0 = chiffre * CASE_L + 2 + colonne * 4 + decalage;
                    let y0 = ligne * 4 + decalage;
                    for y in y0..y0 + 4 {
                        for x in x0..x0 + 4 {
                            let i = (y * largeur + x) * 4;
                            px[i..i + 4].copy_from_slice(&[gris, gris, gris, 255]);
                        }
                    }
                }
            }
        }
    }
    (px, largeur as u32, CASE_H as u32)
}

#[derive(Clone, Copy)]
struct Chiffre {
    actif: bool,
    position: Vec3,
    /// Dérive le long de l'écran (blocs/s).
    derive: f32,
    valeur: u32,
    style: StyleChiffre,
    age: f32,
}

pub struct Chiffres {
    anneau: Vec<Chiffre>,
    prochain: usize,
    alea: Alea,
    pub maillage: MaillageDynamique,
}

impl Chiffres {
    pub fn nouveaux(graine: u32, maillage: MaillageDynamique) -> Self {
        let vide = Chiffre {
            actif: false,
            position: Vec3::ZERO,
            derive: 0.0,
            valeur: 0,
            style: StyleChiffre::Normal,
            age: 0.0,
        };
        Self {
            anneau: vec![vide; CAPACITE],
            prochain: 0,
            alea: Alea::nouveau(u64::from(graine), 11),
            maillage,
        }
    }

    pub fn ajouter(&mut self, position: Vec3, valeur: f32, style: StyleChiffre) {
        let a = &mut self.alea;
        let ch = &mut self.anneau[self.prochain];
        self.prochain = (self.prochain + 1) % CAPACITE;
        // Un peu d'écart : les coups simultanés sur une foule ne s'empilent pas
        *ch = Chiffre {
            actif: true,
            position: position
                + Vec3::new(
                    (a.unite() - 0.5) * ECART,
                    a.unite() * ECART * 0.6,
                    (a.unite() - 0.5) * ECART,
                ),
            derive: (a.unite() - 0.5) * DERIVE,
            valeur: valeur.round().max(1.0) as u32,
            style,
            age: 0.0,
        };
    }

    pub fn avancer(&mut self, dt: f32) {
        for ch in self.anneau.iter_mut().filter(|c| c.actif) {
            ch.age += dt;
            if ch.age >= style(ch.style).duree {
                ch.actif = false;
            }
        }
    }

    pub fn dessiner(&mut self, droite: Vec3, haut: Vec3) {
        self.maillage.commencer();
        for ch in self.anneau.iter().filter(|c| c.actif) {
            let s = style(ch.style);
            let t = ch.age / s.duree;
            // Montée qui ralentit, « pop » des critiques au début, fondu à la fin
            let montee = s.montee * (1.0 - (1.0 - t) * (1.0 - t));
            let pop = 1.0 + s.pop * (1.0 - ch.age / DUREE_POP).max(0.0);
            let taille = s.taille * pop;
            let alpha = if t < 1.0 - FONDU {
                1.0
            } else {
                (1.0 - t) / FONDU
            };
            let mut couleur = couleur_lineaire(s.couleur);
            couleur[3] = alpha;
            let n = nombre_chiffres(ch.valeur);
            let base = ch.position + Vec3::Y * montee + droite * (ch.derive * ch.age);
            let (demi_l, demi_h) = (taille / 2.0, taille * CASE_H as f32 / CASE_L as f32 / 2.0);
            let mut reste = ch.valeur;
            for k in (0..n).rev() {
                let chiffre = (reste % 10) as f32;
                reste /= 10;
                let cote = (k as f32 - (n as f32 - 1.0) / 2.0) * AVANCE * taille;
                let uv = [chiffre / 10.0, 0.0, (chiffre + 1.0) / 10.0, 1.0];
                self.maillage.carre(
                    base + droite * cote,
                    droite * demi_l,
                    haut * demi_h,
                    uv,
                    couleur,
                );
            }
        }
    }
}

fn nombre_chiffres(v: u32) -> usize {
    let mut n = 1;
    let mut r = v / 10;
    while r > 0 && n < CHIFFRES_MAX {
        n += 1;
        r /= 10;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombre_de_chiffres() {
        assert_eq!(nombre_chiffres(0), 1);
        assert_eq!(nombre_chiffres(9), 1);
        assert_eq!(nombre_chiffres(10), 2);
        assert_eq!(nombre_chiffres(12345), 5);
        assert_eq!(nombre_chiffres(99_999_999), CHIFFRES_MAX);
    }

    #[test]
    fn atlas_des_chiffres() {
        let (px, l, h) = atlas();
        assert_eq!((l, h), (320, 40));
        assert_eq!(px.len(), 320 * 40 * 4);
        // Barre du bas du « 1 » (ligne 6, colonnes 0 à 4) : blanche
        let i = ((6 * 4 + 1) * 320 + 32 + 2 + 1) * 4;
        assert_eq!(&px[i..i + 4], &[255, 255, 255, 255]);
    }
}
