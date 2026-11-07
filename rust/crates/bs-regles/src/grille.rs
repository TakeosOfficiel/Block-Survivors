//! Grille spatiale uniforme sur le plan XZ (listes chaînées dans des tableaux) pour
//! trouver les voisins sans tout comparer à tout. Aucune allocation après la création.
//! Même ordre de parcours que src/utils/grilleSpatiale.ts (les armes touchent dans cet ordre).

#[derive(Debug, Clone)]
pub struct GrilleSpatiale {
    origine: f32,
    cellule: f32,
    colonnes: usize,
    tetes: Vec<i32>,
    suivants: Vec<i32>,
}

impl GrilleSpatiale {
    /// Couvre le carré [−demi_cote, demi_cote]² ; les points hors du carré sont rangés au bord.
    pub fn nouvelle(demi_cote: f32, cellule: f32, capacite: usize) -> Self {
        let colonnes = (demi_cote * 2.0 / cellule).ceil() as usize + 1;
        Self {
            origine: -demi_cote,
            cellule,
            colonnes,
            tetes: vec![-1; colonnes * colonnes],
            suivants: vec![-1; capacite],
        }
    }

    pub fn vider(&mut self) {
        self.tetes.fill(-1);
    }

    pub fn inserer(&mut self, index: usize, x: f32, z: f32) {
        let c = self.colonne(z) * self.colonnes + self.colonne(x);
        self.suivants[index] = self.tetes[c];
        self.tetes[c] = index as i32;
    }

    /// Écrit dans `sortie` les index rangés dans les cellules touchées par le cercle
    /// (candidats à filtrer par distance) ; renvoie leur nombre.
    pub fn requeter(&self, x: f32, z: f32, rayon: f32, sortie: &mut [u32]) -> usize {
        let (c0, c1) = (self.colonne(x - rayon), self.colonne(x + rayon));
        let (l0, l1) = (self.colonne(z - rayon), self.colonne(z + rayon));
        let mut n = 0;
        for l in l0..=l1 {
            for c in c0..=c1 {
                let mut i = self.tetes[l * self.colonnes + c];
                while i != -1 {
                    if n >= sortie.len() {
                        return n;
                    }
                    sortie[n] = i as u32;
                    n += 1;
                    i = self.suivants[i as usize];
                }
            }
        }
        n
    }

    fn colonne(&self, valeur: f32) -> usize {
        let c = libm::floorf((valeur - self.origine) / self.cellule);
        if c < 0.0 {
            0
        } else if c as usize >= self.colonnes {
            self.colonnes - 1
        } else {
            c as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrouve_les_voisins_dans_l_ordre_d_insertion_inverse() {
        let mut g = GrilleSpatiale::nouvelle(10.0, 2.0, 3);
        g.inserer(0, 0.1, 0.1);
        g.inserer(1, 0.5, 0.5);
        g.inserer(2, 8.0, 8.0);
        let mut sortie = [0; 8];
        let n = g.requeter(0.0, 0.0, 0.5, &mut sortie);
        assert_eq!(&sortie[..n], &[1, 0]);
    }
}
