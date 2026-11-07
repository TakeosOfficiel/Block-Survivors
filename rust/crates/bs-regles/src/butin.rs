//! Butin d'armure des boss (port de recompenserBoss dans jeu.ts et de
//! collection.butinBoss) : selon le rang du boss dans la partie, une pièce d'armure du
//! monde, peut-être une deuxième, jamais une pièce déjà possédée. La Collection (dans la
//! sauvegarde) donne les pièces possédées au départ (`posseder`) et garde les pièces
//! gagnées (`gagnees`), qui n'agissent que sur les parties suivantes ; les objets
//! d'équipement trouvés y sont aussi inscrits (`objets_trouves`).

use bs_donnees::equipement::Piece;

use crate::evenements::Evenement;
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
    /// Pièces possédées : celles de la Collection pour ce monde, puis celles gagnées.
    pub pieces: Vec<Piece>,
    /// Pièces gagnées pendant la partie, dans l'ordre.
    pub gagnees: Vec<Piece>,
    /// Objets d'équipement pris pendant la partie (index du catalogue, sans doublon).
    pub objets_trouves: Vec<usize>,
}

impl Partie {
    /// Pièces d'armure de ce monde déjà dans la Collection : jamais tirées à nouveau.
    pub fn posseder(&mut self, pieces: &[Piece]) {
        for p in pieces {
            if !self.butin.pieces.contains(p) {
                self.butin.pieces.push(*p);
            }
        }
    }

    /// Objet d'équipement pris : inscrit une fois par partie (la Collection le découvre).
    pub(crate) fn trouver_objet(&mut self, objet: usize) {
        if !self.butin.objets_trouves.contains(&objet) {
            self.butin.objets_trouves.push(objet);
            self.ev.pousser(Evenement::Decouverte(objet));
        }
    }

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
            let mut gagnees = [None; 2];
            for (k, gagnee) in gagnees.iter_mut().enumerate() {
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
                self.butin.gagnees.push(piece);
                *gagnee = Some(piece);
            }
            if gagnees[0].is_some() {
                let monde = self.etat.arene.monde;
                self.ev.pousser(Evenement::ButinBoss {
                    monde,
                    pieces: gagnees,
                });
                self.ev.son("totem");
                let j = &self.etat.joueur;
                self.ev.gerbe("totem", j.x, 1.2, j.z);
            }
        }
    }
}
