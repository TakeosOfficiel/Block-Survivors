//! Options du joueur (src/options.ts) : volume général et de chaque famille de sons,
//! compteur de FPS, secousses de l'écran, ombres, chiffres et compteur des dégâts, vue
//! (F5). Enregistrées pour toute l'installation (« block-survivors.options », même format
//! que le jeu TS : les options pas encore reprises, effets des alliés, sont gardées
//! telles quelles). Chaque changement est appliqué et enregistré aussitôt. Le panneau
//! (`panneau`) s'ouvre du menu principal et de la pause.

mod panneau;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::light::DirectionalLightShadowMap;
use bevy::prelude::*;
use bs_audio::Volumes;
use bs_sauvegarde::Stockage;
use serde_json::{Value, json};

use crate::Phase;
use crate::hud::police::{TextePixel, ecrire};
use crate::soleil::OmbresSoleil;
use crate::sons::Sons;

pub use panneau::PanneauOptions;

const CLE: &str = "block-survivors.options";
/// Format enregistré (src/options.ts `VERSION`).
const VERSION: u64 = 2;
pub const CATEGORIES: [&str; 4] = ["joueur", "armes", "creatures", "interface"];
const NIVEAUX_OMBRES: [&str; 3] = ["aucun", "legers", "complets"];

#[derive(Resource, Clone, PartialEq, Debug)]
pub struct Options {
    pub volume: f32,
    /// Dans l'ordre de `CATEGORIES`.
    pub volumes: [f32; 4],
    pub afficher_fps: bool,
    pub secousses: bool,
    /// Index dans « aucun », « legers », « complets ».
    pub ombres: usize,
    pub chiffres_degats: bool,
    /// Compteur de dégâts en jeu (Tab).
    pub compteur_degats: bool,
    /// Vue à la 3e personne (F5) plutôt que de dessus.
    pub troisieme_personne: bool,
    /// Mises à jour de la voie bêta plutôt que stable.
    pub beta: bool,
    /// Le reste du fichier (options pas encore reprises), réécrit tel quel.
    autres: serde_json::Map<String, Value>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            volume: 0.7,
            volumes: [1.0; 4],
            afficher_fps: false,
            secousses: true,
            ombres: 1,
            chiffres_degats: true,
            compteur_degats: true,
            troisieme_personne: false,
            beta: false,
            autres: serde_json::Map::new(),
        }
    }
}

impl Options {
    /// Lit les options ; toute valeur absente ou invalide garde sa valeur par défaut.
    pub fn lire(lu: Option<Value>) -> Self {
        let mut o = Self::default();
        let Some(Value::Object(lu)) = lu else {
            return o;
        };
        let nombre = |v: &Value| v.as_f64().map(|x| x.clamp(0.0, 1.0) as f32);
        if let Some(v) = lu.get("volume").and_then(nombre) {
            o.volume = v;
        }
        for (k, c) in CATEGORIES.iter().enumerate() {
            if let Some(v) = lu.get("volumes").and_then(|v| v.get(c)).and_then(nombre) {
                o.volumes[k] = v;
            }
        }
        let oui = |cle: &str, defaut: bool| lu.get(cle).and_then(Value::as_bool).unwrap_or(defaut);
        o.afficher_fps = oui("afficherFps", o.afficher_fps);
        o.secousses = oui("secousses", o.secousses);
        o.chiffres_degats = oui("chiffresDegats", o.chiffres_degats);
        o.compteur_degats = oui("compteurDegats", o.compteur_degats);
        o.beta = oui("voieBeta", o.beta);
        o.troisieme_personne = lu.get("vue").and_then(Value::as_str) == Some("troisieme");
        // Avant la version 2, les ombres repartent activées
        if lu.get("version").and_then(Value::as_u64) == Some(VERSION) {
            let niveau = lu.get("shaders").and_then(Value::as_str);
            if let Some(k) = NIVEAUX_OMBRES.iter().position(|n| Some(*n) == niveau) {
                o.ombres = k;
            }
        }
        for (cle, v) in &lu {
            let reprise = [
                "volume",
                "volumes",
                "afficherFps",
                "secousses",
                "chiffresDegats",
                "compteurDegats",
                "vue",
                "voieBeta",
                "shaders",
                "version",
            ];
            if !reprise.contains(&cle.as_str()) {
                o.autres.insert(cle.clone(), v.clone());
            }
        }
        o
    }

    pub fn json(&self) -> Value {
        let mut v = self.autres.clone();
        let volumes: serde_json::Map<String, Value> = CATEGORIES
            .iter()
            .zip(self.volumes)
            .map(|(c, x)| ((*c).to_owned(), json!(x)))
            .collect();
        v.insert("volume".into(), json!(self.volume));
        v.insert("volumes".into(), Value::Object(volumes));
        v.insert("afficherFps".into(), json!(self.afficher_fps));
        v.insert("secousses".into(), json!(self.secousses));
        v.insert("shaders".into(), json!(NIVEAUX_OMBRES[self.ombres]));
        v.insert("chiffresDegats".into(), json!(self.chiffres_degats));
        v.insert("compteurDegats".into(), json!(self.compteur_degats));
        let vue = if self.troisieme_personne {
            "troisieme"
        } else {
            "dessus"
        };
        v.insert("vue".into(), json!(vue));
        v.insert("voieBeta".into(), json!(self.beta));
        v.insert("version".into(), json!(VERSION));
        Value::Object(v)
    }

