//! Pièces solides des armes, dessinées avec le matériau des entités (et non en
//! lueurs) : blocs texturés (météores, blocs soulevés, éclats de glace) et images
//! dressées (lame géante qui tombe). Un lot par texture et par genre ; une texture
//! animée (bande d'images, comme le magma) ne montre que sa première image.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_donnees::armes::Mecanique;

use crate::effets::Lot;
use crate::effets::geometrie::plan;
use crate::materiau::ParametresBloc;
use crate::materiau_entite::{MateriauEntite, tag_surcouche};
use crate::modeles::rendu::Composants;
use crate::textures::Textures;

/// Genre de pièce : bloc cubique, ou image dressée (côté 1, centrée).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Cube,
    Image,
}

/// Une texture à préparer : chemin, genre, lumineuse (non éclairée).
type Besoin = (&'static str, Piece, bool);

/// Textures des pièces solides de toutes les armes.
fn besoins(d: &'static Donnees) -> impl Iterator<Item = Besoin> {
    d.armes.values().flat_map(|a| {
        let (un, deux): (Option<Besoin>, Option<Besoin>) = match &a.mecanique {
            Mecanique::Meteore { texture, .. } => (Some((texture, Piece::Cube, true)), None),
            Mecanique::Nova { texture, sol, .. } => (
                Some((texture, Piece::Cube, false)),
                Some((sol, Piece::Cube, false)),
            ),
            Mecanique::Chute { image, .. } => (Some((image, Piece::Image, false)), None),
            Mecanique::Orbes { texture, .. } => (Some((texture, Piece::Cube, true)), None),
            Mecanique::Piege { image, .. } => (Some((image, Piece::Image, false)), None),
            Mecanique::Pluie { .. } => (Some((&a.icone, Piece::Image, false)), None),
            _ => (None, None),
        };
        un.into_iter().chain(deux)
    })
}

/// Chemins des textures à charger.
pub fn textures(d: &'static Donnees) -> impl Iterator<Item = &'static str> {
    besoins(d).map(|(c, ..)| c)
}

/// Cube unité dont chaque face montre la première image carrée de la texture.
fn cube(proportion: f32) -> Mesh {
    let mut m = Mesh::from(Cuboid::from_length(1.0));
    let n = m.count_vertices();
    if let Some(VertexAttributeValues::Float32x2(uv)) = m.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        for c in uv.iter_mut() {
            c[1] *= proportion;
        }
    }
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; n]);
    m
}

pub struct Solides {
    lots: Vec<(&'static str, Piece, Lot)>,
    /// Relief de l'image : chaque pièce monte au niveau du sol sous elle.
    relief: crate::relief::Relief,
}

/// Ce qu'il faut pour créer les lots.
pub struct Fabrique<'a> {
    pub ambiance: ParametresBloc,
    pub serveur: &'a AssetServer,
    pub textures: &'a Textures,
    pub images: &'a Assets<Image>,
    pub maillages: &'a mut Assets<Mesh>,
    pub materiaux: &'a mut Assets<MateriauEntite>,
}

impl Fabrique<'_> {
    fn lot(&mut self, chemin: &str, piece: Piece, lumineux: bool) -> Lot {
        let mut parametres = self.ambiance;
        parametres.brouillard.z = 0.5;
        parametres.brouillard.w = if lumineux { 1.0 } else { 0.0 };
        let texture = self.textures.obtenir(self.serveur, chemin);
        let proportion = texture
            .as_ref()
            .and_then(|t| self.images.get(t))
            .map_or(1.0, |i| i.width() as f32 / i.height().max(1) as f32);
        let maillage = match piece {
            Piece::Cube => cube(proportion),
            Piece::Image => plan(1.0, 1.0, [0.0, 0.0, 1.0, proportion], [1.0; 4]).en_mesh(),
        };
        Lot::nouveau(
            "piece-arme",
            self.maillages.add(maillage),
            self.materiaux.add(MateriauEntite {
                parametres,
                texture,
                lumiere: None,
            }),
        )
    }
}

impl Solides {
    /// Les lots de toutes les textures d'armes, plus les blocs du sol de l'arène.
    pub fn nouveaux(d: &'static Donnees, sol: &'static str, f: &mut Fabrique) -> Self {
        let mut lots: Vec<(&'static str, Piece, Lot)> = Vec::new();
        for (chemin, piece, lumineux) in besoins(d).chain([(sol, Piece::Cube, false)]) {
            if lots.iter().any(|(c, p, _)| *c == chemin && *p == piece) {
                continue;
            }
            lots.push((chemin, piece, f.lot(chemin, piece, lumineux)));
        }
        Self {
            lots,
            relief: Default::default(),
        }
    }

    /// Commence l'image : aucune pièce posée.
    pub fn commencer(&mut self) {
        for (.., l) in &mut self.lots {
            l.commencer();
        }
        self.relief = crate::relief::Relief::actuel();
    }

    /// Pose une pièce de texture `chemin`.
    pub fn poser(
        &mut self,
        commandes: &mut Commands,
        q: &mut Composants,
        chemin: &str,
        piece: Piece,
        mut tf: Transform,
    ) {
        tf.translation.y += self.relief.sol(tf.translation.x, tf.translation.z);
        let Some((.., lot)) = self
            .lots
            .iter_mut()
            .find(|(c, p, _)| *c == chemin && *p == piece)
        else {
            return;
        };
        lot.ajouter(commandes, q, tf, &tag_surcouche([0.0; 4]));
    }

    pub fn terminer(&mut self, q: &mut Composants) {
        for (.., l) in &mut self.lots {
            l.terminer(q);
        }
    }
}
