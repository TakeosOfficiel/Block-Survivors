//! Génération déterministe d'une arène à partir de sa définition : sol et taches, murs,
//! structures placées ou dispersées, puis carte de collision. Même arène = même monde à
//! chaque partie (graine tirée de son identifiant), et le même qu'en TypeScript : calculs
//! en double précision et même suite de hasard (mulberry32).

use bs_commun::{Mulberry, choisir_pondere, graine, hachage3};
use bs_donnees::Donnees;
use bs_donnees::arenes::{Arene, ElementDecor, Rectangle, Structure, TacheSol};
use bs_donnees::blocs::FormeBloc;

use crate::carte::{BLOQUE_MARCHE, BLOQUE_TIR, CarteMonde};

/// Couches de décor possibles au-dessus du sol.
const HAUTEUR_MAX: usize = 8;

/// Monde généré : blocs à afficher et carte de collision.
#[derive(Debug, Clone)]
pub struct MondeGenere {
    /// Centres des blocs à afficher, par bloc (dans l'ordre de la table des blocs).
    pub blocs: Vec<(String, Vec<[f32; 3]>)>,
    pub carte: CarteMonde,
}

/// Génère l'arène `arene` (définition validée : blocs et structures existent).
pub fn generer_monde(donnees: &Donnees, arene: &Arene) -> MondeGenere {
    Generateur::nouveau(donnees, arene).generer()
}

struct Generateur<'a> {
    donnees: &'a Donnees,
    def: &'a Arene,
    largeur: usize,
    profondeur: usize,
    demi_x: f64,
    demi_z: f64,
    hasard: Mulberry,
    /// Bloc de sol de chaque case (index dans la table des blocs).
    sol: Vec<usize>,
    /// Blocs de décor : index (couche × profondeur + iz) × largeur + ix.
    volume: Vec<Option<usize>>,
    /// Cases dont la colonne contient déjà du décor.
    occupe: Vec<bool>,
}

fn hypot(x: f64, z: f64) -> f64 {
    libm::hypot(x, z)
}

impl<'a> Generateur<'a> {
    fn nouveau(donnees: &'a Donnees, def: &'a Arene) -> Self {
        let largeur = def.largeur as usize;
        let profondeur = def.profondeur as usize;
        let cases = largeur * profondeur;
        Self {
            donnees,
            def,
            largeur,
            profondeur,
            demi_x: largeur as f64 / 2.0,
            demi_z: profondeur as f64 / 2.0,
            hasard: Mulberry::nouveau(graine(&def.id)),
            sol: vec![0; cases],
            volume: vec![None; cases * HAUTEUR_MAX],
            occupe: vec![false; cases],
        }
    }

    fn bloc(&self, id: &str) -> usize {
        self.donnees.blocs.get_index_of(id).unwrap_or(0)
    }

    fn generer(mut self) -> MondeGenere {
        self.poser_sol();
        for tache in &self.def.taches {
            match tache {
                TacheSol::Rectangle { bloc, rect } => self.remplir_rectangle(self.bloc(bloc), rect),
                TacheSol::Taches {
                    bloc,
                    nombre,
                    rayon,
                    zone,
                } => self.poser_taches(self.bloc(bloc), *nombre as usize, *rayon, zone.as_ref()),
            }
        }
        for element in &self.def.decor {
            let Some(structure) = self.donnees.structures.get(element.structure()) else {
                continue;
            };
            match element {
                ElementDecor::Structure { x, z, .. } => {
                    let ix0 = (f64::from(*x) + self.demi_x).floor() as i64;
                    let iz0 = (f64::from(*z) + self.demi_z).floor() as i64;
                    self.poser_structure(structure, ix0, iz0);
                }
                ElementDecor::Dispersion {
                    nombre,
                    espacement,
                    zone,
                    ..
                } => self.disperser(
                    structure,
                    *nombre as usize,
                    *espacement as i64,
                    zone.as_ref(),
                ),
            }
        }
        MondeGenere {
            blocs: self.lister_blocs(),
            carte: self.construire_carte(),
        }
    }

    // --- Sol ---------------------------------------------------------------

    fn poser_sol(&mut self) {
        for iz in 0..self.profondeur {
            for ix in 0..self.largeur {
                let i = choisir_pondere(
                    &self.def.sol,
                    |c| c.poids,
                    hachage3(ix as i32, 0, iz as i32),
                );
                self.sol[iz * self.largeur + ix] = self.bloc(&self.def.sol[i].bloc);
            }
        }
    }