    pub fn volumes(&self) -> Volumes {
        Volumes {
            general: self.volume,
            categories: self.volumes,
        }
    }
}

/// Stockage des options (le dossier des sauvegardes).
#[derive(Resource)]
struct StockageOptions(Stockage);

/// Compteur de FPS (coin haut droit).
#[derive(Component)]
struct CompteurFps;

pub struct PluginOptions;

impl Plugin for PluginOptions {
    fn build(&self, app: &mut App) {
        let stockage = Stockage::du_systeme();
        let options = Options::lire(stockage.lire(CLE));
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .insert_resource(options)
            .insert_resource(StockageOptions(stockage))
            .init_resource::<PanneauOptions>()
            .add_systems(Startup, creer_compteur)
            .add_systems(
                Update,
                (panneau::gerer, panneau::rafraichir, appliquer, compter_fps)
                    .chain()
                    .after(crate::menu::gerer)
                    .after(crate::ecrans::gerer),
            )
            .add_systems(OnExit(Phase::Menu), panneau::fermer)
            .add_systems(OnExit(Phase::Partie), panneau::fermer);
    }
}

/// Applique et enregistre les options changées (et une première fois au lancement).
fn appliquer(
    options: Res<Options>,
    stockage: Res<StockageOptions>,
    mut sons: NonSendMut<Sons>,
    mut ombres: ResMut<OmbresSoleil>,
    mut carte: ResMut<DirectionalLightShadowMap>,
    mut premiere: Local<bool>,
) {
    if !options.is_changed() {
        return;
    }
    if std::mem::replace(&mut *premiere, true) {
        stockage.0.ecrire(CLE, &options.json());
    }
    sons.regler(options.volumes());
    let forcees_off = std::env::var("BS_OMBRES").is_ok_and(|v| v == "0");
    let actives = options.ombres > 0 && !forcees_off;
    if ombres.0 != actives {
        ombres.0 = actives;
    }
    let taille = if options.ombres == 2 { 4096 } else { 2048 };
    if carte.size != taille {
        carte.size = taille;
    }
}

fn creer_compteur(mut commandes: Commands) {
    commandes.spawn((
        Name::new("compteur de FPS"),
        CompteurFps,
        crate::lancement::Permanent,
        TextePixel::nouveau("", Color::WHITE, 1.5).contour(),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(6.0),
            top: Val::Px(4.0),
            ..default()
        },
        GlobalZIndex(40),
        Visibility::Hidden,
    ));
}

/// FPS lissées, deux fois par seconde.
fn compter_fps(
    options: Res<Options>,
    diagnostics: Res<DiagnosticsStore>,
    temps: Res<Time<Real>>,
    mut compteur: Query<(&mut TextePixel, &mut Visibility), With<CompteurFps>>,
    mut attente: Local<f32>,
) {
    let Ok((mut texte, mut visibilite)) = compteur.single_mut() else {
        return;
    };
    let voulue = if options.afficher_fps {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    visibilite.set_if_neq(voulue);
    *attente -= temps.delta_secs();
    if !options.afficher_fps || *attente > 0.0 {
        return;
    }
    *attente = 0.5;
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed());
    if let Some(fps) = fps {
        ecrire(&mut texte, &format!("{fps:.0} FPS"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lecture_du_format_ts() {
        let lu = json!({
            "volume": 0.4, "volumes": {"joueur": 0.5, "armes": 2.0},
            "afficherFps": true, "secousses": false, "shaders": "aucun",
            "chiffresDegats": false, "compteurDegats": false, "vue": "troisieme",
            "effetsAllies": true, "version": 2
        });
        let o = Options::lire(Some(lu));
        assert_eq!(o.volume, 0.4);
        assert_eq!(o.volumes, [0.5, 1.0, 1.0, 1.0]);
        assert!(o.afficher_fps && !o.secousses && !o.chiffres_degats && !o.compteur_degats);
        assert!(o.troisieme_personne);
        assert_eq!(o.ombres, 0);
        // Les options non reprises sont réécrites telles quelles
        let v = o.json();
        assert_eq!(v["vue"], "troisieme");
        assert_eq!(v["effetsAllies"], true);
        assert_eq!(v["compteurDegats"], false);
        assert_eq!(v["shaders"], "aucun");
        // Version 1 : ombres réactivées
        let o = Options::lire(Some(json!({"shaders": "aucun", "version": 1})));
        assert_eq!(o.ombres, 1);
        assert_eq!(
            Options::lire(Some(json!("n'importe quoi"))),
            Options::default()
        );
    }
}
