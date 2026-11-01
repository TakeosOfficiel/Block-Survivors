//! Rendu d'un modèle (src/modeles/rendu.ts) : une entité Bevy par partie, enfant d'une
//! racine par instance. Toutes les instances d'un modèle partagent maillages et
//! matériau : Bevy les dessine en un appel par partie (instanciation automatique).
//!
//! Chaque image, l'appelant pose les instances avec `ajouter` entre `commencer` et
//! `terminer` ; les instances en trop sont cachées, jamais détruites. Une instance
//! n'est créée que la première fois que leur nombre dépasse le record.

use bevy::mesh::MeshTag;
use bevy::prelude::*;

use super::compilation::{ModeleCompile, PIXELS_PAR_BLOC};
use crate::materiau_entite::{MateriauEntite, tag_surcouche};

/// Pose d'une instance, remplie par l'appelant (réutilisée d'une image à l'autre).
#[derive(Debug, Clone)]
pub struct Pose {
    pub position: Vec3,
    /// Lacet (rad), 0 : face vers +Z.
    pub angle: f32,
    /// Chute sur le côté (rad).
    pub inclinaison: f32,
    pub echelle_xz: f32,
    pub echelle_y: f32,
    /// Rotations (x, y, z) de chaque partie.
    pub rotations: Vec<Vec3>,
    /// Surcouche (r, g, b, force) ; force 0 : aucune.
    pub surcouche: [f32; 4],
}

impl Pose {
    pub fn nouvelle(m: &ModeleCompile) -> Self {
        Self {
            position: Vec3::ZERO,
            angle: 0.0,
            inclinaison: 0.0,
            echelle_xz: 1.0,
            echelle_y: 1.0,
            rotations: m.pose_base.clone(),
            surcouche: [0.0; 4],
        }
    }

    fn racine(&self) -> Transform {
        Transform {
            translation: self.position,
            rotation: Quat::from_rotation_y(self.angle) * Quat::from_rotation_z(self.inclinaison),
            scale: Vec3::new(self.echelle_xz, self.echelle_y, self.echelle_xz),
        }
    }
}

/// Objet tenu : son maillage, son matériau et sa place dans le repère de la main.
#[derive(Debug, Clone)]
pub struct ObjetTenu {
    pub maillage: Handle<Mesh>,
    pub materiau: Handle<MateriauEntite>,
    pub local: Transform,
}

impl ObjetTenu {
    pub fn local(m: &ModeleCompile) -> Option<Transform> {
        let main = m.def.main.as_ref()?;
        let [rx, ry, rz] = main.rotation;
        Some(Transform {
            translation: Vec3::from_array(main.position) / PIXELS_PAR_BLOC,
            rotation: Quat::from_euler(EulerRot::XYZ, rx, ry, rz),
            ..default()
        })
    }
}

struct Instance {
    racine: Entity,
    parties: Vec<Entity>,
    objet: Option<Entity>,
}

pub struct RenduModele {
    pub modele: ModeleCompile,
    /// Pose de travail de l'appelant.
    pub pose: Pose,
    maillages: Vec<Handle<Mesh>>,
    materiau: Handle<MateriauEntite>,
    objet: Option<ObjetTenu>,
    instances: Vec<Instance>,
    utilisees: usize,
    nom: String,
}

/// Accès aux composants des instances (un seul paramètre de système).
pub type Composants<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut Visibility,
        Option<&'static mut MeshTag>,
    ),
>;

impl RenduModele {
    pub fn nouveau(
        nom: &str,
        modele: ModeleCompile,
        maillages: Vec<Handle<Mesh>>,
        materiau: Handle<MateriauEntite>,
        objet: Option<ObjetTenu>,
    ) -> Self {
        Self {
            pose: Pose::nouvelle(&modele),
            modele,
            maillages,
            materiau,
            objet,
            instances: Vec::new(),
            utilisees: 0,
            nom: nom.to_owned(),
        }
    }

    /// Entités racines des instances créées.
    pub fn racines(&self) -> impl Iterator<Item = Entity> + '_ {
        self.instances.iter().map(|i| i.racine)
    }

    pub fn commencer(&mut self) {
        self.utilisees = 0;
    }

    /// Pose une instance de plus avec `self.pose`.
    pub fn ajouter(&mut self, commandes: &mut Commands, q: &mut Composants) {
        let i = self.utilisees;
        self.utilisees += 1;
        let racine = self.pose.racine();
        let tag = tag_surcouche(self.pose.surcouche);
        if i == self.instances.len() {
            let instance = self.creer(commandes, racine, &tag);
            self.instances.push(instance);
            return;
        }
        let inst = &self.instances[i];
        if let Ok((mut t, mut v, _)) = q.get_mut(inst.racine) {
            *t = racine;
            v.set_if_neq(Visibility::Inherited);
        }
        for (k, &e) in inst.parties.iter().enumerate() {
            if let Ok((mut t, _, Some(mut m))) = q.get_mut(e) {
                t.rotation = rotation_partie(self.pose.rotations[k]);
                m.set_if_neq(tag.clone());
            }
        }
        if let Some(e) = inst.objet
            && let Ok((_, _, Some(mut m))) = q.get_mut(e)
        {
            m.set_if_neq(tag);
        }
    }

    /// Cache les instances non posées depuis `commencer`.
    pub fn terminer(&mut self, q: &mut Composants) {
        for inst in &self.instances[self.utilisees..] {
            if let Ok((_, mut v, _)) = q.get_mut(inst.racine) {
                v.set_if_neq(Visibility::Hidden);
            }
        }
    }

    fn creer(&self, commandes: &mut Commands, racine: Transform, tag: &MeshTag) -> Instance {
        let id_racine = commandes
            .spawn((Name::new(self.nom.clone()), racine, Visibility::Inherited))
            .id();
        let mut objet = None;
        let parties = self
            .modele
            .parties
            .iter()
            .enumerate()
            .map(|(k, p)| {
                let partie = commandes
                    .spawn((
                        Name::new(p.nom),
                        Mesh3d(self.maillages[k].clone()),
                        MeshMaterial3d(self.materiau.clone()),
                        Transform {
                            translation: p.pivot,
                            rotation: rotation_partie(self.pose.rotations[k]),
                            ..default()
                        },
                        tag.clone(),
                        ChildOf(id_racine),
                    ))
                    .id();
                if let Some(o) = &self.objet
                    && self.modele.main == Some(k)
                {
                    objet = Some(
                        commandes
                            .spawn((
                                Mesh3d(o.maillage.clone()),
                                MeshMaterial3d(o.materiau.clone()),
                                o.local,
                                tag.clone(),
                                ChildOf(partie),
                            ))
                            .id(),
                    );
                }
                partie
            })
            .collect();
        Instance {
            racine: id_racine,
            parties,
            objet,
        }
    }
}

/// Rotation d'une partie : ordre Z, Y, X comme les modèles du jeu.
fn rotation_partie(r: Vec3) -> Quat {
    Quat::from_euler(EulerRot::ZYX, r.z, r.y, r.x)
}
