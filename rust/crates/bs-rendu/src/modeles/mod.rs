//! Modèles Minecraft des entités : compilation (`compilation`), animation
//! (`animation`), géométrie (`geometrie`) et rendu instancié (`rendu`).
//!
//! `Rendus` tient un rendu par apparence : le personnage joué, chaque type d'ennemi,
//! chaque type d'allié, chaque coffre puis les créatures des talents (ravageur, crocs),
//! dans l'ordre des tables de données, puis les pièces d'armure portées (`armure`).

pub mod animation;
pub mod armure;
pub mod compilation;
pub mod geometrie;
pub mod rendu;

use bevy::prelude::*;
use bs_donnees::Apparence;
use bs_donnees::Donnees;
use bs_donnees::arenes::Monde;

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
        .chain(d.coffres.coffres.values().map(|c| &c.apparence))
        .chain(d.allies.creatures_talents.values())
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
    nombre_allies: usize,
    nombre_coffres: usize,
    /// Pièces d'armure portées : index dans `liste`, et partie du joueur de chaque partie.
    armure: Vec<(usize, Vec<Option<usize>>)>,
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

    pub fn coffre(&mut self, type_coffre: usize) -> &mut RenduModele {
        &mut self.liste[1 + self.nombre_ennemis + self.nombre_allies + type_coffre]
    }

    /// Créature d'un talent (index dans `allies.creatures_talents`).
    pub fn creature(&mut self, k: usize) -> &mut RenduModele {
        let debut = 1 + self.nombre_ennemis + self.nombre_allies + self.nombre_coffres;
        &mut self.liste[debut + k]
    }

    /// Modèle compilé (sans emprunt mutable).
    pub fn modele_ennemi(&self, type_ennemi: usize) -> &ModeleCompile {
        &self.liste[1 + type_ennemi].modele
    }

    pub fn modele_allie(&self, type_allie: usize) -> &ModeleCompile {
        &self.liste[1 + self.nombre_ennemis + type_allie].modele
    }

    pub fn modele_creature(&self, k: usize) -> &ModeleCompile {
        let debut = 1 + self.nombre_ennemis + self.nombre_allies + self.nombre_coffres;
        &self.liste[debut + k].modele
    }

    pub fn modele_joueur(&self) -> &ModeleCompile {
        &self.liste[0].modele
    }

    /// Construit les rendus (textures déjà chargées ou introuvables) ; `portees` : monde
    /// de la pièce d'armure portée à chaque emplacement.
    #[allow(clippy::too_many_arguments)]
    pub fn construire(
        d: &'static Donnees,
        personnage: usize,
        portees: &[Option<Monde>; 7],
        ambiance: ParametresBloc,
        serveur: &AssetServer,
        textures: &Textures,
        images: &mut Assets<Image>,
        maillages: &mut Assets<Mesh>,
        materiaux: &mut Assets<MateriauEntite>,
    ) -> Self {
        let mut parametres = ambiance;
        parametres.brouillard.z = DECOUPE;
        parametres.brouillard.w = 0.0;
        let mut liste: Vec<RenduModele> = apparences(d, personnage)
            .map(|a| {
                let skin = textures.obtenir(serveur, &a.skin);
                let objet = objet_tenu(
                    a,
                    &ModeleCompile::nouveau(&d.modeles[a.modele.as_str()]),
                    serveur,
                    textures,
                    images,
                    maillages,
                    materiaux,
                    parametres,
                );
                rendu(d, a, skin, objet, maillages, materiaux, parametres)
            })
            .collect();
        let mut pieces = Vec::new();
        for (piece, monde) in armure::visibles(portees) {
            let Some(id) = armure::modele(piece) else {
                continue;
            };
            let a = Apparence {
                modele: id.to_owned(),
                skin: String::new(),
                couleurs_secours: Default::default(),
                couleur_secours_par_defaut: Some(armure::SECOURS_ARMURE),
                rayon_ombre: 0.0,
                objet_en_main: None,
            };
            let skin = armure::skin(d, piece, monde, serveur, textures, images);
            let r = rendu(d, &a, skin, None, maillages, materiaux, parametres);
            pieces.push((
                liste.len(),
                armure::correspondance(&liste[0].modele, &r.modele),
            ));
            liste.push(r);
        }
        Self {
            liste,
            nombre_ennemis: d.ennemis.len(),
            nombre_allies: d.allies.allies.len(),
            nombre_coffres: d.coffres.coffres.len(),
            armure: pieces,
        }
    }

    /// Dessine l'armure portée sur le joueur tel qu'il vient d'être posé (même place,
    /// mêmes rotations, même surcouche).
    pub fn poser_armure(&mut self, commandes: &mut Commands, q: &mut rendu::Composants) {
        for k in 0..self.armure.len() {
            let (i, _) = self.armure[k];
            let (joueur, reste) = self.liste.split_at_mut(i);
            let source = &joueur[0].pose;
            let r = &mut reste[0];
            let cible = &mut r.pose;
            cible.position = source.position;
            cible.angle = source.angle;
            cible.inclinaison = source.inclinaison;
            cible.echelle_xz = source.echelle_xz;
            cible.echelle_y = source.echelle_y;
            cible.surcouche = source.surcouche;
            for (partie, origine) in self.armure[k].1.iter().enumerate() {
                if let Some(o) = origine {
                    cible.rotations[partie] = source.rotations[*o];
                }
            }
            r.ajouter(commandes, q);
        }
    }
}

/// Rendu d'une apparence : un maillage par partie (blanc si texturé, sinon couleur de
/// secours), un matériau.
fn rendu(
    d: &'static Donnees,
    a: &Apparence,
    skin: Option<Handle<Image>>,
    objet: Option<ObjetTenu>,
    maillages: &mut Assets<Mesh>,
    materiaux: &mut Assets<MateriauEntite>,
    parametres: ParametresBloc,
) -> RenduModele {
    let def = &d.modeles[a.modele.as_str()];
    let modele = ModeleCompile::nouveau(def);
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
        lumiere: None,
    });
    RenduModele::nouveau(&a.modele, modele, parties, materiau, objet)
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
            lumiere: None,
        }),
        local,
    })
}

#[cfg(test)]
mod tests;
