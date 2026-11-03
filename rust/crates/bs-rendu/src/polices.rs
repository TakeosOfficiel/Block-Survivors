//! Police des textes du menu (styles.css `--menu-police`) : Inter, embarquée (licence
//! OFL, `polices/OFL.txt`), en normal et demi-gras ; et celle du logo de l'accueil,
//! Luckiest Guy (licence Apache 2.0, `polices/LICENCE-LuckiestGuy.txt`). La police pixel du jeu
//! (`hud::police`) ne reste qu'en jeu, dans le HUD : partout ailleurs (menus, écrans
//! de cartes, pause, bilan, chargement), les titres `TextePixel` sont redessinés en
//! Luckiest Guy (`titres`), comme les libellés en gras d'au moins `GRAS_TITRE` px.
//! `texte` donne le trio Bevy d'un texte.

mod titres;

use bevy::prelude::*;

const NORMALE: &[u8] = include_bytes!("polices/Inter-Regular.otf");
const GRASSE: &[u8] = include_bytes!("polices/Inter-SemiBold.otf");
const LOGO: &[u8] = include_bytes!("polices/LuckiestGuy-Regular.ttf");

/// Taille (px) à partir de laquelle un texte en gras prend la police du logo.
pub const GRAS_TITRE: f32 = 17.0;
/// Agrandissement des libellés en police du logo.
const AGRANDI: f32 = 1.15;

/// Les deux graisses d'Inter et la police du logo.
#[derive(Resource, Clone)]
pub struct PolicesMenu {
    pub normale: Handle<Font>,
    pub grasse: Handle<Font>,
    pub logo: Handle<Font>,
}

impl PolicesMenu {
    /// Texte du menu : police, taille (px), couleur.
    pub fn texte(
        &self,
        t: impl Into<String>,
        gras: bool,
        taille: f32,
        couleur: Color,
    ) -> (Text, TextFont, TextColor) {
        // Libellés en gras (boutons, liens, noms) : police du logo ; petits : Inter
        // (un peu plus grande : ses minuscules sont des petites capitales)
        let (font, taille) = match (gras, taille >= GRAS_TITRE) {
            (true, true) => (&self.logo, taille * AGRANDI),
            (true, false) => (&self.grasse, taille),
            _ => (&self.normale, taille),
        };
        (
            Text::new(t),
            TextFont {
                font: font.clone(),
                font_size: taille,
                ..default()
            },
            TextColor(couleur),
        )
    }
}

pub struct PluginPolices;

impl Plugin for PluginPolices {
    fn build(&self, app: &mut App) {
        // Tout de suite : l'entrée au menu (premier état) peut passer avant `PreStartup`
        let polices = charger(&mut app.world_mut().resource_mut::<Assets<Font>>());
        app.insert_resource(polices).add_systems(
            PostUpdate,
            (titres::convertir, titres::suivre)
                .chain()
                .before(crate::hud::police::dessiner),
        );
    }
}

fn charger(fontes: &mut Assets<Font>) -> PolicesMenu {
    let mut ajouter = |octets: &[u8]| match Font::try_from_bytes(octets.to_vec()) {
        Ok(f) => fontes.add(f),
        Err(e) => {
            // Police par défaut de Bevy à la place
            error!("police du menu illisible : {e}");
            Handle::default()
        }
    };
    let normale = ajouter(NORMALE);
    let grasse = ajouter(GRASSE);
    let logo = ajouter(LOGO);
    PolicesMenu {
        normale,
        grasse,
        logo,
    }
}
