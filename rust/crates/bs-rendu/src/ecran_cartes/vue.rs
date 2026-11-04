//! Mise en page de l'écran de cartes, au style des menus : voile de nuit, grand titre
//! doré en police du logo, puis un bandeau de nuit par choix, fondu aux deux bords, sans
//! cadre : la couleur de la famille (ou de la rareté) n'est qu'une lueur derrière l'icône
//! et deux liserés fins. À gauche la touche,
//! l'icône, la famille et le nom ; à droite la stat clé, la description et les autres
//! lignes « avant → après » (nouvelle valeur en vert). Le bandeau survolé glisse vers la
//! droite et son filet passe à l'or. Sous les bandeaux, au-dessus du HUD, deux pilules
//! de même taille : « Relancer (R) · n » et « Laisser les objets » (refus d'un coffre).
//! Tout est reconstruit à chaque ouverture ou relance (jamais pendant le jeu).

use bevy::prelude::*;
use bs_donnees::equipement::Rarete;
use bs_regles::textes::{Carte, Famille, Icone};

use crate::dessins::Dessins;
use crate::ecrans::charte::{IVOIRE, OR_VIF, Relief, VERT_GAIN};
use crate::ecrans::nuit::{self, Style, ecrit, nuit as sombre, voile};
use crate::polices::PolicesMenu;
use crate::textures::Textures;

const DESCRIPTION: Color = Color::srgb_u8(0xc8, 0xcc, 0xd6);
const STATS: Color = Color::srgb_u8(0xcf, 0xe9, 0xc8);
/// Largeur maximale d'un bandeau (fondu compris) et de son contenu.
const LARGEUR_BANDEAU: f32 = 1500.0;
const LARGEUR_CONTENU: f32 = 1040.0;

/// Carte cliquable (index dans la liste).
#[derive(Component)]
pub struct BoutonCarte(pub usize);

/// Filet d'un bandeau (or au survol) et sa couleur au repos.
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
            n.padding.bottom = Val::Px(130.0);
        });
    nuit::titre(commandes, c.polices, racine, titre, OR_VIF);
    let liste = nuit::colonne(commandes, racine, 12.0);
    commandes.entity(liste).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Percent(100.0);
        n.align_items = AlignItems::Center;
    });
    let mut refus = None;
    for (k, carte) in cartes.iter().enumerate() {
        // Refus du coffre : action secondaire sous les cartes, pas une carte
        if carte.famille == Famille::Action && carte.etiquette == "Refuser" {
            refus = Some((k, carte));
            continue;
        }
        bandeau(commandes, c, liste, k, carte);
    }

    // Actions : deux pilules de même taille, au-dessus des cœurs
    let bas = nuit::rangee(commandes, racine, 20.0);
    if let Some(n) = relances {
        bouton_relancer(commandes, c, bas, n);
    }
    if let Some((k, carte)) = refus {
        let t = format!("{}  ({})", carte.nom.to_uppercase(), k + 1);
        let b = nuit::pilule(
            commandes,
            c.polices,
            bas,
            &t,
            (Relief::SECONDAIRE, false),
            BoutonCarte(k),
        );
        petite(commandes, b);
    }
    racine
}

