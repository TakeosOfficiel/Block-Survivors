/**
 * Types du contenu du jeu. Les tables de données (blocs, modèles, ennemis…)
 * sont vérifiées contre ces types à la compilation.
 */
import type { IdAllie } from './allies';
import type { IdArme } from './armes';
import type { IdBloc } from './blocs';
import type { IdEnnemi } from './ennemis';
import type { IdModele } from './modeles';
import type { IdObjet } from './objets';
import type { IdPassif } from './passifs';
import type { IdProjectile } from './projectiles';
import type { IdSon } from './sons';
import type { IdStructure } from './structures';

/** Chemin d'une texture vanilla, relatif à assets/minecraft/textures/, sans « .png » (ex. « block/dirt »). */
export type CheminTexture = string;

export type Vec3 = readonly [number, number, number];

// --------------------------------------------------------------------------
// Blocs
// --------------------------------------------------------------------------
/**
 * Forme d'un bloc :
 * - cube : bloc plein (par défaut) ;
 * - croix : deux plans croisés en diagonale (fleurs, herbes, toiles), jamais solide ;
 * - liquide : cube légèrement abaissé (eau, lave), infranchissable à pied.
 */
export type FormeBloc = 'cube' | 'croix' | 'liquide';

export interface DefinitionBloc {
  readonly nom: string;
  readonly faces: {
    readonly haut: CheminTexture;
    readonly cotes: CheminTexture;
    readonly bas: CheminTexture;
  };
  readonly forme?: FormeBloc;
  /** Bloque le passage (vrai par défaut pour les cubes). */
  readonly solide?: boolean;
  /** Texture à trous (feuilles, toiles, fleurs) : alphaTest. */
  readonly decoupe?: boolean;
  /** Brille dans le noir (glowstone, lave, magma) : non éclairé. */
  readonly lumineux?: boolean;
  /** Teinte de biome multipliée sur le dessus (textures vanilla en niveaux de gris). */
  readonly teinteHaut?: number;
  /** Teinte multipliée sur toutes les faces (feuillage, eau). */
  readonly teinte?: number;
  /** Rotation aléatoire par quart de tour autour de Y, comme le blockstate vanilla. */
  readonly rotationAleatoire?: boolean;
  /** Multiplicateur de vitesse sur ce bloc (sable des âmes) ou dedans (toile). */
  readonly ralentit?: number;
  /** Dégâts infligés au joueur qui marche dessus (magma). */
  readonly degats?: number;
  /** Texture animée vanilla (bande de frames) : vitesse et lecture en aller-retour. */
  readonly animation?: { readonly imagesParSeconde: number; readonly allerRetour?: boolean };
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

/** Où un modèle tient un objet : dans le repère d'une partie (son articulation pour origine). */
export interface MainModele {
  readonly partie: string;
  /** Centre de l'objet, en pixels depuis l'articulation de la partie. */
  readonly position: Vec3;
  /** Rotation de l'objet (rad, ordre X, Y, Z) ; à 0, sa droite pointe vers +Z et son haut vers +Y. */
  readonly rotation: Vec3;
  /** Taille affichée du sprite de 16 pixels, en pixels de modèle. */
  readonly taille: number;
}

export interface DefinitionModele {
  readonly tailleTexture: readonly [number, number];
  readonly parties: Readonly<Record<string, CubeModele>>;
  /** Point d'attache d'un objet tenu (voir ApparenceEntite.objetEnMain). */
  readonly main?: MainModele;
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
  /** Rotation continue autour de l'axe vertical (bâtons du blaze), en rad/s. */
  readonly tournoiement?: readonly { readonly partie: string; readonly vitesse: number }[];
  /** Le modèle flotte (blaze) ou bondit (cube de magma) au-dessus du sol. */
  readonly flottement?: {
    readonly hauteur: number;     // blocs au-dessus du sol au repos
    readonly amplitude: number;   // blocs
    readonly frequence: number;   // rad/s
    readonly bonds?: boolean;     // |sin| : retombe au sol à chaque bond
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
  /** Objet tenu en main (sprite d'objet vanilla), si le modèle a une main. */
  readonly objetEnMain?: CheminTexture;
}

// --------------------------------------------------------------------------
// Joueur et personnages
// --------------------------------------------------------------------------
/** Réglages communs à tous les personnages. */
export interface DefinitionJoueur {
  readonly invulnerabilite: number;  // s d'invulnérabilité après un coup
  readonly rayon: number;            // rayon de collision (blocs)
  readonly reactivite: number;       // rapidité d'atteinte de la vitesse cible (1/s)
  readonly vitesseRotation: number;  // rapidité d'orientation vers la direction de marche (1/s)
  readonly collecte: number;         // rayon d'attraction des orbes d'XP (blocs)
  readonly rayonRamassage: number;   // distance de ramassage des objets (blocs)
}

/** Personnage jouable : apparence, statistiques, arme de départ, cartes exclusives. */
export interface DefinitionPersonnage extends ApparenceEntite {
  readonly nom: string;
  readonly description: string;
  readonly pv: number;               // 20 = 10 cœurs
  readonly vitesse: number;          // blocs / seconde
  readonly armesDepart: readonly IdArme[];
  /** Bonus permanents, ajoutés aux modificateurs dès le début. */
  readonly bonus: Partial<Modificateurs>;
  /** Bonus passifs exclusifs que seul ce personnage peut tirer. */
  readonly cartesExclusives: readonly IdPassif[];
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
  /** Flèches par salve (1 par défaut), écartées de `dispersion` rad. */
  readonly nombre?: number;
  readonly dispersion?: number;
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
  /** Expérience lâchée à la mort. */
  readonly xp: number;
  /** Couleur des particules à la mort. */
  readonly couleurParticules: number;
  /** Taille du modèle (1 par défaut) ; le rayon et l'ombre sont donnés à cette taille. */
  readonly echelle?: number;
  /** Part du recul ignorée (0 = normal, 1 = inébranlable). */
  readonly resistanceRecul?: number;
  /** Objet toujours lâché à la mort (en plus des tirages au sort). */
  readonly butin?: IdObjet;
  /** Trop grand pour être gêné par le décor : traverse les obstacles. */
  readonly ignoreObstacles?: boolean;
  /** Présent pour les boss : affichage de leur barre de vie. */
  readonly boss?: { readonly couleurBarre: CouleurBarreBoss };
  /** Sons joués quand il est blessé et quand il meurt. */
  readonly sons?: { readonly blessure: IdSon; readonly mort: IdSon };
  /** Hauteur de ses sons (1 = normale ; < 1 = plus grave, pour les boss). */
  readonly hauteurSons?: number;
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
  /** Son joué à chaque déclenchement. */
  readonly son?: IdSon;
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
}

/** Ennemi pouvant apparaître, à partir d'un moment de la partie, avec un poids de tirage. */
export interface ChoixEnnemi {
  readonly ennemi: IdEnnemi;
  readonly depuis: number;   // s de jeu avant la première apparition
  readonly poids: number;
}

// --------------------------------------------------------------------------
// Arènes
// --------------------------------------------------------------------------
/** Rectangle [x0, z0, x1, z1] en blocs, dans les coordonnées de l'arène (origine au centre). */
export type Rectangle = readonly [number, number, number, number];

/** Remplacement du sol : un rectangle, ou des taches rondes placées au hasard. */
export type TacheSol =
  | { readonly type: 'rectangle'; readonly bloc: IdBloc; readonly rect: Rectangle }
  | {
    readonly type: 'taches';
    readonly bloc: IdBloc;
    readonly nombre: number;
    readonly rayon: readonly [number, number];   // min, max (blocs)
    readonly zone?: Rectangle;
  };

/** Décor : une structure à un endroit précis, ou dispersée au hasard. */
export type ElementDecor =
  | { readonly type: 'structure'; readonly structure: IdStructure; readonly x: number; readonly z: number }
  | {
    readonly type: 'dispersion';
    readonly structure: IdStructure;
    readonly nombre: number;
    readonly espacement: number;  // blocs libres exigés autour
    readonly zone?: Rectangle;
  };

/** Zone de l'arène avec sa propre population d'ennemis. */
export interface ZoneEnnemis {
  readonly nom: string;
  readonly rect: Rectangle;
  readonly ennemis: readonly ChoixEnnemi[];
}

/** Couleurs des barres de boss vanilla (gui/sprites/boss_bar). */
export type CouleurBarreBoss = 'pink' | 'blue' | 'red' | 'green' | 'yellow' | 'purple' | 'white';

export interface DefinitionArene {
  readonly id: string;
  readonly nom: string;
  readonly description: string;
  /** Texture de bloc représentant l'arène dans le menu. */
  readonly icone: CheminTexture;
  /** Côté de la zone jouable en blocs, centrée sur l'origine. */
  readonly taille: number;
  readonly hauteurMur: number;
  readonly sol: readonly ChoixBloc[];
  /** Appliquées dans l'ordre, après le sol de base. */
  readonly taches: readonly TacheSol[];
  readonly murs: readonly ChoixBloc[];
  readonly decor: readonly ElementDecor[];
  /** Rayon laissé libre autour du point de départ (blocs). */
  readonly zoneDepart: number;
  /** Terrain décoratif hors arène : dessus du bloc répété, assombri. */
  readonly exterieur: {
    readonly bloc: IdBloc;
    readonly assombrissement: number;
    readonly taille: number;
  };
  readonly ciel: number;
  readonly brouillard: { readonly debut: number; readonly fin: number };
  readonly lumiere: {
    readonly ambiante: number;
    readonly soleil: number;
    readonly couleurAmbiante: number;
    readonly couleurSoleil: number;
  };
  /** Boss de l'arène, dans l'ordre d'arrivée (puis en boucle). */
  readonly boss: readonly IdEnnemi[];
  /** Population hors zones. */
  readonly ennemis: readonly ChoixEnnemi[];
  /** Zones à population propre (la première qui contient le point d'apparition l'emporte). */
  readonly zones: readonly ZoneEnnemis[];
}

// --------------------------------------------------------------------------
// Structures (décor construit en blocs)
// --------------------------------------------------------------------------
export interface DefinitionStructure {
  /**
   * Couches de bas en haut (couche 0 posée sur le sol). Chaque couche est une
   * liste de lignes du nord au sud, chaque caractère un bloc d'ouest en est.
   * « . » = vide ; les autres caractères renvoient à `legende`.
   */
  readonly couches: readonly (readonly string[])[];
  readonly legende: Readonly<Record<string, IdBloc>>;
}

// --------------------------------------------------------------------------
// Boss
// --------------------------------------------------------------------------
/** Calendrier des boss : apparitions à intervalles réguliers, annoncées à l'écran. */
export interface DefinitionCalendrierBoss {
  readonly premierPassage: number;   // s de jeu avant le premier boss
  readonly intervalle: number;       // s entre deux boss
  readonly avertissement: number;    // s d'alerte avant l'apparition
}

// --------------------------------------------------------------------------
// Objets ramassables
// --------------------------------------------------------------------------
export interface DefinitionObjet {
  readonly nom: string;
  readonly icone: CheminTexture;
  readonly soin: number;             // PV rendus
  /** Probabilité d'être lâché par un ennemi ordinaire. */
  readonly chance: number;
  readonly taille: number;           // blocs
  readonly couleurParticules: readonly number[];
}

// --------------------------------------------------------------------------
// Bonus passifs et progression
// --------------------------------------------------------------------------
/**
 * Modificateurs globaux du joueur, cumulés depuis ses bonus passifs.
 * Multiplicateurs (neutre = 1) : degats, cadence, zone, portee, vitesse, collecte.
 * Ajouts (neutre = 0) : projectiles, pvMax, armure, regeneration, vies.
 */
export interface Modificateurs {
  degats: number;
  cadence: number;       // < 1 : armes plus rapides
  zone: number;
  portee: number;
  projectiles: number;   // projectiles, haches, éclairs, golems en plus
  vitesse: number;
  pvMax: number;
  armure: number;        // dégâts retirés à chaque coup reçu (minimum 1 dégât)
  regeneration: number;  // PV / s
  vies: number;          // totems d'immortalité : résurrections à la mort
  collecte: number;
}

export interface NiveauPassif {
  readonly description: string;
  /** Ajouts aux modificateurs (cumulés rang après rang). */
  readonly modif: Partial<Modificateurs>;
}

export interface DefinitionPassif {
  readonly nom: string;
  readonly description: string;
  readonly icone: CheminTexture;
  /** Réservé aux personnages qui le listent dans `cartesExclusives`. */
  readonly exclusif?: boolean;
  /** Effets des rangs 1, 2… ; rang maximal = longueur. */
  readonly niveaux: readonly NiveauPassif[];
}

export interface DefinitionProgression {
  /** XP pour passer du niveau 1 au niveau 2. */
  readonly xpPremierNiveau: number;
  /** Durée d'une partie (s) : survivre jusque-là, c'est gagner. */
  readonly dureePartie: number;
  /** XP supplémentaire demandée à chaque niveau, par tranche de niveaux. */
  readonly paliers: readonly { readonly jusquAuNiveau: number; readonly increment: number }[];
  readonly cartesParNiveau: number;
  readonly emplacementsArmes: number;
  readonly emplacementsPassifs: number;
  /** Carte proposée quand il n'y a plus rien à améliorer. */
  readonly soin: { readonly nom: string; readonly description: string; readonly icone: CheminTexture; readonly pv: number };
  /** Image de l'orbe (0 à 15 dans la texture) selon sa valeur, de la plus petite à la plus grosse. */
  readonly orbes: readonly { readonly valeurMin: number; readonly image: number }[];
}

// --------------------------------------------------------------------------
// Sons
// --------------------------------------------------------------------------
export interface DefinitionSon {
  /** Chemin vanilla relatif à assets/minecraft/sounds/, sans « .ogg ». */
  readonly chemin: string;
  /** Empreinte SHA-1 du fichier dans l'index d'assets (serveur de Mojang). */
  readonly hachage: string;
  readonly volume: number;
  /** Hauteur tirée au hasard entre ces bornes à chaque lecture (comme en jeu). */
  readonly hauteur: readonly [number, number];
  /** Délai minimal entre deux lectures (s), pour ne pas saturer. */
  readonly intervalleMin: number;
  /** Lectures simultanées au maximum. */
  readonly simultanes: number;
}
