/**
 * Types du contenu du jeu. Les tables de données (blocs, modèles, ennemis…)
 * sont vérifiées contre ces types à la compilation.
 */
import type { IdAllie } from './allies';
import type { IdArme } from './armes';
import type { IdBloc } from './blocs';
import type { IdEnnemi } from './ennemis';
import type { IdModele } from './modeles';
import type { IdProjectile } from './projectiles';

/** Chemin d'une texture vanilla, relatif à assets/minecraft/textures/, sans « .png » (ex. « block/dirt »). */
export type CheminTexture = string;

export type Vec3 = readonly [number, number, number];

// --------------------------------------------------------------------------
// Blocs
// --------------------------------------------------------------------------
export interface DefinitionBloc {
  readonly nom: string;
  readonly faces: {
    readonly haut: CheminTexture;
    readonly cotes: CheminTexture;
    readonly bas: CheminTexture;
  };
  /** Teinte de biome multipliée sur le dessus (textures vanilla en niveaux de gris). */
  readonly teinteHaut?: number;
  /** Rotation aléatoire par quart de tour autour de Y, comme le blockstate vanilla. */
  readonly rotationAleatoire?: boolean;
  /** Couleur unie si les textures n'ont pu être chargées. */
  readonly couleurSecours: number;
}

/** Bloc tiré au hasard (déterministe) parmi une liste pondérée. */
export interface ChoixBloc {
  readonly bloc: IdBloc;
  readonly poids: number;
}

// --------------------------------------------------------------------------
// Modèles cubiques (personnages et créatures)
// --------------------------------------------------------------------------
export interface CubeModele {
  /** Largeur, hauteur, profondeur en pixels de texture. */
  readonly taille: Vec3;
  /** Articulation, en pixels depuis les pieds. */
  readonly pivot: Vec3;
  /** Centre de la boîte par rapport à l'articulation, en pixels. */
  readonly centre: Vec3;
  /** Coin haut-gauche du patron de la boîte dans la texture, en pixels. */
  readonly uv: readonly [number, number];
  /** Patron retourné horizontalement (membre gauche des skins 64x32). */
  readonly miroir?: boolean;
  /** Pose de base (rad), appliquée dans l'ordre Z, Y, X comme en jeu. */
  readonly rotation?: Vec3;
  /** Agrandit la boîte de ce nombre de pixels sur chaque face, sans changer le patron (« inflate » du jeu). */
  readonly gonflement?: number;
}

export type Axe = 'x' | 'y' | 'z';

export interface MembreMarche {
  readonly partie: string;
  readonly axe: Axe;
  /** Rotation maximale (rad) à pleine allure ; le signe donne le sens. */
  readonly amplitude: number;
  readonly dephasage: number;
  /** Multiplicateur de fréquence (1 par défaut). */
  readonly frequence?: number;
  /** |sin| au lieu de cos : mouvement toujours du même côté (pattes qui se lèvent). */
  readonly absolu?: boolean;
}

export interface MembreRepos {
  readonly partie: string;
  readonly signe: 1 | -1;       // 1 = côté droit, -1 = côté gauche
}

export interface DefinitionModele {
  readonly tailleTexture: readonly [number, number];
  readonly parties: Readonly<Record<string, CubeModele>>;
  /** Balancement des membres proportionnel à la vitesse au sol. */
  readonly marche?: {
    readonly frequence: number;         // rad de cycle par bloc parcouru
    readonly vitesseReference: number;  // vitesse (blocs/s) donnant l'amplitude maximale
    readonly lissage: number;           // raideur de la transition marche ↔ arrêt (1/s)
    readonly membres: readonly MembreMarche[];
  };
  /** Léger balancement permanent des bras (respiration). */
  readonly repos?: {
    readonly amplitude: number;   // rad
    readonly frequenceZ: number;  // rad/s
    readonly frequenceX: number;  // rad/s
    readonly membres: readonly MembreRepos[];
  };
  /** Geste d'attaque : les membres listés se lèvent vers l'avant puis retombent. */
  readonly attaque?: {
    readonly duree: number;       // s
    readonly membres: readonly { readonly partie: string; readonly amplitude: number }[];
  };
}

