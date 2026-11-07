//! Grille spatiale de l'arène : retrouve vite les ennemis proches d'un point (armes,
//! écartement des zombies). Reconstruite à chaque pas par un tri par comptage, dans des
//! tableaux préalloués.

/// Cases carrées couvrant l'arène, centrée sur l'origine.
pub struct GrilleSpatiale {
    taille_case: f32,
    colonnes: usize,
    lignes: usize,
    demi_largeur: f32,
    demi_profondeur: f32,
    /// Début de chaque case dans `elements` (une entrée de plus que de cases).
    debuts: Vec<u32>,
    elements: Vec<u32>,
    /// Case de chaque élément inséré (u32::MAX : absent).
    cases: Vec<u32>,
}

impl GrilleSpatiale {
    pub fn nouvelle(largeur: f32, profondeur: f32, taille_case: f32, capacite: usize) -> Self {
        let colonnes = (largeur / taille_case).ceil() as usize;
        let lignes = (profondeur / taille_case).ceil() as usize;
        Self {
            taille_case,
            colonnes,
            lignes,
            demi_largeur: largeur / 2.0,
            demi_profondeur: profondeur / 2.0,
            debuts: vec![0; colonnes * lignes + 1],
            elements: vec![0; capacite],
            cases: vec![u32::MAX; capacite],
        }
    }

    fn colonne(&self, x: f32) -> usize {
        (((x + self.demi_largeur) / self.taille_case) as isize).clamp(0, self.colonnes as isize - 1)
            as usize
    }

    fn ligne(&self, z: f32) -> usize {
        (((z + self.demi_profondeur) / self.taille_case) as isize)
            .clamp(0, self.lignes as isize - 1) as usize
    }

    /// Range les éléments présents (`present[i]`) d'après leurs positions.
    pub fn reconstruire(&mut self, x: &[f32], z: &[f32], present: &[bool]) {
        self.debuts.fill(0);
        for i in 0..x.len() {
            if present[i] {
                let c = (self.ligne(z[i]) * self.colonnes + self.colonne(x[i])) as u32;
                self.cases[i] = c;
                self.debuts[c as usize + 1] += 1;
            } else {
                self.cases[i] = u32::MAX;
            }
        }
        for c in 1..self.debuts.len() {
            self.debuts[c] += self.debuts[c - 1];
        }
        // Remplissage : `debuts[c]` sert de curseur, puis on le recale
        for i in 0..x.len() {
            let c = self.cases[i];
            if c != u32::MAX {
                let place = &mut self.debuts[c as usize];
                self.elements[*place as usize] = i as u32;
                *place += 1;
            }
        }
        for c in (1..self.debuts.len()).rev() {
            self.debuts[c] = self.debuts[c - 1];
        }
        self.debuts[0] = 0;
    }

    /// Appelle `visiter` pour chaque élément des cases qui touchent le carré autour de (x, z).
    pub fn pour_chaque_proche(&self, x: f32, z: f32, rayon: f32, mut visiter: impl FnMut(usize)) {
        let (c0, c1) = (self.colonne(x - rayon), self.colonne(x + rayon));
        let (l0, l1) = (self.ligne(z - rayon), self.ligne(z + rayon));
        for l in l0..=l1 {
            for c in c0..=c1 {
                let case = l * self.colonnes + c;
                for k in self.debuts[case]..self.debuts[case + 1] {
                    visiter(self.elements[k as usize] as usize);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrouve_les_voisins() {
        let mut g = GrilleSpatiale::nouvelle(20.0, 20.0, 2.0, 3);
        g.reconstruire(&[0.0, 0.5, 8.0], &[0.0, 0.5, 8.0], &[true, true, true]);
        let mut trouves = Vec::new();
        g.pour_chaque_proche(0.0, 0.0, 1.0, |i| trouves.push(i));
        trouves.sort();
        assert_eq!(trouves, vec![0, 1]);
    }
}
