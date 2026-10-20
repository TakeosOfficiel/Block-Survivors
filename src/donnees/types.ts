/**
 * Types du contenu du jeu. Les tables de données (blocs, modèles, ennemis…)
 * sont vérifiées contre ces types à la compilation.
 */
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
export interface DefinitionProjectile {
  readonly nom: string;
  readonly texture: CheminTexture;
  readonly tailleTexture: readonly [number, number];
  /** Rectangle [x, y, largeur, hauteur] de la vue de profil dans la texture (pixels). */
  readonly uv: readonly [number, number, number, number];
  readonly longueur: number;   // blocs
  readonly largeur: number;    // blocs
  readonly vitesse: number;    // blocs / seconde
  readonly duree: number;      // s avant disparition
  readonly rayon: number;      // rayon de collision (blocs)
  readonly hauteur: number;    // altitude de vol (blocs)
  readonly couleurSecours: number;
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
