//! Lot d'instances d'un même maillage et d'un même matériau (l'équivalent d'un
//! `THREE.InstancedMesh`) : une entité par instance, créée au premier dépassement du
//! record puis réutilisée ; les instances non posées sont cachées.

use bevy::mesh::MeshTag;
use bevy::prelude::*;

use crate::materiau_entite::MateriauEntite;
use crate::modeles::rendu::Composants;

pub struct Lot {
    maillage: Handle<Mesh>,
    materiau: Handle<MateriauEntite>,
    entites: Vec<Entity>,
    utilisees: usize,
    nom: String,
}

impl Lot {
    pub fn nouveau(nom: &str, maillage: Handle<Mesh>, materiau: Handle<MateriauEntite>) -> Self {
        Self {
            maillage,
            materiau,
            entites: Vec::new(),
            utilisees: 0,
            nom: nom.to_owned(),
        }
    }

    pub fn commencer(&mut self) {
        self.utilisees = 0;
    }

    /// Pose une instance de plus (`tag` : teinte ou surcouche, voir le shader).
    pub fn ajouter(
        &mut self,
        commandes: &mut Commands,
        q: &mut Composants,
        transform: Transform,
        tag: &MeshTag,
    ) {
        let i = self.utilisees;
        self.utilisees += 1;
        if i == self.entites.len() {
            let e = commandes
                .spawn((
                    Name::new(self.nom.clone()),
                    Mesh3d(self.maillage.clone()),
                    MeshMaterial3d(self.materiau.clone()),
                    transform,
                    tag.clone(),
                ))
                .id();
            self.entites.push(e);
            return;
        }
        if let Ok((mut t, mut v, m)) = q.get_mut(self.entites[i]) {
            *t = transform;
            v.set_if_neq(Visibility::Inherited);
            if let Some(mut m) = m {
                m.set_if_neq(tag.clone());
            }
        }
    }

    /// Cache les instances non posées depuis `commencer`.
    pub fn terminer(&mut self, q: &mut Composants) {
        for &e in &self.entites[self.utilisees..] {
            if let Ok((_, mut v, _)) = q.get_mut(e) {
                v.set_if_neq(Visibility::Hidden);
            }
        }
    }
}
