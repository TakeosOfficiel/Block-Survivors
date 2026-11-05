//! Maps Minecraft importées (`cartes/*.schem`, src/minecraft/importation.ts) : pendant le
//! chargement, on cuit la palette (états et modèles de blocs téléchargés au fil des
//! besoins), puis on crée la partie et on demande les textures ; à l'entrée en partie,
//! un maillage par texture, rendu opaque, découpé ou translucide d'après ses pixels.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_donnees::arenes::Arene;
use bs_monde::lumiere::volume_de_schematique;
use bs_monde::minecraft::cartes::{Carte, definition, importer};
use bs_monde::minecraft::maillage::{
    MaillageTexture, RenduTexture, classer, construire_maillages, opaques,
};
use bs_monde::minecraft::modeles::{BlocCuit, cuire, textures as textures_blocs};
use bs_monde::minecraft::schematique::{Schematique, lire_schematique};
use bs_regles::partie::Partie;

use crate::arene::couleur_lineaire;
use crate::fichiers::{Fichiers, Json};
use crate::materiau::{ATTRIBUTE_VENT, AnimationTexture, MateriauBloc, ParametresBloc};
use crate::textures::Textures;
use crate::{PartieEnCours, SourceArene, modeles};

/// Vitesse des textures animées vanilla absentes des blocs du jeu (images par seconde,
/// d'après le `frametime` de leur `.mcmeta`) ; les autres bandes d'images vont à
/// `IMAGES_PAR_SECONDE_DEFAUT`.
const ANIMATIONS_VANILLA: &[(&str, f32)] = &[
    ("block/prismarine", 0.07),
    ("block/sea_lantern", 4.0),
    ("block/lantern", 2.5),
    ("block/soul_lantern", 2.5),
    ("block/fire_0", 20.0),
    ("block/fire_1", 20.0),
    ("block/soul_fire_0", 20.0),
    ("block/soul_fire_1", 20.0),
    ("block/nether_portal", 20.0),
];
const IMAGES_PAR_SECONDE_DEFAUT: f32 = 10.0;

/// Blocs de la map sans modèle (et ennemis inconnus des panneaux), annoncés au joueur au début de la partie.
#[derive(Resource)]
pub struct RapportImport(pub String);

impl RapportImport {
    /// « 3 élément(s) inconnu(s) : a, b, c » (les quatre premiers noms, puis « ... »).
    fn nouveau(inconnus: &[String]) -> Self {
        let mut t = format!("Map : {} élément(s) inconnu(s) : ", inconnus.len());
        t.push_str(&inconnus[..inconnus.len().min(4)].join(", "));
        if inconnus.len() > 4 {
            t.push_str(", ...");
        }
        Self(t)
    }
}

/// Map en cours d'import (phase de chargement).
#[derive(Resource)]
pub struct Import {
    d: &'static Donnees,
    schematique: Option<Schematique>,
    definition: Option<Arene>,
    personnage: usize,
    graine: u32,
    talents: Option<crate::sauvegarde::Depart>,
}

impl Import {
    /// Arène en cours d'import (avant sa construction).
    pub fn arene(&self) -> Option<&Arene> {
        self.definition.as_ref()
    }
}

impl Import {
    /// Lit la schématique embarquée et la définition de l'arène.
    pub fn nouvel(
        d: &'static Donnees,
        carte: &Carte,
        personnage: usize,
        graine: u32,
        talents: crate::sauvegarde::Depart,
    ) -> Result<Self, String> {
        Ok(Self {
            d,
            schematique: Some(lire_schematique(carte.fichier)?),
            definition: Some(definition(d, carte)?),
            personnage,
            graine,
            talents: Some(talents),
        })
    }
}

/// Map importée, à afficher.
#[derive(Resource)]
pub struct CarteImportee {
    schematique: Schematique,
    blocs: Vec<BlocCuit>,
    /// Triées (recherche dichotomique dans `construire_maillages`).
    textures: Vec<String>,
    origine: [f32; 3],
}

/// Une étape du chargement : cuisson tant que des fichiers manquent, puis partie.
pub fn avancer_import(
    mut commandes: Commands,
    serveur: Res<AssetServer>,
    mut fichiers: ResMut<Fichiers>,
    json: Res<Assets<Json>>,
    mut textures: ResMut<Textures>,
    import: Option<ResMut<Import>>,
) {
    let Some(mut import) = import else {
        return;
    };
    let Some(s) = import.schematique.as_ref() else {
        return;
    };
    let blocs = match cuire(&s.palette, &fichiers.vue(&serveur, &json)) {
        Ok(b) => b,
        Err(manquants) => {
            for chemin in &manquants {
                fichiers.demander(&serveur, chemin);
            }
            return;
        }
    };
    let mut s = import.schematique.take().expect("schématique lue");
    let def = import.definition.take().expect("définition lue");
    if def.biome.is_some() {
        s.biome.clone_from(&def.biome);
    }
    let ai = match importer(import.d, def, &s, &blocs) {
        Ok(ai) => ai,
        Err(e) => panic!("map importée illisible : {e}"),
    };
    if !ai.inconnus.is_empty() {
        warn!("blocs sans modèle : {}", ai.inconnus.join(", "));
        commandes.insert_resource(RapportImport::nouveau(&ai.inconnus));
    }
    let d = import.d;
    let arene: &'static Arene = Box::leak(Box::new(ai.arene));
    let partie = Partie::avec_arene(
        d,
        arene,
        ai.carte.clone(),
        import.personnage,
        import.graine,
        true,
    );
    let liste = textures_blocs(&blocs);
    for t in &liste {
        textures.demander(&serveur, t);
    }
    crate::effets::demander_textures(d, &serveur, &mut textures);
    let jeu = PartieEnCours::nouvelle(
        partie,
        import.graine,
        SourceArene::Importee(arene, ai.carte),
        import.talents.take().unwrap_or_default(),
    );
    modeles::demander_textures(d, jeu.apparence_joueur(), &serveur, &mut textures);
    crate::retours::demander_textures(&jeu, &serveur, &mut textures);
    crate::armes::demander_textures(d, &serveur, &mut textures);
    commandes.insert_resource(jeu);
    commandes.insert_resource(CarteImportee {
        schematique: s,
        blocs,
        textures: liste,
        origine: ai.origine,
    });
    commandes.remove_resource::<Import>();
}

