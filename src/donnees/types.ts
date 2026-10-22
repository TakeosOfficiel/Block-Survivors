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
import type { IdGerbe, IdParticule } from './particules';
import type { IdPassif } from './passifs';
import type { IdProjectile } from './projectiles';
import type { IdSon } from './sons';
import type { IdStructure } from './structures';

/** Chemin d'une texture vanilla, relatif à assets/minecraft/textures/, sans « .png » (ex. « block/dirt »). */
export type CheminTexture = string;

/** Skins peintes à partir d'une icône d'item (voir effets/peintures.ts). */
export type IdPeinture = 'fusee';

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
  /** Avec les shaders, ondule au vent : feuillage (tout le bloc frémit) ou plante (le haut se balance). */
  readonly ondule?: 'feuillage' | 'plante';
  /** Teinte de biome multipliée sur le dessus (textures vanilla en niveaux de gris). */
  readonly teinteHaut?: number;
  /** Teinte multipliée sur toutes les faces (feuillage, eau). */
  readonly teinte?: number;
  /** Rotation aléatoire par quart de tour autour de Y, comme le blockstate vanilla. */
  readonly rotationAleatoire?: boolean;
  /** Multiplicateur de vitesse sur ce bloc (sable des âmes) ou dedans (toile). */
  readonly ralentit?: number;
  /** Toile d'araignée : son ralentissement ne gêne pas les araignées. */
  readonly toile?: boolean;
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
  /** Parties vues de face (à leur place, de l'arrière vers l'avant) qui forment le portrait (« tete » par défaut). */
  readonly portrait?: readonly string[];
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
  /** Rôle en quelques mots, sous le nom (menu d'accueil). */
  readonly role: string;
  readonly description: string;
  readonly pv: number;               // 20 = 10 cœurs
  readonly vitesse: number;          // blocs / seconde
  readonly armesDepart: readonly IdArme[];
  /** Attaques de la classe : seules elles sont proposées en carte si l'arsenal est complet. */
  readonly arsenal: readonly IdArme[];
  /**
   * Arsenal encore incomplet (refonte en cours) : la classe peut aussi tirer les
   * armes qui n'appartiennent à aucun arsenal complet.
   */
  readonly arsenalComplet: boolean;
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
  /** Taille du modèle (1 par défaut) ; le rayon et l'ombre sont donnés à cette taille. */
  readonly echelle?: number;
  /** Part du recul ignorée (0 = normal, 1 = inébranlable). */
  readonly resistanceRecul?: number;
  /** Objet toujours lâché à la mort (en plus des tirages au sort). */
  readonly butin?: IdObjet;
  /** Trop grand pour être gêné par le décor : traverse les obstacles. */
  readonly ignoreObstacles?: boolean;
  /** Araignées : pas ralenties par les toiles, comme en jeu. */
  readonly traverseToiles?: boolean;
  /** Présent pour les boss : affichage de leur barre de vie. */
  readonly boss?: { readonly couleurBarre: CouleurBarreBoss };
  /** Mini-boss : arrive à son heure (CALENDRIER_MINIBOSS), barre de vie au-dessus de la tête. */
  readonly miniBoss?: true;
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
/**
 * `croisee` : plans croisés de l'entité (flèches) ; `relief` : objet en relief d'un pixel d'épaisseur,
 * comme les objets du jeu (fusée, hache) ; `modele` : modèle cubique de l'entité (trident) ;
 * `face_camera` : sprite tourné vers la caméra, comme le jeu dessine les boules de feu.
 */
export type FormeProjectile = 'croisee' | 'relief' | 'modele' | 'face_camera';

