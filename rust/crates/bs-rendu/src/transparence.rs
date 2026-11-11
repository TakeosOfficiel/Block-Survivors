//! Vue à travers les blocs : en partie, les matériaux des blocs reçoivent la position
//! des pieds du joueur, et leur shader ne dessine pas ce qui le cache à la caméra
//! au-dessus de ses épaules (plafond d'une cave, toit d'une maison, haut d'un mur).
//! Les ombres restent celles des blocs entiers. Les faces arrière ainsi découvertes
//! sont dessinées en tranche pleine et sombre : on ne voit jamais sous la carte.

use bevy::prelude::*;

use crate::camera::Cible;
use crate::materiau::MateriauBloc;

/// Rayon (blocs) dégagé autour du joueur.
const RAYON: f32 = 3.0;
/// Déplacement (blocs) qui fait renvoyer la position aux matériaux.
const SEUIL: f32 = 0.05;

/// Envoie les pieds du joueur aux matériaux des blocs quand il a bougé (ou qu'une
/// arène vient d'être créée).
pub fn suivre(
    cible: Res<Cible>,
    mut evenements: MessageReader<AssetEvent<MateriauBloc>>,
    mut materiaux: ResMut<Assets<MateriauBloc>>,
) {
    let nouveaux = evenements
        .read()
        .any(|e| matches!(e, AssetEvent::Added { .. }));
    let coupe = Vec4::new(cible.0.x, cible.1, cible.0.y, RAYON);
    poser(&mut materiaux, coupe, nouveaux);
}

/// Hors partie (menu, accueil), rien n'est coupé.
pub fn arreter(mut materiaux: ResMut<Assets<MateriauBloc>>) {
    poser(&mut materiaux, Vec4::ZERO, true);
}

fn poser(materiaux: &mut Assets<MateriauBloc>, coupe: Vec4, toujours: bool) {
    let loin = |m: &MateriauBloc| {
        let a = m.parametres.coupe;
        a.w != coupe.w || a.truncate().distance(coupe.truncate()) > SEUIL
    };
    if !toujours && !materiaux.iter().any(|(_, m)| loin(m)) {
        return;
    }
    let ids: Vec<_> = materiaux
        .iter()
        .filter(|(_, m)| loin(m))
        .map(|(id, _)| id)
        .collect();
    for id in ids {
        if let Some(m) = materiaux.get_mut(id) {
            m.parametres.coupe = coupe;
        }
    }
}