/// Maillages de la map (textures chargées).
pub fn construire_carte(
    mut commandes: Commands,
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    mut images: ResMut<Assets<Image>>,
    jeu: Res<PartieEnCours>,
    carte: Res<CarteImportee>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<MateriauBloc>>,
) {
    let d = jeu.partie.d;
    let ambiance = ParametresBloc::de_l_arene(jeu.partie.etat.arene);
    let poignees: Vec<Option<Handle<Image>>> = carte
        .textures
        .iter()
        .map(|t| textures.obtenir(&serveur, t))
        .collect();
    let image = |k: usize| poignees[k].as_ref().and_then(|p| images.get(p));
    let rendus: Vec<RenduTexture> = (0..carte.textures.len())
        .map(|k| {
            image(k)
                .and_then(|i| Some(classer(i.data.as_ref()?, i.width() as usize)))
                .unwrap_or(RenduTexture::Opaque)
        })
        .collect();
    let liste = construire_maillages(
        &carte.schematique,
        &carte.blocs,
        &carte.textures,
        &rendus,
        carte.origine,
    );
    for m in &liste {
        let chemin = &carte.textures[m.texture];
        let rendu = rendus[m.texture];
        let mut parametres = ambiance;
        parametres.brouillard.z = match rendu {
            RenduTexture::Opaque => 0.0,
            RenduTexture::Decoupe => 0.5,
            RenduTexture::Translucide => 0.02,
        };
        // Bande d'images : toutes les images défilent (réglage du jeu, sinon vanilla)
        let images_bande = image(m.texture).map_or(1, |i| (i.height() / i.width().max(1)).max(1));
        parametres.animation.x = images_bande as f32;
        parametres.animation.w = if rendu == RenduTexture::Translucide {
            1.0
        } else {
            0.0
        };
        let materiau = materiaux.add(MateriauBloc {
            parametres,
            texture: poignees[m.texture].clone(),
            lumiere: None,
            double_face: rendu != RenduTexture::Opaque,
            translucide: rendu == RenduTexture::Translucide,
        });
        let mut entite = commandes.spawn((
            Name::new(format!("map:{chemin}")),
            Mesh3d(maillages.add(en_mesh(m))),
            MeshMaterial3d(materiau.clone()),
        ));
        let animation = d.blocs.values().find_map(|b| {
            let f = &b.faces;
            [&f.haut, &f.cotes, &f.bas]
                .contains(&chemin)
                .then_some(b.animation.as_ref())
                .flatten()
        });
        let animation = match animation {
            Some(a) => Some((a.images_par_seconde, a.aller_retour)),
            // Bande d'images d'un bloc vanilla sans réglage dans le jeu : animée quand même
            None if images_bande > 1 => {
                let vitesse = ANIMATIONS_VANILLA.iter().find(|(c, _)| c == chemin);
                Some((vitesse.map_or(IMAGES_PAR_SECONDE_DEFAUT, |v| v.1), false))
            }
            None => None,
        };
        if let Some((images_par_seconde, aller_retour)) = animation {
            entite.insert(AnimationTexture {
                materiau,
                images_par_seconde,
                aller_retour,
            });
        }
    }
    eclairer(&mut commandes, &mut images, &carte, &rendus);
}

/// Lumière des blocs de la map (la case (0, 0, 0) de la schématique a son coin en
/// `-origine` dans le monde).
fn eclairer(
    commandes: &mut Commands,
    images: &mut Assets<Image>,
    carte: &CarteImportee,
    rendus: &[RenduTexture],
) {
    let opaque = opaques(&carte.blocs, &carte.textures, rendus);
    let v = volume_de_schematique(&carte.schematique, &carte.blocs, &opaque);
    let o = carte.origine;
    crate::lumiere::inserer(commandes, images, &v, -Vec3::new(o[0], o[1], o[2]));
}

/// Maillage Bevy d'une texture de la map (teintes en couleurs de sommets).
fn en_mesh(m: &MaillageTexture) -> Mesh {
    let couleurs: Vec<[f32; 4]> = m.couleurs.iter().map(|c| couleur_lineaire(*c)).collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, m.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, m.normales.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, m.uvs.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, couleurs)
    .with_inserted_attribute(ATTRIBUTE_VENT, m.vents.clone())
    .with_inserted_indices(Indices::U32(m.indices.clone()))
}
