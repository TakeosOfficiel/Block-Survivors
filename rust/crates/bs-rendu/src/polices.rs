//! Police des textes du menu (styles.css `--menu-police`) : Inter, embarquée (licence
//! OFL, `polices/OFL.txt`), en normal et demi-gras ; et celle du logo de l'accueil,
//! Lilita One (licence OFL, `polices/OFL-LilitaOne.txt`). La police pixel du jeu
//! (`hud::police`) reste celle des titres. `texte` donne le trio Bevy d'un texte.

use bevy::prelude::*;

const NORMALE: &[u8] = include_bytes!("polices/Inter-Regular.otf");
const GRASSE: &[u8] = include_bytes!("polices/Inter-SemiBold.otf");
const LOGO: &[u8] = include_bytes!("polices/LilitaOne-Regular.ttf");

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
        let font = if gras { &self.grasse } else { &self.normale };
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
        app.insert_resource(polices);
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