/** Apparence d'une entité : modèle, skin, couleurs de secours, ombre. */
export interface ApparenceEntite {
  readonly modele: IdModele;
  readonly skin: CheminTexture;
  /** Couleur unie par partie du modèle si le skin est introuvable. */
  readonly couleursSecours: Readonly<Record<string, number>>;
  /** Couleur des parties absentes de `couleursSecours`. */
  readonly couleurSecoursParDefaut?: number;
  readonly rayonOmbre: number;     // blocs
}

// --------------------------------------------------------------------------
// Joueur
// --------------------------------------------------------------------------
export interface DefinitionJoueur extends ApparenceEntite {
  readonly pv: number;               // 20 = 10 cœurs
  readonly invulnerabilite: number;  // s d'invulnérabilité après un coup
  readonly rayon: number;            // rayon de collision (blocs)
  readonly vitesse: number;          // blocs / seconde
  readonly reactivite: number;       // rapidité d'atteinte de la vitesse cible (1/s)
  readonly vitesseRotation: number;  // rapidité d'orientation vers la direction de marche (1/s)
  readonly armesDepart: readonly IdArme[];
}

// --------------------------------------------------------------------------
// Ennemis
// --------------------------------------------------------------------------
/** Va au contact et frappe. */
export interface ComportementMelee {
  readonly type: 'melee';
  readonly degats: number;
  readonly portee: number;   // distance de frappe au-delà du contact (blocs)
  readonly cadence: number;  // s entre deux coups
}

/** Garde ses distances et tire des projectiles. */
export interface ComportementArcher {
  readonly type: 'archer';
  readonly projectile: IdProjectile;
  readonly degats: number;
  readonly distanceIdeale: number;
  readonly portee: number;
  readonly cadence: number;
}

/** S'approche, gonfle puis explose (annulé si le joueur s'éloigne). */
export interface ComportementExplosif {
  readonly type: 'explosif';
  readonly distanceAmorcage: number;
  readonly distanceAnnulation: number;
  readonly duree: number;    // s de mèche
  readonly rayon: number;    // rayon de l'explosion (blocs)
  readonly degats: number;   // au centre, décroissant jusqu'au bord
}

export type ComportementEnnemi = ComportementMelee | ComportementArcher | ComportementExplosif;

export interface DefinitionEnnemi extends ApparenceEntite {
  readonly nom: string;
  readonly pv: number;
  readonly vitesse: number;   // blocs / seconde
  readonly rayon: number;     // rayon de collision (blocs)
  readonly comportement: ComportementEnnemi;
  /** Couleur des particules à la mort. */
  readonly couleurParticules: number;
}

// --------------------------------------------------------------------------
// Projectiles
// --------------------------------------------------------------------------
/**
 * Forme d'affichage d'un projectile :
 * - croisee : deux plans croisés le long de la trajectoire (flèches) ;
 * - plat : icône d'objet posée à plat, sa pointe (coin haut-droit) dans le sens de la trajectoire ;
 * - face_camera : icône toujours tournée vers la caméra (boules de feu).
 */
export type FormeProjectile = 'croisee' | 'plat' | 'face_camera';

export interface DefinitionProjectile {
  readonly nom: string;
  readonly forme: FormeProjectile;
  readonly texture: CheminTexture;
  readonly tailleTexture: readonly [number, number];
  /** Rectangle [x, y, largeur, hauteur] affiché, dans la texture (pixels). */
  readonly uv: readonly [number, number, number, number];
  readonly longueur: number;   // blocs (côté de l'icône pour plat / face_camera)
  readonly largeur: number;    // blocs (épaisseur des plans croisés)
  readonly vitesse: number;    // blocs / seconde (tir des ennemis)
  readonly duree: number;      // s avant disparition (tir des ennemis)
  readonly rayon: number;      // rayon de collision (blocs)
  readonly hauteur: number;    // altitude de vol (blocs)
  /** Rotation sur lui-même (rad/s), pour les haches. */
  readonly rotationPropre?: number;
  /** Couleurs des particules laissées derrière lui. */
  readonly trainee?: readonly number[];
  readonly couleurSecours: number;
}

// --------------------------------------------------------------------------
// Armes du joueur
// --------------------------------------------------------------------------
/** Statistiques d'une arme ; chaque mécanique n'utilise que celles qui la concernent. */
export interface StatsArme {
  readonly degats: number;
  readonly cadence: number;      // s entre deux déclenchements (orbite : délai avant de retoucher un ennemi)
  readonly nombre: number;       // projectiles, haches, éclairs ou golems
  readonly portee: number;       // blocs : recherche de cible, distance de vol
  readonly vitesse: number;      // blocs/s (orbite : rad/s)
  readonly zone: number;         // blocs : rayon de balayage, d'orbite, d'explosion, d'impact, d'allonge
  readonly duree: number;        // s de vie (golems)
  readonly perforation: number;  // ennemis traversés avant de disparaître
  readonly recul: number;        // blocs/s donnés aux ennemis touchés
}