    fn remplir_rectangle(&mut self, bloc: usize, rect: &Rectangle) {
        let [x0, z0, x1, z1] = rect.map(f64::from);
        for iz in 0..self.profondeur {
            for ix in 0..self.largeur {
                let x = ix as f64 - self.demi_x + 0.5;
                let z = iz as f64 - self.demi_z + 0.5;
                if x >= x0 && x < x1 && z >= z0 && z < z1 {
                    self.sol[iz * self.largeur + ix] = bloc;
                }
            }
        }
    }

    /// Taches rondes au hasard dans la zone, jamais sur la zone de départ.
    fn poser_taches(
        &mut self,
        bloc: usize,
        nombre: usize,
        [r_min, r_max]: [f32; 2],
        zone: Option<&Rectangle>,
    ) {
        let (r_min, r_max) = (f64::from(r_min), f64::from(r_max));
        for _ in 0..nombre {
            let rayon = r_min + self.hasard.tirer() * (r_max - r_min);
            for _ in 0..30 {
                let (cx, cz) = self.point_dans(zone);
                if hypot(cx, cz) < f64::from(self.def.zone_depart) + rayon + 1.0 {
                    continue;
                }
                for iz in 0..self.profondeur {
                    for ix in 0..self.largeur {
                        let dx = ix as f64 - self.demi_x + 0.5 - cx;
                        let dz = iz as f64 - self.demi_z + 0.5 - cz;
                        if hypot(dx, dz) <= rayon {
                            self.sol[iz * self.largeur + ix] = bloc;
                        }
                    }
                }
                break;
            }
        }
    }

    // --- Décor -------------------------------------------------------------

    /// Pose une structure dont le coin nord-ouest est la case (ix0, iz0).
    fn poser_structure(&mut self, structure: &Structure, ix0: i64, iz0: i64) {
        for (couche, lignes) in structure.couches.iter().enumerate().take(HAUTEUR_MAX) {
            for (r, ligne) in lignes.iter().enumerate() {
                for (c, caractere) in ligne.chars().enumerate() {
                    let Some(id) = structure.legende.get(&caractere) else {
                        continue;
                    };
                    let ix = ix0 + c as i64;
                    let iz = iz0 + r as i64;
                    if ix < 0 || iz < 0 || ix >= self.largeur as i64 || iz >= self.profondeur as i64
                    {
                        continue;
                    }
                    let (ix, iz) = (ix as usize, iz as usize);
                    self.volume[(couche * self.profondeur + iz) * self.largeur + ix] =
                        Some(self.bloc(id));
                    self.occupe[iz * self.largeur + ix] = true;
                }
            }
        }
    }

    /// Disperse des structures au hasard là où il y a de la place.
    fn disperser(
        &mut self,
        structure: &Structure,
        nombre: usize,
        espacement: i64,
        zone: Option<&Rectangle>,
    ) {
        let largeur = structure
            .couches
            .iter()
            .flatten()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as i64;
        let profondeur = structure.couches.iter().map(Vec::len).max().unwrap_or(0) as i64;
        let mut poses = 0;
        let mut essai = 0;
        while essai < nombre * 40 && poses < nombre {
            essai += 1;
            let (x, z) = self.point_dans(zone);
            let ix0 = (x + self.demi_x - largeur as f64 / 2.0).floor() as i64;
            let iz0 = (z + self.demi_z - profondeur as f64 / 2.0).floor() as i64;
            if !self.est_libre(
                ix0 - espacement,
                iz0 - espacement,
                largeur + 2 * espacement,
                profondeur + 2 * espacement,
            ) {
                continue;
            }
            self.poser_structure(structure, ix0, iz0);
            poses += 1;
        }
    }

    /// Rectangle de cases entièrement dans l'arène, sans décor, sans liquide, hors zone de départ.
    fn est_libre(&self, ix0: i64, iz0: i64, largeur: i64, profondeur: i64) -> bool {
        for iz in iz0..iz0 + profondeur {
            for ix in ix0..ix0 + largeur {
                if ix < 0 || iz < 0 || ix >= self.largeur as i64 || iz >= self.profondeur as i64 {
                    return false;
                }
                let c = iz as usize * self.largeur + ix as usize;
                let liquide = self.donnees.blocs[self.sol[c]].forme == Some(FormeBloc::Liquide);
                if self.occupe[c] || liquide {
                    return false;
                }
                if hypot(ix as f64 - self.demi_x + 0.5, iz as f64 - self.demi_z + 0.5)
                    < f64::from(self.def.zone_depart)
                {
                    return false;
                }
            }
        }
        true
    }