/// Un choix : bandeau fondu aux bords, identité à gauche, effets à droite.
fn bandeau(commandes: &mut Commands, c: &Contexte, liste: Entity, k: usize, carte: &Carte) {
    let (famille, lueur) = couleur_famille(carte.famille);
    let b = commandes
        .spawn((
            Button,
            BoutonCarte(k),
            Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(LARGEUR_BANDEAU),
                min_height: Val::Px(118.0),
                padding: UiRect::vertical(Val::Px(12.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(liste),
        ))
        .id();
    // Bande de nuit fondue aux bords, lueur de la famille derrière l'icône
    let fond = sombre(0.62);
    commandes.entity(b).insert(BackgroundGradient(vec![
        Gradient::Radial(RadialGradient::new(
            UiPosition::new(Vec2::new(-0.5, 0.0), Val::Percent(21.0), Val::ZERO),
            RadialGradientShape::Ellipse(Val::Px(240.0), Val::Px(100.0)),
            vec![
                ColorStop::new(
                    famille.with_alpha((lueur * 1.6).min(0.45)),
                    Val::Percent(0.0),
                ),
                ColorStop::new(Color::NONE, Val::Percent(100.0)),
            ],
        )),
        Gradient::Linear(LinearGradient::to_right(vec![
            ColorStop::new(Color::NONE, Val::Percent(0.0)),
            ColorStop::new(fond, Val::Percent(22.0)),
            ColorStop::new(fond, Val::Percent(78.0)),
            ColorStop::new(Color::NONE, Val::Percent(100.0)),
        ])),
    ]));
    // Liserés fins en haut et en bas, couleur de la famille fondue
    for haut in [true, false] {
        let mut n = Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(15.0),
            right: Val::Percent(15.0),
            height: Val::Px(1.0),
            ..default()
        };
        if haut {
            n.top = Val::ZERO
        } else {
            n.bottom = Val::ZERO
        }
        commandes.spawn((
            n,
            BackgroundGradient::from(LinearGradient::to_right(vec![
                ColorStop::new(Color::NONE, Val::Percent(0.0)),
                ColorStop::new(famille.with_alpha(0.55), Val::Percent(35.0)),
                ColorStop::new(famille.with_alpha(0.2), Val::Percent(75.0)),
                ColorStop::new(Color::NONE, Val::Percent(100.0)),
            ])),
            ChildOf(b),
        ));
    }
    let ligne = nuit::rangee(commandes, b, 24.0);
    commandes.entity(ligne).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(LARGEUR_CONTENU);
        n.max_width = Val::Percent(94.0);
        n.justify_content = JustifyContent::FlexStart;
    });
    let chiffre = (b'1' + k as u8) as char;
    let chiffre = chiffre.encode_utf8(&mut [0; 4]).to_owned();
    let t = c.ecrit(
        commandes,
        ligne,
        &chiffre,
        (Style::Logo, 40.0, IVOIRE.with_alpha(0.22)),
    );
    commandes.entity(t).insert(Node {
        width: Val::Px(28.0),
        flex_shrink: 0.0,
        ..default()
    });
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
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(ligne),
        ));
    }
    // Identité : famille (ou étiquette), nom, filet
    let gauche = nuit::colonne(commandes, ligne, 4.0);
    commandes
        .entity(gauche)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.width = Val::Px(390.0);
            n.flex_shrink = 0.0;
            n.align_items = AlignItems::FlexStart;
        });
    let surtitre = if carte.etiquette != carte.famille.nom() {
        carte.etiquette.to_uppercase()
    } else {
        carte.famille.nom().to_uppercase()
    };
    c.ecrit(commandes, gauche, &surtitre, (Style::Grasse, 13.0, famille));
    c.ecrit(commandes, gauche, &carte.nom, (Style::Logo, 30.0, IVOIRE));
    let filet = commandes
        .spawn((
            Node {
                width: Val::Px(44.0),
                height: Val::Px(2.0),
                margin: UiRect::top(Val::Px(4.0)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(famille),
            Outline::new(Val::Px(2.0), Val::ZERO, famille),
            ChildOf(gauche),
        ))
        .id();
    commandes.entity(b).insert(Halo(filet, famille));
    // Effets : stat clé, description, autres lignes « avant → après »
    let droite = nuit::colonne(commandes, ligne, 5.0);
    commandes
        .entity(droite)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.flex_grow = 1.0;
            n.flex_shrink = 1.0;
            n.min_width = Val::ZERO;
            n.flex_basis = Val::ZERO;
            n.align_items = AlignItems::FlexStart;
        });
    // Stat clé : la nouvelle valeur en vert (lisible même sur un bandeau vert)
    if let Some(cle) = carte.lignes.first() {
        let rangee = nuit::rangee(commandes, droite, 6.0);
        let (debut, valeur) = coupe(cle);
        c.ecrit(commandes, rangee, debut, (Style::Grasse, 18.0, IVOIRE));
        if !valeur.is_empty() {
            c.ecrit(commandes, rangee, valeur, (Style::Grasse, 18.0, VERT_GAIN));
        }
    }
    let d = (Style::Normale, 16.0, DESCRIPTION);
    c.ecrit(commandes, droite, &carte.description, d);
    // Autres lignes côte à côte (à la ligne si trop long), pour garder les bandeaux fins
    let suite = commandes
        .spawn((
            Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(18.0),
                row_gap: Val::Px(2.0),
                ..default()
            },
            ChildOf(droite),
        ))
        .id();
    for l in carte.lignes.iter().skip(1) {
        let rangee = nuit::rangee(commandes, suite, 4.0);
        let (debut, valeur) = coupe(l);
        c.ecrit(commandes, rangee, debut, (Style::Normale, 14.0, STATS));
        if !valeur.is_empty() {
            c.ecrit(commandes, rangee, valeur, (Style::Grasse, 14.0, VERT_GAIN));
        }
    }
}

/// « Dégâts : 10 → 15 » en (« Dégâts : 10 → », « 15 ») ; sans flèche, tout au début.
fn coupe(l: &str) -> (&str, &str) {
    match l.rfind("→ ") {
        Some(i) => l.split_at(i + "→ ".len()),
        None => (l, ""),
    }
}

/// « RELANCER (R) · n » en pilule, ou « Plus de relance » éteint.
fn bouton_relancer(commandes: &mut Commands, c: &Contexte, parent: Entity, n: u32) {
    if n == 0 {
        let t = (Style::Grasse, 17.0, nuit::TERNE);
        c.ecrit(commandes, parent, "Plus de relance", t);
        return;
    }
    let t = format!("RELANCER (R) · {n}");
    let relief = (Relief::SECONDAIRE, false);
    let b = nuit::pilule(commandes, c.polices, parent, &t, relief, BoutonRelancer);
    petite(commandes, b);
}

/// Pilule d'action secondaire : moins haute que celles des menus.
fn petite(commandes: &mut Commands, b: Entity) {
    commandes.entity(b).entry::<Node>().and_modify(|mut n| {
        n.height = Val::Px(50.0);
        n.min_width = Val::Px(220.0);
        n.padding = UiRect::horizontal(Val::Px(28.0));
    });
}
