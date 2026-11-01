//! Butin d'armure des boss (port de recompenserBoss dans jeu.ts et de
//! collection.butinBoss) : selon le rang du boss dans la partie, une pièce d'armure du
//! monde, peut-être une deuxième, jamais une pièce déjà gagnée. La Collection n'agit que
//! sur les parties suivantes ; ici on garde les pièces gagnées et surtout les tirages
//! dans l'aléa, pour que la suite de nombres reste celle du jeu TS.

use bs_donnees::equipement::Piece;

use crate::partie::Partie;

const PIECES: [Piece; 7] = [
    Piece::Casque,
    Piece::Plastron,
    Piece::Jambieres,
    Piece::Bottes,
    Piece::Bouclier,
    Piece::Cape,
    Piece::Talisman,
];

#[derive(Debug, Clone, Default)]
pub struct Butin {
    /// Boss déjà récompensés.
    pub recompenses: u32,
    /// Pièces gagnées pendant la partie (sauvegarde vierge au départ).
    pub pieces: Vec<Piece>,
}

impl Partie {
    /// Récompense un boss vaincu pas encore récompensé (un par pas, juste après les ennemis).
    pub fn recompenser_boss(&mut self) {
        if self.etat.stats.boss_vaincus > self.butin.recompenses {
            let rang = self.butin.recompenses as usize;
            self.butin.recompenses += 1;
            let table = &self.d.armures.butin_boss;
            let Some(butin) = table.get(rang.min(table.len().saturating_sub(1))) else {
                return;
            };
            if self.alea.tirer() >= f64::from(butin.chance) {
                return;
            }
            for k in 0..2 {
                if k == 1 && self.alea.tirer() >= f64::from(butin.chance_seconde) {
                    break;
                }
                let poids = |p: &Piece| butin.poids.get(p).copied().unwrap_or(0.0);
                let manquante = |p: &&Piece| !self.butin.pieces.contains(p) && poids(p) > 0.0;
                let total: f32 = PIECES.iter().filter(manquante).map(poids).sum();
                let mut seuil = self.alea.tirer() * f64::from(total);
                let mut choisie = None;
                for p in PIECES.iter().filter(manquante) {
                    seuil -= f64::from(poids(p));
                    if seuil < 0.0 {
                        choisie = Some(*p);
                        break;
                    }
                }
                let Some(piece) = choisie.or_else(|| PIECES.iter().find(manquante).copied()) else {
                    break;
                };
                self.butin.pieces.push(piece);
                self.ev.son("totem");
            }
        }
    }
}