export type MecaniqueArme =
  /** Frappe en arc de cercle devant le joueur. */
  | { readonly type: 'balayage'; readonly angle: number; readonly effet: readonly CheminTexture[] }
  /** Projectiles en ligne droite vers l'ennemi le plus proche. */
  | { readonly type: 'tir'; readonly projectile: IdProjectile; readonly dispersion: number }
  /** Projectile qui traverse tout, puis revient au joueur. */
  | { readonly type: 'retour'; readonly projectile: IdProjectile; readonly dispersion: number }
  /** Objets qui tournent en permanence autour du joueur. */
  | { readonly type: 'orbite'; readonly projectile: IdProjectile }
  /** Projectile qui explose au premier ennemi touché. */
  | { readonly type: 'explosif'; readonly projectile: IdProjectile; readonly couleurs: readonly number[] }
  /** Éclairs qui frappent des ennemis au hasard autour du joueur. */
  | { readonly type: 'foudre'; readonly couleur: number }
  /** Alliés qui combattent seuls pendant un temps limité. */
  | { readonly type: 'invocation'; readonly allie: IdAllie };

export interface NiveauArme {
  /** Texte de la carte d'amélioration. */
  readonly description: string;
  /** Ajouts aux statistiques (cumulés rang après rang). */
  readonly modif: Partial<StatsArme>;
}

export interface DefinitionArme {
  readonly nom: string;
  readonly description: string;
  readonly icone: CheminTexture;
  readonly mecanique: MecaniqueArme;
  /** Statistiques au rang 1. */
  readonly stats: StatsArme;
  /** Améliorations des rangs 2, 3… ; rang maximal = 1 + longueur. */
  readonly niveaux: readonly NiveauArme[];
}

// --------------------------------------------------------------------------
// Alliés (invocations)
// --------------------------------------------------------------------------
export interface DefinitionAllie extends ApparenceEntite {
  readonly nom: string;
  readonly vitesse: number;      // blocs/s
  readonly rayon: number;        // blocs
  readonly cadenceCoups: number; // s entre deux coups
  readonly detection: number;    // distance de recherche d'une cible (blocs)
  readonly couleurParticules: number;
}

// --------------------------------------------------------------------------
// Apparitions
// --------------------------------------------------------------------------
/** Valeur interpolée linéairement entre le début et la fin de la montée en difficulté. */
export interface Progression {
  readonly debut: number;
  readonly fin: number;
}

export interface DefinitionApparitions {
  /** Durée (s) pour passer des valeurs « debut » aux valeurs « fin ». */
  readonly dureeMontee: number;
  readonly intervalle: Progression;        // s entre deux vagues d'apparition
  readonly tailleGroupe: Progression;      // ennemis par vague
  readonly maxSimultanes: Progression;
  readonly multiplicateurPv: Progression;
  readonly distanceMin: number;            // distance minimale au joueur (blocs)
  readonly dispersionGroupe: number;       // rayon d'un groupe (blocs)
  readonly ennemis: readonly {
    readonly ennemi: IdEnnemi;
    readonly depuis: number;               // s de jeu avant la première apparition
    readonly poids: number;
  }[];
}

// --------------------------------------------------------------------------
// Arènes
// --------------------------------------------------------------------------
export interface DefinitionArene {
  readonly id: string;
  readonly nom: string;
  /** Côté de la zone jouable en blocs, centrée sur l'origine. */
  readonly taille: number;
  readonly hauteurMur: number;
  readonly sol: readonly ChoixBloc[];
  readonly murs: readonly ChoixBloc[];
  /** Terrain décoratif hors arène : dessus du bloc répété, assombri. */
  readonly exterieur: {
    readonly bloc: IdBloc;
    readonly assombrissement: number;
    readonly taille: number;
  };
  readonly ciel: number;
  readonly brouillard: { readonly debut: number; readonly fin: number };
}

// --------------------------------------------------------------------------
// Boss
// --------------------------------------------------------------------------
export interface DefinitionBoss {
  readonly id: string;
  readonly nom: string;
}
