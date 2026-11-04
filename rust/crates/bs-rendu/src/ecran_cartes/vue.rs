//! Mise en page de l'écran de cartes, au style des menus : voile de nuit, grand titre
//! doré en police du logo, cartes sans cadre (fond de nuit arrondi, lueur de la couleur
//! de la famille en haut, rareté pour les objets), seule la carte survolée prend un liseré
//! d'or et se soulève. Zones fixes : famille en surtitre coloré (pastille ronde), touche
//! dans un rond, icône dans un halo, nom, stat clé, étiquette en pilule, description,
//! autres lignes « avant → après » (nouvelle valeur en vert). Sous les cartes :
//! « Relancer (R) » en pilule et « Laisser les objets » (refus d'un coffre) en lien.
//! Tout est reconstruit à chaque ouverture ou relance (jamais pendant le jeu).

use bevy::prelude::*;
use bs_donnees::equipement::Rarete;
use bs_regles::textes::{Carte, Famille, Icone};

use crate::dessins::Dessins;
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF, Relief, VERT_GAIN};
use crate::ecrans::nuit::{self, Style, ecrit, nuit as sombre, voile};
use crate::polices::PolicesMenu;
use crate::textures::Textures;

use super::propositions;

const DESCRIPTION: Color = Color::srgb_u8(0xc8, 0xcc, 0xd6);
const STATS: Color = Color::srgb_u8(0xcf, 0xe9, 0xc8);
const LARGEUR_CARTE: f32 = 290.0;
const MEDAILLON: f32 = 150.0;

/// Carte cliquable (index dans la liste).
#[derive(Component)]
pub struct BoutonCarte(pub usize);

/// Médaillon d'une carte (liseré d'or au survol) et sa couleur au repos.
#[derive(Component)]
pub struct Halo(pub Entity, pub Color);

/// Bouton « Relancer ».
#[derive(Component)]
pub struct BoutonRelancer;

/// Ressources nécessaires aux icônes et aux textes.
pub struct Contexte<'a> {
    pub serveur: &'a AssetServer,
    pub textures: &'a Textures,
    pub dessins: &'a Dessins,
    pub polices: &'a PolicesMenu,
}

impl Contexte<'_> {
    pub(super) fn ecrit(
        &self,
        c: &mut Commands,
        parent: Entity,
        t: &str,
        s: (Style, f32, Color),
    ) -> Entity {
        ecrit(c, self.polices, parent, t, s)
    }
}

/// Couleur de la famille et opacité de sa lueur (styles.css `.famille-*`).
pub(crate) fn couleur_famille(f: Famille) -> (Color, f32) {
    let (rgb, voile) = match f {
        Famille::Arme => (0xd8dde6, 0.16),
        Famille::Passif => (0x55e055, 0.22),
        Famille::NouvelleArme => (0x4fd8ff, 0.26),
        Famille::Exclusif => (0xff6ad5, 0.3),
        Famille::Eveil => (0xb06cff, 0.36),
        Famille::Soin => (0xffc83a, 0.22),
        Famille::Objet(Rarete::Commune) => (0xc8ccd4, 0.16),
        Famille::Objet(Rarete::Rare) => (0x4f8dff, 0.28),
        Famille::Objet(Rarete::Epique) => (0xb06cff, 0.34),
        Famille::Objet(Rarete::Legendaire) => (0xff9a1f, 0.36),
        Famille::Action => (0x8a93a3, 0.14),
    };
    let c = Color::srgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8);
    (c, voile)
}

