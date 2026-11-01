//! Modèles Minecraft des entités : compilation (`compilation`), animation
//! (`animation`), géométrie (`geometrie`) et rendu instancié (`rendu`).
//!
//! `Rendus` tient un rendu par apparence : le personnage joué, chaque type d'ennemi
//! puis chaque type d'allié, dans l'ordre des tables de données.

pub mod animation;
pub mod compilation;
pub mod geometrie;
pub mod rendu;

use bevy::prelude::*;
use bs_donnees::Apparence;
use bs_donnees::Donnees;

use crate::arene::couleur_lineaire;
use crate::materiau::ParametresBloc;
use crate::materiau_entite::MateriauEntite;
use crate::textures::Textures;
use compilation::{ModeleCompile, PIXELS_PAR_BLOC};
use geometrie::Maillage;
use rendu::{ObjetTenu, RenduModele};

/// Couleur des parties sans couleur de secours (src/modeles/rendu.ts).
const SECOURS_PAR_DEFAUT: u32 = 0x808080;
/// Seuil de découpe des skins (pixels transparents percés).
const DECOUPE: f32 = 0.5;

/// Toutes les apparences à afficher, dans l'ordre des rendus.
pub fn apparences(
    d: &'static Donnees,
    personnage: usize,
) -> impl Iterator<Item = &'static Apparence> {
    let joueur = &d.personnages.personnages[personnage].apparence;
    std::iter::once(joueur)
        .chain(d.ennemis.values().map(|e| &e.apparence))
        .chain(d.allies.allies.values().map(|a| &a.apparence))
}

/// Textures de toutes les apparences (skins et objets tenus).
pub fn demander_textures(
    d: &'static Donnees,
    personnage: usize,
    serveur: &AssetServer,
    t: &mut Textures,
) {
    for a in apparences(d, personnage) {
        t.demander(serveur, &a.skin);
        if let Some(o) = &a.objet_en_main {
            t.demander(serveur, o);
        }
    }
}

#[derive(Resource)]
pub struct Rendus {
    pub liste: Vec<RenduModele>,
    nombre_ennemis: usize,
}

impl Rendus {
    pub fn joueur(&mut self) -> &mut RenduModele {
        &mut self.liste[0]
    }

    pub fn ennemi(&mut self, type_ennemi: usize) -> &mut RenduModele {
        &mut self.liste[1 + type_ennemi]
    }

    pub fn allie(&mut self, type_allie: usize) -> &mut RenduModele {
        &mut self.liste[1 + self.nombre_ennemis + type_allie]
    }

    /// Modèle compilé (sans emprunt mutable).
    pub fn modele_ennemi(&self, type_ennemi: usize) -> &ModeleCompile {
        &self.liste[1 + type_ennemi].modele
    }

    pub fn modele_allie(&self, type_allie: usize) -> &ModeleCompile {
        &self.liste[1 + self.nombre_ennemis + type_allie].modele
    }

    pub fn modele_joueur(&self) -> &ModeleCompile {
        &self.liste[0].modele
    }

    /// Construit les rendus (textures déjà chargées ou introuvables).
    pub fn construire(
        d: &'static Donnees,
        personnage: usize,
        ambiance: ParametresBloc,
        serveur: &AssetServer,
        textures: &Textures,
        images: &Assets<Image>,
        maillages: &mut Assets<Mesh>,
        materiaux: &mut Assets<MateriauEntite>,
    ) -> Self {
        let mut parametres = ambiance;
        parametres.brouillard.z = DECOUPE;
        parametres.brouillard.w = 0.0;
        let liste = apparences(d, personnage)
            .map(|a| {
                let def = &d.modeles[a.modele.as_str()];
                let modele = ModeleCompile::nouveau(def);
                let skin = textures.obtenir(serveur, &a.skin);
                let parties = def
                    .parties
                    .iter()
                    .map(|(nom, cube)| {
                        let couleur = if skin.is_some() {
                            [1.0; 4]
                        } else {
                            couleur_lineaire(
                                a.couleurs_secours
                                    .get(nom)
                                    .copied()
                                    .or(a.couleur_secours_par_defaut)
                                    .unwrap_or(SECOURS_PAR_DEFAUT),
                            )
                        };
                        let mut m = Maillage::default();
                        geometrie::cube(&mut m, cube, def.taille_texture, couleur);
                        maillages.add(m.en_mesh())
                    })
                    .collect();
                let materiau = materiaux.add(MateriauEntite {
                    parametres,
                    texture: skin,
                });
                let objet = objet_tenu(
                    a, &modele, serveur, textures, images, maillages, materiaux, parametres,
                );
                RenduModele::nouveau(&a.modele, modele, parties, materiau, objet)
            })
            .collect();
        Self {
            liste,
            nombre_ennemis: d.ennemis.len(),
        }
    }
}

/// Objet tenu en main, extrudé depuis les pixels de sa texture (aucun si la texture
/// manque ou si le modèle n'a pas de main).
fn objet_tenu(
    a: &Apparence,
    modele: &ModeleCompile,
    serveur: &AssetServer,
    textures: &Textures,
    images: &Assets<Image>,
    maillages: &mut Assets<Mesh>,
    materiaux: &mut Assets<MateriauEntite>,
    parametres: ParametresBloc,
) -> Option<ObjetTenu> {
    let main = modele.def.main.as_ref()?;
    let local = ObjetTenu::local(modele)?;
    let poignee = textures.obtenir(serveur, a.objet_en_main.as_ref()?)?;
    let image = images.get(&poignee)?;
    let (largeur, hauteur) = (image.width() as usize, image.height() as usize);
    let pixels = image.data.as_ref()?;
    if pixels.len() != largeur * hauteur * 4 {
        warn!(
            "objet tenu {:?} : format de pixels inattendu",
            a.objet_en_main
        );
        return None;
    }
    let pixel = main.taille / largeur as f32 / PIXELS_PAR_BLOC;
    let m = geometrie::objet(pixels, largeur, hauteur, pixel);
    Some(ObjetTenu {
        maillage: maillages.add(m.en_mesh()),
        materiau: materiaux.add(MateriauEntite {
            parametres,
            texture: Some(poignee),
        }),
        local,
    })
}

#[cfg(test)]
mod tests;
