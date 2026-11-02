//! Accueil (src/ui/ecranMenu.ts, styles.css `#menu`) posé sur la photo de l'arène
//! (`fond`) : logo et bandeau de maîtrise en haut, classes à gauche (`classes`), pseudo
//! et héros 3D au centre avec les boutons des pages, arènes en vignettes à droite
//! (`arenes`), grand bouton « Jouer » vert en bas, liens et version dans les coins.
//! Titres en police du jeu, le reste en Inter (`polices`). Refait en entier à chaque
//! changement de choix (hors de la boucle de jeu).

mod arenes;
mod bas;
mod classes;

use bevy::prelude::*;

use super::{ActionMenu, Icones};
use crate::ecrans::style::{BoutonPage, OR_CLAIR, Teinte};
use crate::hud::police::TextePixel;
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::sauvegarde::Sauvegarde;

pub(super) const FOND: Color = Color::srgb_u8(0x0b, 0x0e, 0x16);
/// Largeur des colonnes de côté, marge au bord de l'écran.
const LARGEUR_COTE: f32 = 300.0;
const MARGE: f32 = 32.0;
const BLANC: Color = Color::WHITE;
const GRIS_CLAIR: Color = Color::srgb_u8(0xd8, 0xd8, 0xd8);
const GRIS_MOYEN: Color = Color::srgb_u8(0xa8, 0xa8, 0xa8);
const TEXTE_BOUTON: Color = Color::srgb_u8(0xe8, 0xe8, 0xe8);
const ORE_TRANSPARENT: Color = Color::srgba(1.0, 1.0, 1.0, 0.0);
const VERRE: Color = Color::srgba(1.0, 1.0, 1.0, 0.08);
const BORD_VERRE: Color = Color::srgba(1.0, 1.0, 1.0, 0.18);
const OMBRE: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);
/// Bouton du menu (`.bouton-menu`) au repos.
const BOUTON: Teinte = Teinte {
    bord: Color::srgba(1.0, 1.0, 1.0, 0.22),
    fond: Color::srgba_u8(12, 14, 20, 153),
};

/// Outils de construction : commandes, polices et icônes.
pub(super) struct Pinceau<'a, 'w, 's, 'i> {
    pub c: &'a mut Commands<'w, 's>,
    pub ic: &'a mut Icones<'i>,
}

impl Pinceau<'_, '_, '_, '_> {
    pub fn noeud(&mut self, parent: Entity, n: Node) -> Entity {
        self.c.spawn((n, ChildOf(parent))).id()
    }

    /// Texte Inter (`gras` : demi-gras), ombré.
    pub fn texte(&mut self, parent: Entity, t: &str, gras: bool, taille: f32, c: Color) -> Entity {
        let ombre = TextShadow {
            offset: Vec2::new(0.0, 2.0),
            color: OMBRE,
        };
        let t = self.ic.polices.texte(t, gras, taille, c);
        self.c.spawn((t, ombre, ChildOf(parent))).id()
    }

    /// Titre de section en police du jeu (`.menu-section`).
    pub fn section(&mut self, parent: Entity, t: &str) {
        let e = self
            .c
            .spawn((TextePixel::nouveau(t, OR_CLAIR, 2.0), ChildOf(parent)))
            .id();
        self.c.entity(e).insert(Node {
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        });
    }

    /// Intitulé en capitales (`.menu-intitule`).
    pub fn intitule(&mut self, parent: Entity, t: &str) {
        let e = self.texte(parent, &t.to_uppercase(), false, 13.0, GRIS_MOYEN);
        self.c.entity(e).insert(Node {
            margin: UiRect::top(Val::Px(12.0)),
            ..default()
        });
    }

    /// Icône d'un sprite du jeu, carrée.
    pub fn icone(&mut self, parent: Entity, chemin: &str, cote: f32) -> Entity {
        let image = self.ic.icone(chemin);
        let n = Node {
            width: Val::Px(cote),
            height: Val::Px(cote),
            ..default()
        };
        self.c
            .spawn((ImageNode::new(image), n, ChildOf(parent)))
            .id()
    }