    /// Point au hasard dans la zone (toute l'arène par défaut, à 1 bloc des murs).
    fn point_dans(&mut self, zone: Option<&Rectangle>) -> (f64, f64) {
        let mx = self.demi_x - 1.0;
        let mz = self.demi_z - 1.0;
        let [x0, z0, x1, z1] = zone.map_or([-mx, -mz, mx, mz], |z| z.map(f64::from));
        let x = (x0 + self.hasard.tirer() * (x1 - x0)).clamp(-mx, mx);
        let z = (z0 + self.hasard.tirer() * (z1 - z0)).clamp(-mz, mz);
        (x, z)
    }

    // --- Résultats ---------------------------------------------------------

    fn lister_blocs(&self) -> Vec<(String, Vec<[f32; 3]>)> {
        let mut par_bloc: Vec<Vec<[f32; 3]>> = vec![Vec::new(); self.donnees.blocs.len()];
        let (dx, dz) = (self.demi_x as f32, self.demi_z as f32);
        // Sol : face supérieure à y = 0
        for iz in 0..self.profondeur {
            for ix in 0..self.largeur {
                par_bloc[self.sol[iz * self.largeur + ix]].push([
                    ix as f32 - dx + 0.5,
                    -0.5,
                    iz as f32 - dz + 0.5,
                ]);
            }
        }
        // Décor
        let cases = self.largeur * self.profondeur;
        for (cle, bloc) in self.volume.iter().enumerate() {
            let Some(bloc) = bloc else { continue };
            let ix = cle % self.largeur;
            let iz = (cle / self.largeur) % self.profondeur;
            let couche = cle / cases;
            par_bloc[*bloc].push([
                ix as f32 - dx + 0.5,
                couche as f32 + 0.5,
                iz as f32 - dz + 0.5,
            ]);
        }
        // Murs : anneau de blocs juste autour de la zone jouable (coins compris)
        let mut mur = |x: f32, y: f32, z: f32| {
            let tirage = hachage3(x.floor() as i32, y.floor() as i32, z.floor() as i32);
            let i = choisir_pondere(&self.def.murs, |c| c.poids, tirage);
            par_bloc[self.bloc(&self.def.murs[i].bloc)].push([x, y, z]);
        };
        for h in 0..self.def.hauteur_mur as usize {
            let y = h as f32 + 0.5;
            for i in -1..=self.largeur as i64 {
                let x = -dx + i as f32 + 0.5;
                mur(x, y, -dz - 0.5);
                mur(x, y, dz + 0.5);
            }
            for i in 0..self.profondeur {
                let z = -dz + i as f32 + 0.5;
                mur(-dx - 0.5, y, z);
                mur(dx + 0.5, y, z);
            }
        }
        self.donnees
            .blocs
            .keys()
            .zip(par_bloc)
            .filter(|(_, p)| !p.is_empty())
            .map(|(id, p)| (id.clone(), p))
            .collect()
    }

    /// Couches 0 et 1 (hauteur du corps) : blocs pleins et liquides bloquent la marche ;
    /// les blocs pleins arrêtent aussi les projectiles. Les sols et les croix (toiles)
    /// donnent leurs effets.
    fn construire_carte(&self) -> CarteMonde {
        let mut carte = CarteMonde::nouvelle(self.largeur, self.profondeur);
        let cases = self.largeur * self.profondeur;
        for c in 0..cases {
            let sol = &self.donnees.blocs[self.sol[c]];
            if sol.forme == Some(FormeBloc::Liquide) {
                carte.drapeaux[c] |= BLOQUE_MARCHE;
            }
            carte.ralentissement[c] = sol.ralentit.unwrap_or(1.0);
            carte.ralentissement_hors_toiles[c] = carte.ralentissement[c];
            carte.degats[c] = sol.degats.unwrap_or(0.0);
            for couche in 0..=1 {
                let Some(id) = self.volume[couche * cases + c] else {
                    continue;
                };
                let bloc = &self.donnees.blocs[id];
                if bloc.bloque_le_passage() {
                    carte.drapeaux[c] |= BLOQUE_MARCHE;
                    if bloc.forme != Some(FormeBloc::Liquide) {
                        carte.drapeaux[c] |= BLOQUE_TIR;
                    }
                } else if let Some(r) = bloc.ralentit {
                    carte.ralentissement[c] = carte.ralentissement[c].min(r);
                    if !bloc.toile {
                        carte.ralentissement_hors_toiles[c] =
                            carte.ralentissement_hors_toiles[c].min(r);
                    }
                }
            }
        }
        carte
    }
}