/// Construit l'écran ; `relances` : `None` sans bouton (étape de remplacement).
pub fn construire(
    commandes: &mut Commands,
    c: &Contexte,
    titre: &str,
    cartes: &[Carte],
    relances: Option<u32>,
) -> Entity {
    let racine = voile(commandes, (sombre(0.86), sombre(0.96)));
    commandes
        .entity(racine)
        .insert(Name::new("écran de cartes"))
        .entry::<Node>()
        .and_modify(|mut n| {
            n.row_gap = Val::Px(26.0);
            // Au-dessus des cœurs et de la barre d'XP
            n.padding.bottom = Val::Px(40.0);
        });
    nuit::titre(commandes, c.polices, racine, titre, OR_VIF);

    // Présentation à l'essai (BS_CARTES_STYLE) : médaillons par défaut
    let style = std::env::var("BS_CARTES_STYLE").unwrap_or_default();
    let liste = nuit::rangee(commandes, racine, 56.0);
    commandes
        .entity(liste)
        .entry::<Node>()
        .and_modify(move |mut n| {
            n.align_items = AlignItems::Stretch;
            match style.as_str() {
                "inventaire" => n.column_gap = Val::Px(36.0),
                "parchemin" | "verre" => n.column_gap = Val::Px(28.0),
                "bandes" => {
                    n.column_gap = Val::Px(14.0);
                    n.width = Val::Percent(100.0);
                    n.flex_grow = 1.0;
                }
                _ => {}
            }
        });
    let style = std::env::var("BS_CARTES_STYLE").unwrap_or_default();
    let nombre = cartes
        .iter()
        .filter(|c| c.famille != Famille::Action)
        .count();
    let mut refus = None;
    for (k, carte) in cartes.iter().enumerate() {
        // Refus du coffre : action secondaire sous les cartes, pas une carte
        if carte.famille == Famille::Action && carte.etiquette == "Refuser" {
            refus = Some((k, carte));
            continue;
        }
        match style.as_str() {
            "inventaire" => propositions::inventaire(commandes, c, liste, k, carte),
            "parchemin" => propositions::parchemin(commandes, c, liste, (k, nombre), carte),
            "bandes" => propositions::bande(commandes, c, liste, k, carte),
            "verre" => propositions::verre(commandes, c, liste, k, carte),
            _ => construire_carte(commandes, c, liste, k, carte),
        }
    }

    let bas = nuit::rangee(commandes, racine, 28.0);
    if let Some(n) = relances {
        bouton_relancer(commandes, c, bas, n);
    }
    if let Some((k, carte)) = refus {
        let t = format!("{}  ({})", carte.nom, k + 1);
        nuit::lien(
            commandes,
            c.polices,
            bas,
            &t,
            (19.0, IVOIRE),
            BoutonCarte(k),
        );
    }
    racine
}

