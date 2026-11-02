//! Mises à jour dans le jeu (plan, section 5) : le service de `bs-maj` cherche en
//! arrière-plan dès le lancement, sur la voie choisie dans les options (stable ou bêta).
//! Au menu, un encart en bas à gauche montre le téléchargement, puis « Version x.y.z
//! prête » avec « Redémarrer » : l'installeur attend la fermeture du jeu, applique la
//! version et le relance. Sans clic, elle s'installe au lancement suivant ; pendant une
//! partie, rien ne s'affiche ni ne s'interrompt.

use bevy::prelude::*;
use bs_maj::{Etat, MiseAJour, Reglages};
use bs_sauvegarde::stockage::Stockage;

use crate::ecrans::style::{self, GRIS, OR_CLAIR};
use crate::hud::police::{TextePixel, ecrire};
use crate::menu::ElementMenu;
use crate::options::Options;

#[derive(Resource)]
pub struct Maj(pub MiseAJour);

#[derive(Component)]
pub struct Encart;

#[derive(Component)]
pub struct TexteMaj;

#[derive(Component)]
pub struct BoutonRedemarrer;

/// Au lancement : démarre la recherche.
pub fn demarrer(mut commandes: Commands, options: Res<Options>) {
    let fichier_serie = Stockage::du_systeme().dossier().join("maj-serie");
    let m = MiseAJour::demarrer(Reglages {
        fichier_serie,
        beta: options.beta,
    });
    if let Etat::Inactive(raison) = m.etat() {
        info!("mises à jour : {raison}");
    }
    commandes.insert_resource(Maj(m));
}

/// Option « Versions bêta » changée : le service change de voie.
pub fn suivre_voie(options: Res<Options>, maj: Res<Maj>) {
    if options.is_changed() {
        maj.0.changer_voie(options.beta);
    }
}

/// Entrée au menu : l'encart, caché tant qu'il n'y a rien à dire.
pub fn creer_encart(mut commandes: Commands, mut vu: Local<Option<Etat>>) {
    *vu = None;
    let encart = commandes
        .spawn((
            Name::new("mise à jour"),
            Encart,
            ElementMenu,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                bottom: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            GlobalZIndex(20),
            Visibility::Hidden,
        ))
        .id();
    commandes.spawn((
        TexteMaj,
        TextePixel::nouveau("", OR_CLAIR, 1.5),
        ChildOf(encart),
    ));
    let b = style::bouton(
        &mut commandes,
        encart,
        "Redémarrer",
        200.0,
        false,
        BoutonRedemarrer,
    );
    commandes.entity(b).insert(Visibility::Hidden);
}

/// Texte de l'encart pour un état (`None` : caché) et bouton visible.
fn texte(etat: &Etat) -> Option<(String, bool)> {
    match etat {
        Etat::Telechargement(p) => {
            Some((format!("Téléchargement de la mise à jour : {p} %"), false))
        }
        Etat::Prete(v) => Some((format!("Version {v} prête"), true)),
        _ => None,
    }
}

/// Suit l'état du service (seulement quand il change).
pub fn rafraichir(
    maj: Res<Maj>,
    mut vu: Local<Option<Etat>>,
    mut encart: Query<&mut Visibility, (With<Encart>, Without<BoutonRedemarrer>)>,
    mut bouton: Query<&mut Visibility, (With<BoutonRedemarrer>, Without<Encart>)>,
    mut textes: Query<&mut TextePixel, With<TexteMaj>>,
    ajoute: Query<(), Added<Encart>>,
) {
    let etat = maj.0.etat();
    if vu.as_ref() == Some(&etat) && ajoute.is_empty() {
        return;
    }
    if let Etat::Erreur(e) = &etat {
        warn!("mise à jour : {e}");
    }
    let affiche = texte(&etat);
    *vu = Some(etat);
    let visible = |oui: bool| {
        if oui {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    for mut v in &mut encart {
        v.set_if_neq(visible(affiche.is_some()));
    }
    for mut v in &mut bouton {
        v.set_if_neq(visible(affiche.as_ref().is_some_and(|a| a.1)));
    }
    if let (Some((t, _)), Ok(mut texte)) = (&affiche, textes.single_mut()) {
        ecrire(&mut texte, t);
    }
}

/// « Redémarrer » : l'installeur attend la fin du jeu, qui se ferme proprement.
pub fn cliquer(
    maj: Res<Maj>,
    boutons: Query<&Interaction, (Changed<Interaction>, With<BoutonRedemarrer>)>,
    mut textes: Query<&mut TextePixel, With<TexteMaj>>,
    mut sortie: MessageWriter<AppExit>,
) {
    if !boutons.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    match maj.0.redemarrer() {
        Ok(()) => {
            sortie.write(AppExit::Success);
        }
        Err(e) => {
            if let Ok(mut t) = textes.single_mut() {
                t.couleur = GRIS;
                ecrire(&mut t, &format!("Mise à jour impossible : {e}"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn textes_de_l_encart() {
        assert_eq!(texte(&Etat::AJour), None);
        assert_eq!(texte(&Etat::Recherche), None);
        assert_eq!(texte(&Etat::Erreur("x".into())), None);
        assert_eq!(
            texte(&Etat::Prete("0.2.0".into())),
            Some(("Version 0.2.0 prête".into(), true))
        );
        assert!(texte(&Etat::Telechargement(40)).is_some_and(|t| t.0.contains("40 %") && !t.1));
    }
}
