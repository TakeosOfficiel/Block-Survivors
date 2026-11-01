//! Retours de la simulation vers l'affichage et le son : gerbes de particules, sons,
//! chiffres de dégâts, secousses. La simulation les empile dans un tampon réutilisé ;
//! le jeu les consomme à chaque image (le robot les ignore). Rien ici ne change l'état.

/// Style d'un chiffre de dégâts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleChiffre {
    Normal,
    Critique,
    Feu,
    Poison,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Evenement {
    /// Gerbe de particules (identifiant de donnees/particules.ron) en (x, y, z).
    Gerbe {
        gerbe: &'static str,
        x: f32,
        y: f32,
        z: f32,
    },
    /// Son (identifiant de donnees/sons.ron), hauteur imposée ou tirée au hasard.
    Son {
        son: &'static str,
        hauteur: Option<f32>,
    },
    Degats {
        x: f32,
        y: f32,
        z: f32,
        valeur: f32,
        style: StyleChiffre,
    },
    /// Intensité ajoutée à la secousse de la caméra (0 à 1).
    Secousse(f32),
    /// Geste d'attaque du modèle : le joueur (`None`) ou l'allié de cet identifiant.
    Geste(Option<u32>),
}

/// Tampon des événements d'un pas (capacité gardée d'un pas à l'autre).
#[derive(Debug, Default)]
pub struct Evenements {
    pub liste: Vec<Evenement>,
    /// Faux : rien n'est retenu (robot, serveur) ; aucun coût.
    pub actifs: bool,
}

impl Evenements {
    pub fn nouveaux(actifs: bool) -> Self {
        Self {
            liste: Vec::with_capacity(if actifs { 4096 } else { 0 }),
            actifs,
        }
    }

    #[inline]
    pub fn pousser(&mut self, e: Evenement) {
        if self.actifs && self.liste.len() < self.liste.capacity() {
            self.liste.push(e);
        }
    }

    #[inline]
    pub fn gerbe(&mut self, gerbe: &'static str, x: f32, y: f32, z: f32) {
        self.pousser(Evenement::Gerbe { gerbe, x, y, z });
    }

    #[inline]
    pub fn son(&mut self, son: &'static str) {
        self.pousser(Evenement::Son { son, hauteur: None });
    }

    /// Son joué à une hauteur (vitesse de lecture) donnée.
    pub fn son_hauteur(&mut self, son: &'static str, hauteur: f32) {
        self.pousser(Evenement::Son {
            son,
            hauteur: Some(hauteur),
        });
    }

    #[inline]
    pub fn secousse(&mut self, force: f32) {
        self.pousser(Evenement::Secousse(force));
    }

    /// Geste d'attaque du joueur (`None`) ou d'un allié.
    pub fn geste(&mut self, allie: Option<u32>) {
        self.pousser(Evenement::Geste(allie));
    }

    pub fn vider(&mut self) {
        self.liste.clear();
    }
}