export interface DefinitionProjectile {
  readonly nom: string;
  readonly forme: FormeProjectile;
  /** Modèle de l'entité (forme `modele`). */
  readonly modele?: IdModele;
  /** Skin du modèle peinte d'après l'image de `texture` (objet sans entité en jeu, ex. la fusée). */
  readonly peinture?: IdPeinture;
  readonly texture: CheminTexture;
  readonly tailleTexture: readonly [number, number];
  /** Rectangle [x, y, largeur, hauteur] affiché, dans la texture (pixels). */
  readonly uv: readonly [number, number, number, number];
  readonly longueur: number;   // blocs (côté de l'icône pour relief / face_camera ; ignoré pour modele)
  readonly largeur: number;    // blocs (épaisseur des plans croisés)
  readonly vitesse: number;    // blocs / seconde (tir des ennemis)
  readonly duree: number;      // s avant disparition (tir des ennemis)
  readonly rayon: number;      // rayon de collision (blocs)
  readonly hauteur: number;    // altitude de vol (blocs)
  /** Rotation sur lui-même (rad/s), pour les haches. */
  readonly rotationPropre?: number;
  /** Particules laissées derrière lui. */
  readonly trainee?: IdGerbe;
  /** Distance (blocs) derrière le centre où naît la traînée (tuyère de la fusée). */
  readonly decalageTrainee?: number;
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
  readonly duree: number;        // s de vie (golems), de rotation (tourbillon), de ralentissement
  readonly perforation: number;  // ennemis traversés avant de disparaître
  readonly recul: number;        // blocs/s donnés aux ennemis touchés (négatif : les attire)
  readonly ralenti: number;      // part de vitesse retirée aux ennemis touchés, pendant `duree` (0 : aucun)
  readonly etourdi: number;      // s d'étourdissement des ennemis touchés (0 : aucun)
}

