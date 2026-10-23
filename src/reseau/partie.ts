/**
 * Partie à plusieurs : ce que jeu.ts appelle aux points d'accroche, côté hôte
 * (il simule tout, voir hote.ts) ou côté joueur (il affiche ce que l'hôte
 * envoie et prédit sa propre marche, voir client.ts).
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import type { Talents } from '../combat/talents/talents';
import type { Entrees } from '../controles/entrees';
import type { ChiffresDegats, StyleChiffre } from '../effets/chiffresDegats';
import type { Ombres } from '../effets/ombres';
import type { Particules } from '../effets/particules';
import type { RenduProjectiles } from '../effets/renduProjectiles';
import type { Retours } from '../effets/retours';
import type { Coffres } from '../entites/coffres';
import type { Ennemis } from '../entites/ennemis';
import type { Joueur } from '../entites/joueur';
import type { Objets } from '../entites/objets';
import type { Orbes } from '../entites/orbes';
import type { ProjectilesEnnemis } from '../entites/projectilesEnnemis';
import type { CibleJoueur } from '../entites/types';
import type { EtatPartie } from '../etat';
import type { Terrain } from '../monde/navigation';
import type { CameraJeu } from '../moteur/camera';
import type { Options } from '../options';
import type { Amelioration } from '../progression/ameliorations';
import { PartieClient } from './client';
import { PartieHote } from './hote';
import type { SessionReseau } from './session';

/** Les systèmes de la partie dont le réseau a besoin (fournis par jeu.ts). */
export interface ElementsPartie {
  readonly etat: EtatPartie;
  readonly terrain: Terrain;
  readonly scene: THREE.Scene;
  readonly camera: THREE.Camera;
  readonly cameraJeu: CameraJeu;
  readonly textures: BibliothequeTextures;
  readonly entrees: Entrees;
  readonly joueur: Joueur;
  /** Ce qui blesse le joueur local (avec le renfort du mode sans fin). */
  readonly cible: CibleJoueur;
  readonly mecaniques: Talents;
  readonly ennemis: Ennemis;
  readonly projectilesEnnemis: ProjectilesEnnemis;
  readonly orbes: Orbes;
  readonly objets: Objets;
  readonly coffres: Coffres;
  readonly particules: Particules;
  readonly chiffres: ChiffresDegats;
  readonly retours: Retours;
  /** Fin de partie décidée : récompenses, puis écran de fin (jeu.ts). */
  conclure(): void;
  /** Victoire continuée en mode sans fin (jeu.ts). */
  continuerSansFin(): void;
}

/** Points d'accroche appelés par jeu.ts. */
export interface CrochetsReseau {
  /** Vrai chez un joueur qui a rejoint : il ne simule ni les ennemis ni les armes. */
  readonly client: boolean;
  /** Multiplicateur du rythme des apparitions (hôte). */
  readonly facteurApparitions: number;
  /**
   * Le joueur local choisit ses cartes : il ne marche plus et ses armes se
   * reposent, mais la partie continue pour les autres (lui est protégé).
   */
  readonly pauseLocale: boolean;
  /** Hôte : marche des joueurs distants, après celle du joueur local. */
  majJoueurs(dt: number): void;
  /** Armes et talents des autres joueurs, après celles du joueur local (chez un joueur qui a rejoint : leurs effets seulement). */
  majArmes(dt: number): void;
  /** Hôte : fin du pas (niveau commun, coffres, réanimations, fin de partie). */
  finPas(dt: number): void;
  /** Joueur : un pas (prédiction de sa marche, envoi de ses entrées). */
  simulerClient(dt: number): void;
  vitesseTemps(cartesOuvertes: boolean): number;
  /** Début de l'image : le joueur applique le dernier instantané reçu et interpole. */
  avantImage(dtReel: number): void;
  majVisuelOmbres(ombres: Ombres): void;
  majVisuelProjectiles(rendu: RenduProjectiles): void;
  /** Fin de l'image : effets des joueurs distants, puis (hôte) envoi des instantanés. */
  majVisuelEffets(dtJeu: number): void;
  /** Chiffre de dégâts affiché chez l'hôte (envoyé aussi aux joueurs proches). */
  surDegats(x: number, y: number, z: number, valeur: number, style: StyleChiffre): void;
  /** Carte choisie par le joueur local (niveau ou coffre). */
  surCarte(amelioration: Amelioration): void;
  /** La partie gagnée continue en mode sans fin (pour tout le monde). */
  surContinuer(): void;
  appliquerOptions(options: Options): void;
}

export function brancherPartie(session: SessionReseau, elements: ElementsPartie): CrochetsReseau {
  return session.role === 'hote' ? new PartieHote(session, elements) : new PartieClient(session, elements);
}
