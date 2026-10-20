/**
 * Types du contenu du jeu. Les tables de données (blocs, modèles, arènes…)
 * sont vérifiées contre ces types à la compilation.
 */
import type { IdBloc } from './blocs';
import type { IdModele } from './modeles';

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
}

export interface MembreMarche {
  readonly partie: string;
  readonly amplitude: number;   // rad à pleine vitesse
  readonly dephasage: number;   // rad
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
  /** Léger balancement permanent (respiration). */
  readonly repos?: {
    readonly amplitude: number;   // rad
    readonly frequenceZ: number;  // rad/s
    readonly frequenceX: number;  // rad/s
    readonly membres: readonly MembreRepos[];
  };
}

// --------------------------------------------------------------------------
// Joueur
// --------------------------------------------------------------------------
export interface DefinitionJoueur {
  readonly rayon: number;            // rayon de collision (blocs)
  readonly vitesse: number;          // blocs / seconde
  readonly reactivite: number;       // rapidité d'atteinte de la vitesse cible (1/s)
  readonly vitesseRotation: number;  // rapidité d'orientation vers la direction de marche (1/s)
  readonly modele: IdModele;
  readonly skin: CheminTexture;
  /** Couleur unie par partie du modèle si le skin est introuvable. */
  readonly couleursSecours: Readonly<Record<string, number>>;
  readonly rayonOmbre: number;       // blocs
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