export type MecaniqueArme =
  /** Frappe en arc de cercle devant le joueur. */
  | { readonly type: 'balayage'; readonly angle: number; readonly effet: readonly CheminTexture[] }
  /** Projectiles en ligne droite vers l'ennemi le plus proche. */
  | { readonly type: 'tir'; readonly projectile: IdProjectile; readonly dispersion: number }
  /** Projectile qui traverse tout, puis revient au joueur. */
  /** Avant le rang `loyaute`, l'arme part en ligne et traverse tout ; ensuite elle revient (enchantement Loyauté). */
  | { readonly type: 'retour'; readonly projectile: IdProjectile; readonly dispersion: number; readonly loyaute: number }
  /** Objets qui tournent en permanence autour du joueur. */
  | { readonly type: 'orbite'; readonly projectile: IdProjectile }
  /** Projectile qui explose au premier ennemi touché. */
  /**
   * Projectile qui explose au premier ennemi touché (`gerbe` et `son` de l'explosion, sinon ceux des
   * boules de feu) ; dès le rang `eclatsAuRang`, l'explosion lance 4 petites explosions autour.
   */
  | {
      readonly type: 'explosif'; readonly projectile: IdProjectile;
      readonly gerbe?: IdGerbe; readonly son?: IdSon; readonly eclatsAuRang?: number;
    }
  /** Éclairs qui frappent des ennemis au hasard autour du joueur. */
  | { readonly type: 'foudre'; readonly couleur: number }
  /** Alliés qui combattent seuls pendant un temps limité. */
  | { readonly type: 'invocation'; readonly allie: IdAllie; readonly brulureAuRang?: number }
  /** Fissures qui soulèvent les blocs du sol en ligne vers le plus gros groupe (`ecart` : rad entre fissures). */
  | { readonly type: 'fissure'; readonly ecart: number; readonly vitesse: number }
  /** Arme géante (`image`, `taille` blocs) qui tombe du ciel sur le plus gros groupe d'ennemis. */
  | { readonly type: 'chute'; readonly image: CheminTexture; readonly taille: number }
  /** Le joueur tourne sur lui-même et frappe tout autour toutes les `intervalle` s. */
  | { readonly type: 'tourbillon'; readonly intervalle: number; readonly effet: readonly CheminTexture[] }
  /** Coup de bouclier en cône : repousse et interrompt ; renvoie les projectiles dès le rang `renvoi`. */
  | { readonly type: 'heurt'; readonly angle: number; readonly renvoi: number }
  /** Colonne de cubes (`texture`) qui avance vers les ennemis et blesse autour d'elle toutes les `intervalle` s. */
  /**
   * Tornade en entonnoir : spirales de `feu` ; `texture` : fragments de magma (Tempête de feu) ;
   * `brulure` : trace laissée au sol.
   */
  | {
      readonly type: 'tornade'; readonly texture: CheminTexture; readonly feu: CheminTexture;
      readonly brulure: CheminTexture; readonly vitesse: number; readonly intervalle: number;
    }
  /**
   * Anneau d'éclats (`texture`) qui s'étend autour du joueur en `expansion` s ; dès le rang `solAuRang`,
   * il laisse un sol (`sol`) qui ralentit pendant `dureeSol` s.
   */
  | {
      readonly type: 'nova'; readonly texture: CheminTexture; readonly expansion: number;
      readonly sol: CheminTexture; readonly solAuRang: number; readonly dureeSol: number;
    }
  /** Bloc (`texture`) qui tombe sur un groupe, puis feu au sol (`feu`) : `degatsSol` × dégâts toutes les 0,5 s. */
  | { readonly type: 'meteore'; readonly texture: CheminTexture; readonly feu: CheminTexture; readonly degatsSol: number }
  /**
   * Orbes (`texture`) qui tournent autour du joueur pendant la durée, puis foncent sur les ennemis ;
   * dès le rang `contactAuRang`, elles blessent au contact (`contact` × dégâts) pendant l'orbite.
   */
  | { readonly type: 'orbes'; readonly texture: CheminTexture; readonly contactAuRang: number; readonly contact: number }
  /**
   * Flèche à tête chercheuse qui rebondit d'ennemi en ennemi (perforation = rebonds) dans un rayon de
   * `distanceRebond` blocs, en perdant `perte` de ses dégâts à chaque rebond.
   */
  | { readonly type: 'ricochet'; readonly projectile: IdProjectile; readonly perte: number; readonly distanceRebond: number }
  /** Flèches (`image`) qui tombent du ciel sur le plus gros groupe, `impactsParSeconde` pendant la durée. */
  | { readonly type: 'pluie'; readonly projectile: IdProjectile; readonly impactsParSeconde: number }
  /**
   * Piège (buissons `image`) posé sous le joueur : blesse et ralentit toutes les `intervalle` s ;
   * dès le rang `enracineAuRang`, il enracine un instant les ennemis qui y entrent.
   */
  | { readonly type: 'piege'; readonly image: CheminTexture; readonly intervalle: number; readonly enracineAuRang: number };

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
  /** Famille qui profite des bonus de talents (« +8 % dégâts de mêlée »…). */
  readonly famille?: FamilleArme;
  /** Statistiques au rang 1. */
  readonly stats: StatsArme;
  /** Améliorations des rangs 2, 3… ; rang maximal = 1 + longueur. */
  readonly niveaux: readonly NiveauArme[];
  /** Son joué à chaque déclenchement. */
  readonly son?: IdSon;
  /** Noms des statistiques sur les cartes, s'ils diffèrent (« Bouclier » au lieu de « Dégâts »). */
  readonly libelles?: Partial<Record<keyof StatsArme, string>>;
  /** Statistiques montrées sur les cartes, si elles diffèrent de celles de sa mécanique. */
  readonly statsAffichees?: readonly (keyof StatsArme)[];
  /** Éveils proposés au rang maximal : deux variantes exclusives. */
  readonly eveils?: readonly [EveilArme, EveilArme];
}

/** Éveil d'une arme au rang maximal : une variante exclusive (l'autre n'est plus proposée). */
export interface EveilArme {
  readonly nom: string;
  readonly description: string;
  readonly icone: CheminTexture;
  /** Ajouts aux statistiques de base (avant les modificateurs). */
  readonly ajout?: Partial<StatsArme>;
  /** Parts en plus (0,2 = +20 %) des statistiques finales. */
  readonly part?: Partial<StatsArme>;
  /** État infligé à chaque coup de l'arme. */
  readonly etat?: EtatEveil;
}