    /// Case vitrée autour d'une icône (`.menu-classe-icone`, `.case-mc`).
    pub fn case(&mut self, parent: Entity, chemin: &str, cote: f32, icone: f32) -> Entity {
        let c = self.c.spawn((
            Node {
                width: Val::Px(cote),
                height: Val::Px(cote),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(cote * 0.18)),
                ..default()
            },
            BackgroundColor(VERRE),
            BorderColor::all(BORD_VERRE),
            ChildOf(parent),
        ));
        let c = c.id();
        self.icone(c, chemin, icone);
        c
    }

    /// Pastille arrondie : icône facultative et valeur (`.menu-puce`).
    pub fn puce(
        &mut self,
        parent: Entity,
        t: &str,
        icone: Option<&str>,
        fond: Color,
        couleur: Color,
    ) {
        let p = self.noeud(
            parent,
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(3.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
        );
        self.c
            .entity(p)
            .insert((BackgroundColor(fond), BorderColor::all(VERRE)));
        if let Some(i) = icone {
            self.icone(p, i, 16.0);
        }
        self.texte(p, t, true, 14.0, couleur);
    }

    /// Bouton du menu (`.bouton-menu`) : texte demi-gras, bord clair, or au survol.
    pub fn bouton(&mut self, parent: Entity, t: &str, action: ActionMenu) -> Entity {
        let b = self.bouton_nu(parent, action);
        self.texte(b, t, true, 14.0, TEXTE_BOUTON);
        b
    }

    /// Bouton du menu sans contenu.
    pub fn bouton_nu(&mut self, parent: Entity, action: ActionMenu) -> Entity {
        self.c
            .spawn((
                Button,
                BoutonPage,
                BOUTON,
                action,
                Node {
                    min_width: Val::Px(96.0),
                    height: Val::Px(40.0),
                    padding: UiRect::horizontal(Val::Px(14.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BorderColor::all(BOUTON.bord),
                BackgroundColor(BOUTON.fond),
                ChildOf(parent),
            ))
            .id()
    }
}

/// Nœud absolu placé par ses bords.
fn place(gauche: Option<Val>, droite: Option<Val>, haut: Option<Val>, bas: Option<Val>) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: gauche.unwrap_or(Val::Auto),
        right: droite.unwrap_or(Val::Auto),
        top: haut.unwrap_or(Val::Auto),
        bottom: bas.unwrap_or(Val::Auto),
        ..default()
    }
}

/// Colonne absolue de côté (`.menu-gauche`, `.menu-droite`).
fn colonne_cote(p: &mut Pinceau, parent: Entity, a_gauche: bool) -> Entity {
    let bord = Some(Val::Px(MARGE));
    let (g, d) = if a_gauche { (bord, None) } else { (None, bord) };
    p.noeud(
        parent,
        Node {
            width: Val::Px(LARGEUR_COTE),
            flex_direction: FlexDirection::Column,
            ..place(g, d, Some(Val::Px(92.0)), Some(Val::Px(112.0)))
        },
    )
}

/// Construit l'accueil ; renvoie sa racine.
pub fn construire(
    commandes: &mut Commands,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    ic: &mut Icones,
    choix: ChoixPartie,
) -> Entity {
    let racine = commandes
        .spawn((
            Name::new("menu"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            // Transparent : la photo et le héros 3D sont dessous
        ))
        .id();
    let mut p = Pinceau { c: commandes, ic };
    logo(&mut p, racine);
    maitrise(&mut p, dj, sauvegarde, choix, racine);
    let gauche = colonne_cote(&mut p, racine, true);
    classes::construire(&mut p, dj, sauvegarde, choix, gauche);
    centre(&mut p, sauvegarde, racine);
    let droite = colonne_cote(&mut p, racine, false);
    arenes::construire(&mut p, dj, sauvegarde, choix, droite);
    bas::construire(&mut p, dj, sauvegarde, choix, racine);
    racine
}

/// « Block Survivors », Survivors en or (`#menu-titre`).
fn logo(p: &mut Pinceau, racine: Entity) {
    let l = p.noeud(
        racine,
        Node {
            column_gap: Val::Px(20.0),
            ..place(Some(Val::Px(MARGE)), None, Some(Val::Px(18.0)), None)
        },
    );
    p.c.spawn((TextePixel::nouveau("Block", BLANC, 5.0), ChildOf(l)));
    p.c.spawn((TextePixel::nouveau("Survivors", OR_CLAIR, 5.0), ChildOf(l)));
}

/// Bandeau de maîtrise (src/ui/maitrise.ts) : fiole, niveau, points libres, barre d'XP.
fn maitrise(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    racine: Entity,
) {
    let d = dj.d;
    let (id, perso) = d
        .personnages
        .personnages
        .get_index(choix.personnage)
        .expect("classe choisie");
    let prog = &sauvegarde.progression;
    let m = prog.maitrise(id);
    let libres = prog.points_libres(id, prog.arbre_actif(id));
    let b = p.noeud(
        racine,
        Node {
            width: Val::Px(260.0),
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..place(None, Some(Val::Px(18.0)), Some(Val::Px(14.0)), None)
        },
    );
    p.icone(b, &d.talents.maitrise.icone, 32.0);
    let textes = p.noeud(
        b,
        Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        },
    );
    let ligne = p.noeud(
        textes,
        Node {
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
    );
    let nom = format!("{} · Maîtrise {}", perso.nom, m.niveau);
    p.texte(ligne, &nom, false, 15.0, BLANC);
    let s = if libres > 1 { "s" } else { "" };
    let couleur = if libres > 0 { OR_CLAIR } else { GRIS_MOYEN };
    p.texte(ligne, &format!("{libres} point{s}"), false, 15.0, couleur);
    let barre = p.noeud(
        textes,
        Node {
            height: Val::Px(10.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
    );
    p.c.entity(barre).insert((
        BackgroundColor(Color::srgb_u8(0x1a, 0x1a, 0x1a)),
        BorderColor::all(Color::BLACK),
    ));
    let etendue = m.fin_niveau.saturating_sub(m.debut_niveau).max(1) as f32;
    let part = (m.xp.saturating_sub(m.debut_niveau) as f32 / etendue).clamp(0.0, 1.0);
    let vert = |hex: u32, px: f32| {
        ColorStop::new(
            Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8),
            Val::Px(px),
        )
    };
    let rempli = p.noeud(
        barre,
        Node {
            width: Val::Percent(part * 100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    );
    let degrade = LinearGradient::to_bottom(vec![
        vert(0xb6ff6a, 0.0),
        vert(0xb6ff6a, 2.0),
        vert(0x80ff20, 2.0),
        vert(0x80ff20, 4.0),
        vert(0x3d9a12, 4.0),
    ]);
    p.c.entity(rempli).insert(BackgroundGradient::from(degrade));
}

/// Pseudo (clic : page Profil) et dé, place du héros, boutons des pages.
fn centre(p: &mut Pinceau, sauvegarde: &Sauvegarde, racine: Entity) {
    let c = p.noeud(
        racine,
        Node {
            width: Val::Px(440.0),
            margin: UiRect::left(Val::Px(-220.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..place(Some(Val::Percent(50.0)), None, Some(Val::Px(92.0)), None)
        },
    );
    let ligne = p.noeud(
        c,
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        },
    );
    let pseudo = p.bouton_nu(ligne, ActionMenu::OuvrirProfil);
    let invisible = Teinte {
        bord: ORE_TRANSPARENT,
        fond: ORE_TRANSPARENT,
    };
    p.c.entity(pseudo).insert((
        invisible,
        BorderColor::all(invisible.bord),
        BackgroundColor(invisible.fond),
    ));
    p.c.entity(pseudo)
        .entry::<Node>()
        .and_modify(|mut n| n.height = Val::Px(34.0));
    p.texte(pseudo, &sauvegarde.profil.pseudo, true, 22.0, BLANC);
    bas::de(p, ligne);
    // Place du héros (dessiné par la caméra 3D), ombre au sol
    let place_heros = p.noeud(
        c,
        Node {
            width: Val::Px(300.0),
            height: Val::Px(440.0),
            ..default()
        },
    );
    let sol = p.noeud(
        place_heros,
        Node {
            width: Val::Px(220.0),
            height: Val::Px(40.0),
            margin: UiRect::left(Val::Px(-110.0)),
            ..place(Some(Val::Percent(50.0)), None, None, Some(Val::Px(24.0)))
        },
    );
    let ombre = RadialGradient::new(
        UiPosition::CENTER,
        RadialGradientShape::FarthestSide,
        vec![
            ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.55), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(100.0)),
        ],
    );
    p.c.entity(sol).insert(BackgroundGradient::from(ombre));
    let boutons = p.noeud(
        c,
        Node {
            column_gap: Val::Px(8.0),
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        },
    );
    p.bouton(boutons, "Apparence", ActionMenu::OuvrirProfil);
    p.bouton(boutons, "Talents", ActionMenu::Talents);
    p.bouton(boutons, "Collection", ActionMenu::Collection);
    p.bouton(boutons, "Profils", ActionMenu::OuvrirProfils);
}