fn construire_carte(
    commandes: &mut Commands,
    c: &Contexte,
    liste: Entity,
    k: usize,
    carte: &Carte,
) {
    let (famille, lueur) = couleur_famille(carte.famille);
    let bouton = commandes
        .spawn((
            Button,
            BoutonCarte(k),
            Node {
                width: Val::Px(LARGEUR_CARTE),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            ChildOf(liste),
        ))
        .id();
    // Médaillon rond : lueur de la famille, icône, touche en bas
    let medaillon = commandes
        .spawn((
            Node {
                width: Val::Px(MEDAILLON),
                height: Val::Px(MEDAILLON),
                margin: UiRect::bottom(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::MAX,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundGradient::from(RadialGradient::new(
                UiPosition::CENTER,
                RadialGradientShape::ClosestSide,
                vec![
                    ColorStop::new(
                        famille.with_alpha((lueur * 2.2).min(0.85)),
                        Val::Percent(0.0),
                    ),
                    ColorStop::new(famille.with_alpha(lueur * 0.9), Val::Percent(62.0)),
                    ColorStop::new(Color::NONE, Val::Percent(100.0)),
                ],
            )),
            Outline::new(Val::Px(3.0), Val::Px(4.0), famille.with_alpha(0.35)),
            ChildOf(bouton),
        ))
        .id();
    commandes
        .entity(bouton)
        .insert(Halo(medaillon, famille.with_alpha(0.35)));
    let image = match carte.icone {
        Icone::Texture(chemin) => c.textures.obtenir(c.serveur, chemin),
        Icone::Objet(i) => c.dessins.objets.get(i).cloned(),
    };
    if let Some(h) = image {
        commandes.spawn((
            ImageNode::new(h),
            Node {
                width: Val::Px(76.0),
                height: Val::Px(76.0),
                ..default()
            },
            ChildOf(medaillon),
        ));
    }
    let touche = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(-12.0),
                width: Val::Px(30.0),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(sombre(0.95)),
            Outline::new(Val::Px(2.0), Val::ZERO, famille),
            ChildOf(medaillon),
        ))
        .id();
    let chiffre = (b'1' + k as u8) as char;
    let chiffre = chiffre.encode_utf8(&mut [0; 4]).to_owned();
    c.ecrit(commandes, touche, &chiffre, (Style::Logo, 16.0, IVOIRE));
    // Famille (ou étiquette) en surtitre coloré, nom, stat clé, description
    let surtitre = if carte.etiquette != carte.famille.nom() {
        carte.etiquette.to_uppercase()
    } else {
        carte.famille.nom().to_uppercase()
    };
    c.ecrit(commandes, bouton, &surtitre, (Style::Grasse, 13.0, famille));
    let nom = c.ecrit(commandes, bouton, &carte.nom, (Style::Logo, 30.0, IVOIRE));
    commandes
        .entity(nom)
        .insert(TextLayout::new_with_justify(Justify::Center));
    if let Some(cle) = carte.lignes.first() {
        let e = c.ecrit(commandes, bouton, cle, (Style::Grasse, 16.0, VERT_GAIN));
        commandes
            .entity(e)
            .insert(TextLayout::new_with_justify(Justify::Center));
    }
    let d = (Style::Normale, 15.0, DESCRIPTION);
    let d = c.ecrit(commandes, bouton, &carte.description, d);
    commandes
        .entity(d)
        .insert(TextLayout::new_with_justify(Justify::Center));
    // Les autres lignes (la première est la stat clé)
    for l in carte.lignes.iter().skip(1) {
        // « Dégâts : 10 → 15 » : la nouvelle valeur ressort ; à la ligne si trop long
        let ligne = commandes
            .spawn((
                Node {
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(4.0),
                    ..default()
                },
                ChildOf(bouton),
            ))
            .id();
        let (debut, valeur) = match l.rfind("→ ") {
            Some(i) => l.split_at(i + "→ ".len()),
            None => (l.as_str(), ""),
        };
        c.ecrit(commandes, ligne, debut, (Style::Normale, 14.0, STATS));
        if !valeur.is_empty() {
            c.ecrit(commandes, ligne, valeur, (Style::Grasse, 14.0, VERT_GAIN));
        }
    }
}

/// « RELANCER (R) » en pilule, restantes à côté, ou « Plus de relance » éteint.
fn bouton_relancer(commandes: &mut Commands, c: &Contexte, parent: Entity, n: u32) {
    if n == 0 {
        c.ecrit(
            commandes,
            parent,
            "Plus de relance",
            (Style::Grasse, 17.0, nuit::TERNE),
        );
        return;
    }
    let rangee = nuit::rangee(commandes, parent, 12.0);
    let relief = (Relief::SECONDAIRE, false);
    let b = nuit::pilule(
        commandes,
        c.polices,
        rangee,
        "RELANCER (R)",
        relief,
        BoutonRelancer,
    );
    commandes.entity(b).entry::<Node>().and_modify(|mut n| {
        n.height = Val::Px(52.0);
        n.min_width = Val::Px(200.0);
        n.padding = UiRect::horizontal(Val::Px(30.0));
    });
    let pluriel = if n > 1 { "s" } else { "" };
    let t = format!("{n} restante{pluriel}");
    c.ecrit(commandes, rangee, &t, (Style::Grasse, 16.0, AIDE));
}