export interface EtatEveil {
  readonly type: 'brulure' | 'poison' | 'ralenti' | 'etourdi';
  /** Dégâts par seconde (brûlure, poison), part de vitesse retirée (ralenti) ou durée (étourdi, s). */
  readonly valeur: number;
  readonly duree: number;
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
  /** Son de chaque coup porté. */
  readonly sonCoup: IdSon;
  readonly sonBlessure: IdSon;
  readonly sonMort: IdSon;
  /** Points de vie : les ennemis au corps à corps l'attaquent, il peut mourir avant la fin. */
  readonly pv: number;
  /** Dégâts retirés à chaque coup reçu (voir defense.ts). */
  readonly armure?: number;
  /** Nombre maximal en même temps ; au-delà, une invocation renouvelle la durée d'un allié présent. */
  readonly quota: number;
  /** Profite des « +1 allié » permanents (Appel de la meute…) ; pas les golems. */
  readonly renforts: boolean;
  /** Distance (blocs) à laquelle il détourne les ennemis du joueur. */
  readonly provocation: number;
  /** Vole à cette hauteur (blocs) en traversant le décor (vex). */
  readonly vol?: number;
  /** Combat à distance : salves de `salve` projectiles écartés de `ecart` rad, à `distance` blocs de la cible. */
  readonly tir?: { readonly projectile: IdProjectile; readonly salve: number; readonly ecart: number; readonly distance: number };
  /** Soutien : ne combat pas, suit le joueur, lui donne du bouclier puis soigne les alliés blessés. */
  readonly soutien?: boolean;
  /**
   * Immobile, posé sur le plus gros groupe ; toutes les `intervalle` s, ralentit et blesse autour de lui,
   * et enracine `enracinement` s les ennemis qui entrent (une fois toutes les `delai` s).
   */
  readonly aura?: { readonly intervalle: number; readonly enracinement: number; readonly delai: number };
  /** Dessiné en blocs empilés (`empilement`, de bas en haut) entourés de `racines`, au lieu de son modèle. */
  readonly blocs?: { readonly empilement: readonly CheminTexture[]; readonly racines: CheminTexture };
  /** Au quota, le plus ancien est replanté sur le nouveau groupe au lieu d'être renouvelé sur place. */
  readonly remplacement?: boolean;
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
  /** Zone jouable en blocs, centrée sur l'origine : d'ouest en est, et du nord au sud. */
  readonly largeur: number;
  readonly profondeur: number;
  readonly hauteurMur: number;
  readonly sol: readonly ChoixBloc[];
  /** Appliquées dans l'ordre, après le sol de base. */
  readonly taches: readonly TacheSol[];
  readonly murs: readonly ChoixBloc[];
  readonly decor: readonly ElementDecor[];
  /** Rayon laissé libre autour du point de départ (blocs). */
  readonly zoneDepart: number;
  /** Terrain décoratif hors arène : dessus du bloc répété, assombri (aucun : null). */
  readonly exterieur: {
    readonly bloc: IdBloc;
    readonly assombrissement: number;
    readonly taille: number;
  } | null;
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
  /** Mini-boss de l'arène, dans l'ordre d'arrivée (puis en boucle). */
  readonly miniBoss: readonly IdEnnemi[];
  /** Population hors zones. */
  readonly ennemis: readonly ChoixEnnemi[];
  /** Zones à population propre (la première qui contient le point d'apparition l'emporte). */
  readonly zones: readonly ZoneEnnemis[];
  /** Point de départ du joueur (centre de l'arène par défaut). */
  readonly depart?: { readonly x: number; readonly z: number };
  /** Arène construite dans Minecraft : schématique (adresse, souvent en data:), lue au lancement. */
  readonly source?: { readonly fichier: string };
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
/** Calendrier des boss (ou des mini-boss) : apparitions à intervalles réguliers, annoncées à l'écran. */
export interface DefinitionCalendrierBoss {
  readonly premierPassage: number;   // s de jeu avant le premier boss
  readonly intervalle: number;       // s entre deux boss
  readonly avertissement: number;    // s d'alerte avant l'apparition
  /** PV multipliés comme ceux des ennemis ordinaires à ce moment de la partie (mini-boss). */
  readonly pvSelonTemps: boolean;
}

// --------------------------------------------------------------------------
// Coffres
// --------------------------------------------------------------------------
/** Coffre posé sur la carte : ouvert au contact, il offre des cartes d'amélioration. */
export interface DefinitionCoffre {
  readonly nom: string;
  /** Titre de l'écran des cartes à l'ouverture. */
  readonly titre: string;
  readonly apparence: ApparenceEntite;
  /** Payé en émeraudes (prix croissant) ; sinon gratuit. */
  readonly payant: boolean;
  /** Cartes choisies à l'ouverture (une à la fois). */
  readonly choix: number;
  readonly son: IdSon;
}

/** Quand et où les coffres apparaissent, et leur prix. */
export interface ReglesCoffres {
  readonly capacite: number;
  /** Un coffre après `premier` ennemis tués, puis tous les `intervalle` (+ `croissance` à chaque fois). */
  readonly tues: { readonly premier: number; readonly intervalle: number; readonly croissance: number };
  /** Prix du premier coffre payant ouvert, puis + `increment` à chacun. */
  readonly prix: { readonly base: number; readonly increment: number };
  /** Distance au joueur (blocs) des coffres qui apparaissent après des ennemis tués. */
  readonly distance: readonly [number, number];
  readonly rayonOuverture: number;     // blocs
  readonly dureeOuverture: number;     // s : le couvercle se lève
  readonly resteOuvert: number;        // s avant de disparaître
}

// --------------------------------------------------------------------------
// Objets ramassables
// --------------------------------------------------------------------------
export interface DefinitionObjet {
  readonly nom: string;
  readonly icone: CheminTexture;
  readonly soin: number;             // PV rendus
  /** Tas d'émeraudes (monnaie de la partie, pour les coffres) : entre [min, max] au hasard, plus gros quand il en vaut plus. */
  readonly emeraudes?: readonly [number, number];
  /** Probabilité d'être lâché par un ennemi ordinaire. */
  readonly chance: number;
  readonly taille: number;           // blocs
  /** Couleur de l'objet posé si son icône manque. */
  readonly couleurSecours: number;
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
  projectiles: number;   // projectiles en plus (armes qui en profitent, voir plafonds.ts)
  allies: number;        // alliés en plus à chaque invocation
  vitesse: number;
  pvMax: number;
  armure: number;        // dégâts retirés à chaque coup reçu (voir defense.ts)
  regeneration: number;  // PV / s
  vies: number;          // totems d'immortalité : résurrections à la mort
  collecte: number;
  critique: number;      // chance de coup critique (0 à 1)
  degatsCritiques: number;  // multiplicateur des dégâts d'un coup critique
  // Talents : bonus par famille d'armes (parts en plus, 0,04 = +4 %)
  degatsMelee: number;
  zoneMelee: number;
  degatsProjectiles: number;
  vitesseProjectiles: number;
  degatsFeu: number;
  cadenceFeu: number;       // ajouté au délai des armes de feu (négatif : plus rapides)
  degatsAllies: number;
  vitesseAllies: number;
  pvAllies: number;
  dureeAllies: number;      // s de vie en plus des alliés invoqués
  recul: number;            // multiplicateur du recul des armes
  // Talents : états et soins
  degatsBrulure: number;    // parts en plus des dégâts de brûlure
  dureeBrulure: number;
  cumulsBrulure: number;    // cumuls en plus au maximum
  degatsPoison: number;
  dureeRalenti: number;     // parts en plus de la durée des ralentissements
  degatsRalentis: number;   // dégâts en plus contre les ennemis ralentis
  regenerationPart: number; // part des PV max rendue par seconde
  soinsObjets: number;      // parts en plus des soins des pommes
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
  /**
   * Objet à usage unique (totem) : sans rangs, chaque carte en donne un de
   * plus tant que la réserve (vies pas encore utilisées) est sous `reserveMax`.
   * N'occupe pas d'emplacement de bonus ; son compteur s'affiche près des cœurs.
   */
  readonly consommable?: { readonly reserveMax: number; readonly libelle: string };
  /** Effets des rangs 1, 2… ; rang maximal = longueur (un consommable n'en a qu'un, répété). */
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
  /** Relances des cartes (nouveau tirage) disponibles par partie. */
  readonly relances: number;
  readonly emplacementsArmes: number;
  readonly emplacementsPassifs: number;
  /** Carte proposée quand il n'y a plus rien à améliorer. */
  readonly soin: { readonly nom: string; readonly description: string; readonly icone: CheminTexture; readonly pv: number };
  /** Image de l'orbe (0 à 15 dans la texture) selon sa valeur, de la plus petite à la plus grosse. */
  readonly orbes: readonly { readonly valeurMin: number; readonly image: number }[];
  /** Mode sans fin (après une victoire) : renforcement des ennemis par minute au-delà de `dureePartie`. */
  readonly sansFin: {
    /** PV en plus (fraction) : ennemis, mini-boss et boss. */
    readonly pvParMinute: number;
    /** Dégâts infligés au joueur en plus (fraction). */
    readonly degatsParMinute: number;
    /** Ennemis en plus par groupe, jusqu'à `groupeMax`. */
    readonly groupeParMinute: number;
    readonly groupeMax: number;
  };
}

// --------------------------------------------------------------------------
// Sons
// --------------------------------------------------------------------------
/** Famille de sons, chacune avec son curseur de volume dans les options. */
export type CategorieSon = 'joueur' | 'armes' | 'creatures' | 'interface';

export interface DefinitionSon {
  readonly categorie: CategorieSon;
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

// --------------------------------------------------------------------------
// Particules
// --------------------------------------------------------------------------
/**
 * Choix de l'image : `age` parcourt la série au fil de la vie de la particule,
 * `fixe` garde la première, `morceau` prend un quart de l'icône donnée à l'émission.
 */
export type AnimationParticule = 'age' | 'fixe' | 'morceau';

export interface DefinitionParticule {
  readonly images: readonly CheminTexture[];
  readonly animation: AnimationParticule;
  readonly taille: readonly [number, number];   // côté (blocs), tiré entre ces bornes
  readonly duree: readonly [number, number];    // s
  /** blocs/s² vers le bas (négatif : la particule monte). */
  readonly gravite: number;
  /** Part de la vitesse perdue par seconde (0 à 1). */
  readonly frottement: number;
  /** Teinte grise tirée entre ces bornes (1 = image telle quelle)… */
  readonly gris?: readonly [number, number];
  /** … ou une de ces couleurs. */
  readonly teintes?: readonly number[];
  /** Rétrécit en fin de vie (flammes, étincelles). */
  readonly retrecit?: boolean;
}

/** État d'un ennemi (etats.ts). */
export interface DefinitionEtat {
  /** Particules émises toutes les `periode` s tant qu'il dure. */
  readonly gerbe: IdGerbe;
  readonly periode: number;
  /** Teinte du modèle (linéaire) et sa force. */
  readonly couleur?: readonly [number, number, number];
  readonly force?: number;
  /** Dégâts par seconde : cumuls au maximum et intervalle entre deux dégâts (s). */
  readonly cumulsMax?: number;
  readonly tic?: number;
}

/** Une salve de particules d'un même type ; une gerbe en enchaîne plusieurs. */
export interface SalveParticules {
  readonly particule: IdParticule;
  readonly nombre: number;
  readonly vitesse: number;     // blocs/s, horizontale, tirée jusqu'à cette valeur
  readonly dispersion: number;  // rayon autour du point d'émission (blocs)
  readonly montee: number;      // vitesse verticale initiale max (blocs/s)
}

export type DefinitionGerbe = readonly SalveParticules[];

// --------------------------------------------------------------------------
// Progression entre les parties (talents, émeraudes, déblocages)
// --------------------------------------------------------------------------
/** Famille d'armes, que certains talents renforcent toutes ensemble. */
export type FamilleArme = 'melee' | 'projectile' | 'feu' | 'allie';

/** Bonus d'un talent pour une arme précise, cumulé à chaque rang. */
export interface BonusArmeTalent {
  readonly arme: IdArme;
  /** Ajouts aux statistiques de base de l'arme (avant les modificateurs). */
  readonly ajout?: Partial<StatsArme>;
  /** Parts en plus (0,1 = +10 %) des statistiques finales. */
  readonly part?: Partial<StatsArme>;
  /** Alliés invoqués par l'arme : PV et vitesse en plus (parts). */
  readonly pv?: number;
  readonly vitesse?: number;
}

/** Ce que fait un talent (ou une option d'un talent à choix), cumulé à chaque rang. */
export interface EffetTalent {
  readonly modif?: Partial<Modificateurs>;
  readonly armes?: readonly BonusArmeTalent[];
  /** Rangs offerts de cette arme au départ (une par rang). */
  readonly rangsArme?: IdArme;
  /** Commence avec cette arme, au rang du talent ; désactivable, une seule à la fois. */
  readonly armeDepart?: IdArme;
  readonly relances?: number;
  /** Effet propre, géré par le combat (voir combat/talents.ts). */
  readonly mecanique?: string;
}

export type TypeNoeud = 'bonus' | 'capacite' | 'choix' | 'final';
export type BrancheArbre = 'tronc' | 'gauche' | 'centre' | 'droite' | 'libre';

/** Une des deux options d'un talent à choix. */
export interface OptionTalent extends EffetTalent {
  readonly nom: string;
  readonly icone: CheminTexture;
  readonly effet: string;
}

/** Un talent de l'arbre, à sa place (ligne 0 en haut, colonnes 0 à 6). */
export interface NoeudTalent extends EffetTalent {
  /** Identifiant stable (sauvegarde). */
  readonly id: string;
  readonly ligne: number;
  readonly colonne: number;
  readonly type: TypeNoeud;
  readonly branche: BrancheArbre;
  readonly nom: string;
  readonly icone: CheminTexture;
  /** Effet affiché (tous rangs, ex. « Dégâts +4 / 8 / 12 % »). */
  readonly effet: string;
  readonly rangMax: number;
  /** Talents reliés au-dessus : il faut qu'un d'eux soit complet. Absent : racine ou talent libre. */
  readonly prerequis?: readonly string[];
  /** Relié à rien : s'ouvre avec son palier. */
  readonly libre?: boolean;
  /** Talent à choix : ses deux options (l'achat débloque les deux). */
  readonly options?: readonly [OptionTalent, OptionTalent];
}

export interface DefinitionArbre {
  readonly id: string;
  readonly nom: string;
  readonly description: string;
  readonly icone: CheminTexture;
  readonly branches: Partial<Record<BrancheArbre, string>>;
  readonly noeuds: readonly NoeudTalent[];
}

/** Maîtrise d'une classe : XP de partie cumulée, niveaux, un point de talent par niveau. */
export interface DefinitionMaitrise {
  readonly icone: CheminTexture;
  /** XP pour le niveau 1. */
  readonly premierNiveau: number;
  /** XP en plus demandée par chaque niveau suivant. */
  readonly increment: number;
}

export type ConditionDeblocage =
  | { readonly type: 'survie'; readonly duree: number; readonly arene?: string }
  | { readonly type: 'boss'; readonly nombre: number };

export interface DefinitionDeblocage {
  readonly type: 'personnage' | 'arene';
  readonly id: string;
  /** Nom annoncé une fois débloqué. */
  readonly nom: string;
  readonly condition: ConditionDeblocage;
  readonly texte: string;
}
