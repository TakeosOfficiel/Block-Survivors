/**
 * Ennemis : pool unique, déplacement générique (comportement + recul +
 * séparation), dégâts, états (brûlure, poison, ralenti…), mort et explosion.
 * Rendu : un ModeleInstancie par type.
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { GERBES } from '../donnees/particules';
import { ENNEMIS, IDS_ENNEMIS, type IdEnnemi } from '../donnees/ennemis';
import { ETATS, RESISTANCE_BOSS } from '../donnees/etats';
import { JOUEUR_BASE } from '../donnees/joueur';
import { MODELES } from '../donnees/modeles';
import type { BarresVie } from '../effets/barresVie';
import type { Ombres } from '../effets/ombres';
import type { Retours } from '../effets/retours';
import type { Particules } from '../effets/particules';
import type { EtatPartie } from '../etat';
import { angleChute, avancerAnimation, calculerRotations, hauteurFlottement, reinitialiserAnimation } from '../modeles/animation';
import { compilerModele, type ModeleCompile } from '../modeles/compilation';
import { creerPoseModele, ModeleInstancie, type PoseModele } from '../modeles/rendu';
import { GrilleSpatiale } from '../utils/grilleSpatiale';
import { borner, ecartAngle, facteurLissage, longueur } from '../utils/maths';
import { Pool } from '../utils/pool';
import type { Point } from '../monde/carte';
import type { Terrain } from '../monde/navigation';
import { appliquerComportement, type ContexteComportement, type Vers } from './comportements';
import { creerEnnemi, effacerEtats, type DegatsPeriodiques, type Ennemi } from './ennemi';
import type { Butin } from './butin';
import type { CiblesAllies, EquipeJoueurs, SourceEntites } from './types';
import type { ArmeEquipee } from '../combat/armeEquipee';
import type { Allie } from '../combat/invocations';
import { PLAFONDS } from '../donnees/plafonds';

/**
 * Origine d'un coup : arme du joueur, allié invoqué (profite de la marque),
 * ou dégâts d'un état (jamais critiques, sans recul ni son de blessure).
 */
export type NatureCoup = 'arme' | 'allie' | 'brulure' | 'poison' | 'saignement';

/**
 * Talents branchés sur les coups portés aux ennemis (combat/talents) : bonus
 * de dégâts et de critiques avant le coup, réactions après, à la mort et au
 * choc contre un mur.
 */
export interface CrochetsTalents {
  multiplicateur(e: Ennemi, arme: number, nature: NatureCoup): number;
  bonusCritique(e: Ennemi, arme: number, nature: NatureCoup): number;
  /** Multiplicateur final d'un critique, à partir de celui des passifs. */
  facteurCritique(e: Ennemi, arme: number, nature: NatureCoup, base: number): number;
  apresCoup(e: Ennemi, degats: number, arme: number, critique: boolean, nature: NatureCoup): void;
  surMort(e: Ennemi, arme: number, nature: NatureCoup): void;
  surImpactMur(e: Ennemi): void;
  surSortiePiege(e: Ennemi): void;
}

/** Tout ce qu'il faut pour afficher un type d'ennemi. */
interface RenduType {
  readonly modele: ModeleCompile;
  readonly instances: ModeleInstancie;
  readonly pose: PoseModele;
}

const RAYON_MAX = Math.max(...IDS_ENNEMIS.map((id) => ENNEMIS[id].rayon));
/** s entre deux vérifications de la ligne de vue d'un ennemi. */
const PERIODE_VUE = 0.25;

// Temporaires réutilisés
const vers: Vers = { dx: 0, dz: 0, distance: 0, ligneDeTir: true, cheminX: 0, cheminZ: 0 };
const chemin: Point = { x: 0, z: 0 };
const corrige: Point = { x: 0, z: 0 };

export class Ennemis implements SourceEntites {
  private readonly etat: EtatPartie;
  private readonly terrain: Terrain;
  private readonly contexte: ContexteComportement;
  private readonly particules: Particules;
  private readonly butin: Butin;
  private readonly retours: Retours;
  /** Public pour le multijoueur : l'hôte le lit, le client le remplit d'après l'hôte. */
  readonly pool = new Pool<Ennemi>(CONFIG.ennemis.capacite, creerEnnemi);
  private readonly grille: GrilleSpatiale;
  private readonly voisins = new Int32Array(64);
  private readonly groupe = new Int32Array(64);
  /** Rendu par type ; seuls ceux de l'arène existent au départ (les autres à la première apparition). */
  private readonly rendus: (RenduType | null)[];
  private readonly scene: THREE.Scene;
  private readonly textures: BibliothequeTextures;
  /** Hauteur d'affichage des dégâts, par type (au-dessus de la tête). */
  private readonly hauteurs: Float32Array;
  private prochainId = 1;
  /** Alliés invoqués (branchés après coup : ils dépendent du contexte de combat). */
  private allies: CiblesAllies | null = null;
  /** Allié visé par l'ennemi en cours de mise à jour, et contexte qui le blesse au lieu du joueur. */
  private allieVise: Allie | null = null;
  private contexteAllie: ContexteComportement | null = null;
  private talents: CrochetsTalents | null = null;
  /** Multijoueur (hôte) : chaque ennemi vise le joueur vivant le plus proche. */
  equipe: EquipeJoueurs | null = null;
  /** Multijoueur (hôte) : ennemis, et boss ou mini-boss, plus robustes selon le nombre de joueurs. */
  multiplicateurPv = 1;
  multiplicateurPvBoss = 1;
  /** Multijoueur (joueur qui a rejoint) : les coups ne blessent pas ici, seul l'hôte en décide. */
  coupsActifs = true;

  constructor(
    etat: EtatPartie,
    terrain: Terrain,
    contexte: ContexteComportement,
    particules: Particules,
    butin: Butin,
    retours: Retours,
    scene: THREE.Scene,
    textures: BibliothequeTextures,
  ) {
    this.retours = retours;
    this.etat = etat;
    this.terrain = terrain;
    this.contexte = contexte;
    this.particules = particules;
    this.butin = butin;
    this.grille = new GrilleSpatiale(Math.max(etat.arene.largeur, etat.arene.profondeur) / 2 + 1, CONFIG.ennemis.celluleGrille, CONFIG.ennemis.capacite);
    this.scene = scene;
    this.textures = textures;
    // Seuls les mobs, mini-boss et boss de l'arène sont préparés (moins de shaders à compiler au chargement)
    const a = etat.arene;
    const presents = new Set<IdEnnemi>([...a.ennemis, ...a.zones.flatMap((z) => z.ennemis)].map((e) => e.ennemi));
    for (const id of a.boss) presents.add(id);
    for (const id of a.miniBoss) presents.add(id);
    this.rendus = IDS_ENNEMIS.map(() => null);
    for (const id of presents) this.rendu(IDS_ENNEMIS.indexOf(id));
    this.hauteurs = Float32Array.from(IDS_ENNEMIS, (id) => compilerModele(MODELES[ENNEMIS[id].modele]).hauteur * (ENNEMIS[id].echelle ?? 1) + 0.3);
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  /** Les ennemis au corps à corps pourront s'en prendre aux alliés invoqués. */
  definirAllies(allies: CiblesAllies): void {
    this.allies = allies;
    this.contexteAllie = {
      cible: {
        subirDegats: (degats: number) => {
          if (this.allieVise) allies.blesser(this.allieVise, degats);
          return true;
        },
      },
      tireur: this.contexte.tireur,
    };
  }

  definirTalents(talents: CrochetsTalents): void {
    this.talents = talents;
  }

  /** Rendu du type (créé à la première demande). */
  private rendu(indexType: number): RenduType {
    const existant = this.rendus[indexType];
    if (existant) return existant;
    const def = ENNEMIS[IDS_ENNEMIS[indexType]];
    const modele = compilerModele(MODELES[def.modele]);
    const rendu: RenduType = {
      modele,
      instances: new ModeleInstancie(modele, def, this.textures, CONFIG.ennemis.capacite, this.scene),
      pose: creerPoseModele(modele),
    };
    this.rendus[indexType] = rendu;
    return rendu;
  }

  /** Fait apparaître un ennemi ; renvoie faux si le pool est plein. */
  apparaitre(id: IdEnnemi, x: number, z: number, multiplicateurPv = 1): boolean {
    const e = this.pool.acquerir();
    if (!e) return false;
    const def = ENNEMIS[id];
    const limiteX = this.etat.arene.largeur / 2 - def.rayon;
    const limiteZ = this.etat.arene.profondeur / 2 - def.rayon;
    e.id = this.prochainId++;
    e.indexType = IDS_ENNEMIS.indexOf(id);
    e.definition = def;
    e.x = borner(x, -limiteX, limiteX);
    e.z = borner(z, -limiteZ, limiteZ);
    e.vx = 0;
    e.vz = 0;
    e.dirX = 0;
    e.dirZ = 0;
    e.angle = Math.atan2(this.etat.joueur.x - e.x, this.etat.joueur.z - e.z);
    e.vitesseSol = 0;
    e.pvMax = Math.round(def.pv * multiplicateurPv * (def.boss || def.miniBoss ? this.multiplicateurPvBoss : this.multiplicateurPv));
    e.pv = e.pvMax;
    e.minuterie = 0.5;
    e.blessure = 0;
    e.meche = 0;
    e.mort = -1;
    e.aExploser = false;
    e.voitJoueur = true;
    e.ligneDeTir = true;
    e.cibleAllie = null;
    e.minuterieVue = Math.random() * PERIODE_VUE;  // étale les vérifications
    e.immunites.fill(0);
    effacerEtats(e);
    reinitialiserAnimation(e.animation);
    return true;
  }

  /** Ennemi actif à un index renvoyé par chercher(). */
  obtenir(index: number): Ennemi {
    return this.pool.elements[index];
  }

  /**
   * Écrit dans `sortie` les index des ennemis vivants qui touchent le cercle
   * (leur propre rayon compris) ; renvoie leur nombre.
   */
  chercher(x: number, z: number, rayon: number, sortie: Int32Array): number {
    const candidats = this.grille.requeter(x, z, rayon + RAYON_MAX, this.voisins);
    let n = 0;
    for (let k = 0; k < candidats && n < sortie.length; k++) {
      const i = this.voisins[k];
      const e = this.pool.elements[i];
      const dx = e.x - x;
      const dz = e.z - z;
      const portee = rayon + e.definition.rayon;
      if (e.mort < 0 && dx * dx + dz * dz <= portee * portee) sortie[n++] = i;
    }
    return n;
  }

  /** Premier boss vivant (pour sa barre de vie), ou null. */
  bossActif(): Ennemi | null {
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      if (e.definition.boss && e.mort < 0) return e;
    }
    return null;
  }

  /** Ennemi vivant le plus proche à moins de `portee`, ou null. */
  plusProche(x: number, z: number, portee: number): Ennemi | null {
    let meilleur: Ennemi | null = null;
    let meilleureD2 = portee * portee;
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      if (e.mort >= 0) continue;
      const d2 = (e.x - x) ** 2 + (e.z - z) ** 2;
      if (d2 < meilleureD2) {
        meilleureD2 = d2;
        meilleur = e;
      }
    }
    return meilleur;
  }

  /** Ennemi vivant tiré au hasard à moins de `portee`, ou null (tirage uniforme en un passage). */
  auHasard(x: number, z: number, portee: number): Ennemi | null {
    let choisi: Ennemi | null = null;
    let vus = 0;
    const p2 = portee * portee;
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      if (e.mort >= 0 || (e.x - x) ** 2 + (e.z - z) ** 2 > p2) continue;
      if (Math.random() * ++vus < 1) choisi = e;
    }
    return choisi;
  }

  /** Blesse l'ennemi si l'arme `arme` ne l'a pas touché depuis moins de `delai` secondes. */
  blesserAvecDelai(e: Ennemi, arme: number, delai: number, degats: number, sourceX: number, sourceZ: number, recul: number): boolean {
    if (e.mort >= 0 || e.immunites[arme] > this.etat.tempsJeu) return false;
    e.immunites[arme] = this.etat.tempsJeu + delai;
    this.blesser(e, degats, sourceX, sourceZ, recul, arme);
    return true;
  }

  /**
   * Blesse l'ennemi et le repousse depuis (sourceX, sourceZ). `arme` (index
   * dans IDS_ARMES, −1 hors armes) crédite les dégâts dans les statistiques.
   */
  blesser(e: Ennemi, degats: number, sourceX: number, sourceZ: number, recul: number, arme = -1, nature: NatureCoup = 'arme'): void {
    if (e.mort >= 0 || !this.coupsActifs) return;
    const stats = this.etat.stats;
    const periodique = nature === 'brulure' || nature === 'poison' || nature === 'saignement';
    if (nature === 'allie' && e.marqueTemps > 0) degats *= 1 + e.marque;
    const mods = this.etat.modificateurs;
    const talents = this.talents;
    if (e.ralenti > 0 && !periodique) degats *= 1 + mods.degatsRalentis;
    // Équipement : bonus contre la famille de l'ennemi
    const famille = e.definition.famille;
    if (famille === 'mort_vivant') degats *= 1 + mods.degatsMortsVivants;
    else if (famille === 'arthropode') degats *= 1 + mods.degatsArthropodes;
    else if (famille === 'nether') degats *= 1 + mods.degatsNether;
    if (talents) degats *= talents.multiplicateur(e, arme, nature);
    // Coup critique (armes du joueur) : dégâts multipliés et étincelles du jeu
    const chance = mods.critique + (talents && arme >= 0 && !periodique ? talents.bonusCritique(e, arme, nature) : 0);
    const critique = arme >= 0 && !periodique && Math.random() < Math.min(PLAFONDS.critique, chance);
    if (critique) {
      degats *= talents ? talents.facteurCritique(e, arme, nature, mods.degatsCritiques) : mods.degatsCritiques;
      this.particules.emettre(e.x, 1, e.z, GERBES.critique);
    }
    if (arme >= 0) {
      const style = nature === 'brulure' ? 'feu' : nature === 'poison' ? 'poison' : critique ? 'critique' : 'normal';
      this.retours.degats(e.x, this.hauteurs[e.indexType], e.z, degats, style);
    }
    if (arme >= 0) {
      stats.degatsParArme[arme] += Math.min(degats, e.pv);
      stats.coupsParArme[arme]++;
      if (critique) stats.critiquesParArme[arme]++;
      if (degats > stats.plusGrosCoupParArme[arme]) stats.plusGrosCoupParArme[arme] = degats;
    }
    e.pv -= degats;
    // Les dégâts d'un état ne font pas rougir : on garde la teinte de l'état
    if (!periodique) e.blessure = CONFIG.surcouche.blessure.duree;
    const dx = e.x - sourceX;
    const dz = e.z - sourceZ;
    const d = longueur(dx, dz) || 1;
    const k = recul * (1 - (e.definition.resistanceRecul ?? 0));
    e.vx += (dx / d) * k;
    e.vz += (dz / d) * k;
    const def = e.definition;
    if (e.pv <= 0) {
      e.mort = 0;
      e.meche = 0;
      this.etat.ennemisTues++;
      stats.tuesParType[e.indexType]++;
      if (arme >= 0) stats.tuesParArme[arme]++;
      if (def.boss) {
        stats.bossVaincus++;
        this.retours.secousse(0.7);
      }
      this.butin.lacher(def, e.x, e.z);
      if (def.sons) this.retours.son(def.sons.mort, def.hauteurSons);
      talents?.surMort(e, arme, nature);
    } else if (def.sons && !periodique) {
      this.retours.son(def.sons.blessure, def.hauteurSons);
    }
    // Après la mort éventuelle : un ennemi tombé (mort ≥ 0) ignore les coups suivants
    talents?.apresCoup(e, degats, arme, critique, nature);
  }

  /** Coup d'une arme du joueur : ses dégâts, puis le ralentissement et l'étourdissement de ses statistiques. */
  toucher(e: Ennemi, arme: ArmeEquipee, sourceX: number, sourceZ: number, recul: number = arme.stats.recul, facteur = 1): void {
    const s = arme.stats;
    this.blesser(e, s.degats * facteur, sourceX, sourceZ, recul, arme.index);
    if (s.ralenti > 0) this.ralentir(e, s.ralenti, s.duree);
    if (s.etourdi > 0) this.etourdir(e, s.etourdi);
  }

  /** Coupe l'attaque en préparation (mèche du creeper, arc bandé) et retarde la suivante. */
  interrompre(e: Ennemi): void {
    if (e.mort >= 0) return;
    e.meche = 0;
    e.minuterie = Math.max(e.minuterie, CONFIG.ennemis.interruption);
  }

  /**
   * Centre du plus gros groupe d'ennemis à moins de `portee` de (x, z) : parmi
   * quelques ennemis tirés au hasard, celui qui a le plus de voisins dans
   * `rayon`. Écrit dans `sortie` ; faux s'il n'y a personne.
   */
  plusGrosGroupe(x: number, z: number, portee: number, rayon: number, sortie: Point): boolean {
    let meilleur = -1;
    for (let essai = 0; essai < CONFIG.ennemis.essaisGroupe; essai++) {
      const e = this.auHasard(x, z, portee);
      if (!e) return meilleur >= 0;
      const voisins = this.chercher(e.x, e.z, rayon, this.groupe);
      if (voisins > meilleur) {
        meilleur = voisins;
        sortie.x = e.x;
        sortie.z = e.z;
      }
    }
    return meilleur >= 0;
  }

  // --- États ---------------------------------------------------------------

  /**
   * Brûlure : `cumuls` de plus (plafonnés), durée remise à neuf ; les dégâts
   * les plus forts l'emportent. Les talents renforcent dégâts, durée et cumuls.
   */
  bruler(e: Ennemi, cumuls: number, degatsParSeconde: number, duree: number, arme = -1, cumulsMax: number = ETATS.brulure.cumulsMax): void {
    if (e.mort >= 0) return;
    const m = this.etat.modificateurs;
    appliquerPeriodique(
      e.brulure, cumuls, degatsParSeconde * (1 + m.degatsBrulure), duree * (1 + m.dureeBrulure), arme,
      cumulsMax + m.cumulsBrulure, ETATS.brulure.tic, this.equipe?.actif ?? 0,
    );
  }

  /** Saignement (Saignée) : comme le poison, avec ses propres cumuls. */
  saigner(e: Ennemi, cumuls: number, degatsParSeconde: number, duree: number, arme = -1): void {
    if (e.mort < 0) appliquerPeriodique(e.saignement, cumuls, degatsParSeconde, duree, arme, ETATS.saignement.cumulsMax, ETATS.saignement.tic, this.equipe?.actif ?? 0);
  }

  /** Poison : comme la brûlure, avec ses propres cumuls. */
  empoisonner(e: Ennemi, cumuls: number, degatsParSeconde: number, duree: number, arme = -1, cumulsMax: number = ETATS.poison.cumulsMax): void {
    if (e.mort >= 0) return;
    const m = this.etat.modificateurs;
    appliquerPeriodique(e.poison, cumuls, degatsParSeconde * (1 + m.degatsPoison), duree, arme, cumulsMax, ETATS.poison.tic, this.equipe?.actif ?? 0);
  }

  /** Ralentit de `part` (0 à 1) ; le plus fort l'emporte. Un boss résiste. */
  ralentir(e: Ennemi, part: number, duree: number): void {
    if (e.mort >= 0) return;
    duree *= 1 + this.etat.modificateurs.dureeRalenti;
    if (e.definition.boss) part = Math.min(RESISTANCE_BOSS.ralentiMax, part * RESISTANCE_BOSS.ralenti);
    this.appliquerRalenti(e, part, duree);
  }

  /** Immobilise sans empêcher d'attaquer ; un boss est seulement ralenti. */
  enraciner(e: Ennemi, duree: number): void {
    if (e.mort >= 0) return;
    if (e.definition.boss) this.appliquerRalenti(e, RESISTANCE_BOSS.enracinement, duree);
    else e.enracine = Math.max(e.enracine, duree);
  }

  /** Ni marche ni attaque ; sans effet sur un boss. */
  etourdir(e: Ennemi, duree: number): void {
    if (e.mort < 0 && !e.definition.boss) e.etourdi = Math.max(e.etourdi, duree);
  }

  /** Marque : les alliés invoqués lui font `bonus` de dégâts en plus (la plus forte l'emporte). */
  marquer(e: Ennemi, bonus: number, duree: number): void {
    if (e.mort >= 0) return;
    if (bonus >= e.marque || e.marqueTemps <= 0) {
      e.marque = bonus;
      e.marqueTemps = Math.max(e.marqueTemps, duree);
    }
  }

  private appliquerRalenti(e: Ennemi, part: number, duree: number): void {
    if (part >= e.ralenti || e.ralentiTemps <= 0) {
      e.ralenti = part;
      e.ralentiTemps = Math.max(e.ralentiTemps, duree);
    }
  }

  /** Durées des états, dégâts par seconde et particules. */
  private majEtats(e: Ennemi, dt: number): void {
    this.tiquer(e, e.brulure, ETATS.brulure.tic, 'brulure', dt);
    this.tiquer(e, e.poison, ETATS.poison.tic, 'poison', dt);
    this.tiquer(e, e.saignement, ETATS.saignement.tic, 'saignement', dt);
    if (e.mort >= 0) return;
    if (e.marqueChasseur > 0) e.marqueChasseur -= dt;
    if (e.dansPiege && this.etat.tempsJeu > e.piege) {
      e.dansPiege = false;
      this.talents?.surSortiePiege(e);
    }
    if (e.tempsCumuls > 0 && (e.tempsCumuls -= dt) <= 0) e.cumulsTalent = 0;
    if (e.encercle > 0) e.encercle -= dt;
    if (e.ralentiTemps > 0 && (e.ralentiTemps -= dt) <= 0) e.ralenti = 0;
    if (e.marqueTemps > 0 && (e.marqueTemps -= dt) <= 0) e.marque = 0;
    if (e.enracine > 0) e.enracine -= dt;
    if (e.etourdi > 0) e.etourdi -= dt;

    if ((e.minuterieEtats -= dt) > 0) return;
    e.minuterieEtats = ETATS.brulure.periode;
    const h = this.hauteurs[e.indexType];
    if (e.brulure.cumuls > 0) this.particules.emettre(e.x, h * 0.45, e.z, GERBES[ETATS.brulure.gerbe]);
    else if (e.poison.cumuls > 0 && Math.random() < ETATS.brulure.periode / ETATS.poison.periode) {
      this.particules.emettre(e.x, h * 0.5, e.z, GERBES[ETATS.poison.gerbe]);
    }
    if ((e.ralentiTemps > 0 || e.enracine > 0) && Math.random() < ETATS.brulure.periode / ETATS.ralenti.periode) {
      this.particules.emettre(e.x, h * 0.5, e.z, GERBES[ETATS.ralenti.gerbe]);
    }
    if (e.saignement.cumuls > 0 && Math.random() < ETATS.brulure.periode / ETATS.saignement.periode) {
      this.particules.emettre(e.x, h * 0.5, e.z, GERBES[ETATS.saignement.gerbe]);
    }
    if (e.etourdi > 0) this.particules.emettre(e.x, h - 0.1, e.z, GERBES[ETATS.etourdi.gerbe]);
    if ((e.marqueTemps > 0 || e.marqueChasseur > 0) && Math.random() < ETATS.brulure.periode / ETATS.marque.periode) {
      this.particules.emettre(e.x, h - 0.1, e.z, GERBES[ETATS.marque.gerbe]);
    }
  }

  /** Dégâts d'une brûlure ou d'un poison à chaque intervalle, jusqu'à la fin de sa durée. */
  private tiquer(e: Ennemi, p: DegatsPeriodiques, intervalle: number, nature: NatureCoup, dt: number): void {
    if (p.cumuls === 0) return;
    p.temps -= dt;
    p.tic -= dt;
    if (p.tic <= 0) {
      p.tic += intervalle;
      // À plusieurs, les dégâts (statistiques, talents) vont à celui qui a infligé l'état
      const equipe = this.equipe;
      const avant = equipe ? equipe.activer(p.source) : 0;
      this.blesser(e, p.degatsParSeconde * p.cumuls * intervalle, e.x, e.z, 0, p.arme, nature);
      if (equipe) equipe.activer(avant);
    }
    if (p.temps <= 0) p.cumuls = 0;
  }

  /** Index (pour `obtenir`) des ennemis vivants en feu ; renvoie leur nombre. */
  brulants(sortie: Int32Array): number {
    let n = 0;
    for (let i = 0; i < this.pool.nombre && n < sortie.length; i++) {
      const e = this.pool.elements[i];
      if (e.mort < 0 && e.brulure.cumuls > 0) sortie[n++] = i;
    }
    return n;
  }

  /** Fin de partie gagnée : toute la horde tombe, sans butin ni statistiques. */
  terrasserTous(): void {
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      if (e.mort >= 0) continue;
      e.mort = 0;
      e.meche = 0;
      e.blessure = CONFIG.surcouche.blessure.duree;
    }
  }

  /** Repousse sans les blesser les ennemis vivants dans un cercle (résurrection du joueur). */
  repousser(x: number, z: number, rayon: number, force: number): void {
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      const dx = e.x - x;
      const dz = e.z - z;
      const d = longueur(dx, dz);
      if (e.mort >= 0 || d > rayon + e.definition.rayon) continue;
      const k = force * (1 - (e.definition.resistanceRecul ?? 0)) / (d || 1);
      e.vx += dx * k;
      e.vz += dz * k;
    }
  }

  /** Blesse tous les ennemis vivants dans un cercle. */
  frapperZone(x: number, z: number, rayon: number, degats: number, recul: number): void {
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      const dx = e.x - x;
      const dz = e.z - z;
      if (e.mort < 0 && dx * dx + dz * dz <= (rayon + e.definition.rayon) ** 2) this.blesser(e, degats, x, z, recul);
    }
  }

  maj(dt: number): void {
    const n = this.pool.nombre;
    const elements = this.pool.elements;

    // 1. Comportement et déplacement (la grille date de la fin du pas précédent :
    //    ses index sont valides, seuls des ajouts ont pu avoir lieu depuis)
    for (let i = 0; i < n; i++) {
      const e = elements[i];
      if (e.mort < 0) this.majEtats(e, dt);
      if (e.mort >= 0) {
        e.mort += dt;
        e.vitesseSol = 0;
        continue;
      }
      this.deplacer(e, i, dt);
    }

    // 2. Retrait des ennemis explosés ou morts depuis assez longtemps
    for (let i = n - 1; i >= 0; i--) {
      const e = elements[i];
      if (e.aExploser) {
        this.exploser(e);
        this.pool.liberer(i);
      } else if (e.mort >= CONFIG.ennemis.dureeMort) {
        this.particules.emettre(e.x, 0.6, e.z, GERBES.mort);
        this.pool.liberer(i);
      }
    }

    // 3. Grille des ennemis vivants, utilisée par les armes et au pas suivant
    this.reconstruireGrille();
  }

  /** Grille des ennemis vivants (aussi chez un joueur qui a rejoint, d'après les instantanés). */
  reconstruireGrille(): void {
    const elements = this.pool.elements;
    this.grille.vider();
    for (let i = 0; i < this.pool.nombre; i++) {
      if (elements[i].mort < 0) this.grille.inserer(i, elements[i].x, elements[i].z);
    }
  }

  majVisuel(ombres: Ombres, barres: BarresVie): void {
    for (let t = 0; t < this.rendus.length; t++) this.rendus[t]?.instances.commencer();
    barres.commencer();

    const blessure = CONFIG.surcouche.blessure;
    const flash = CONFIG.surcouche.flash;
    const g = CONFIG.ennemis;
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      const rendu = this.rendu(e.indexType);
      const pose = rendu.pose;
      pose.x = e.x;
      pose.z = e.z;
      pose.angle = e.angle;
      pose.inclinaison = e.mort >= 0 ? angleChute(e.mort, g.dureeMort) : 0;
      const echelle = e.definition.echelle ?? 1;
      pose.y = hauteurFlottement(rendu.modele, e.animation) * echelle;
      pose.echelleXZ = echelle;
      pose.echelleY = echelle;
      pose.surcouche[3] = 0;
      calculerRotations(rendu.modele, e.animation, pose.rotations);

      // Creeper : gonfle et clignote en blanc avant d'exploser (courbes du jeu)
      const c = e.definition.comportement;
      if (c.type === 'explosif' && e.meche > 0) {
        const progression = Math.min(e.meche / c.duree, 1);
        const tremblement = 1 + Math.sin(progression * 100) * progression * 0.01;
        const gonflement = progression ** 3;
        pose.echelleXZ = echelle * (1 + gonflement * g.gonflementLargeur) * tremblement;
        pose.echelleY = echelle * (1 + gonflement * g.gonflementHauteur) / tremblement;
        if (Math.floor(progression * 10) % 2 === 1) {
          pose.surcouche[0] = flash.couleur[0];
          pose.surcouche[1] = flash.couleur[1];
          pose.surcouche[2] = flash.couleur[2];
          pose.surcouche[3] = flash.force * borner(progression, 0.5, 1);
        }
      }

      // Teinte de l'état le plus visible (la brûlure vacille comme des flammes)
      if (pose.surcouche[3] === 0 && e.mort < 0) {
        if (e.brulure.cumuls > 0) teinter(pose, ETATS.brulure.couleur, ETATS.brulure.force * (0.75 + 0.25 * Math.sin(this.etat.tempsJeu * 23 + e.id)));
        else if (e.poison.cumuls > 0) teinter(pose, ETATS.poison.couleur, ETATS.poison.force);
        else if (e.saignement.cumuls > 0) teinter(pose, ETATS.saignement.couleur, ETATS.saignement.force);
        else if (e.ralentiTemps > 0 || e.enracine > 0) teinter(pose, ETATS.ralenti.couleur, ETATS.ralenti.force);
      }

      // Rouge pendant la blessure et la mort
      if (e.blessure > 0 || e.mort >= 0) {
        pose.surcouche[0] = blessure.couleur[0];
        pose.surcouche[1] = blessure.couleur[1];
        pose.surcouche[2] = blessure.couleur[2];
        pose.surcouche[3] = blessure.force;
      }

      rendu.instances.ajouter(pose);
      ombres.ajouter(e.x, e.z, e.definition.rayonOmbre * (pose.echelleXZ / echelle));
      // Mini-boss : sa barre de vie au-dessus de la tête, toujours visible
      if (e.definition.miniBoss && e.mort < 0) {
        barres.ajouter(e.x, pose.y + this.hauteurs[e.indexType], e.z, e.pv / e.pvMax, g.barreMiniBoss);
      }
    }

    for (let t = 0; t < this.rendus.length; t++) this.rendus[t]?.instances.terminer();
    barres.terminer();
  }

  // -----------------------------------------------------------------------

  private deplacer(e: Ennemi, index: number, dt: number): void {
    const g = CONFIG.ennemis;
    let j = this.etat.joueur;
    let contexteJoueur = this.contexte;
    let flux = this.terrain.flux;
    const equipe = this.equipe;
    const k = equipe ? equipe.plusProche(e.x, e.z) : -1;
    if (equipe && k >= 0) {
      j = equipe.joueur(k);
      contexteJoueur = equipe.contexte(k);
      flux = equipe.flux(k);
    }
    const def = e.definition;
    if (e.minuterie > 0) e.minuterie -= dt;
    if (e.blessure > 0) e.blessure -= dt;

    // Vers le joueur : en ligne droite s'il est en vue, sinon par le champ de flux
    const carte = this.terrain.carte;
    const versX = j.x - e.x;
    const versZ = j.z - e.z;
    const distance = longueur(versX, versZ) || 1e-6;
    const dx = versX / distance;
    const dz = versZ / distance;
    const auContact = def.comportement.type === 'melee';
    if ((e.minuterieVue -= dt) <= 0) {
      e.minuterieVue = PERIODE_VUE;
      e.voitJoueur = def.ignoreObstacles === true || carte.ligneDeVue(e.x, e.z, j.x, j.z);
      if (def.comportement.type === 'archer') {
        e.ligneDeTir = carte.ligneDeTir(e.x, e.z, j.x, j.z, CONFIG.projectiles.rayonDecor);
      }
      // Un allié proche (plus que le joueur) attire les ennemis au corps à corps
      if (auContact && this.allies) {
        const a = this.allies.provocateur(e.x, e.z, distance);
        e.cibleAllie = a;
        e.cibleAllieId = a ? a.id : 0;
      }
    }
    vers.dx = dx;
    vers.dz = dz;
    vers.distance = distance;
    vers.ligneDeTir = e.ligneDeTir;
    if (!e.voitJoueur && flux.direction(e.x, e.z, chemin)) {
      vers.cheminX = chemin.x;
      vers.cheminZ = chemin.z;
    } else {
      vers.cheminX = dx;
      vers.cheminZ = dz;
    }

    // Cible : l'allié qui l'a attiré s'il vit encore, sinon le joueur
    let contexte = contexteJoueur;
    const a = e.cibleAllie;
    if (a && this.allies && this.contexteAllie && this.allies.vivant(a, e.cibleAllieId)) {
      const ax = a.x - e.x;
      const az = a.z - e.z;
      const d = longueur(ax, az) || 1e-6;
      vers.dx = ax / d;
      vers.dz = az / d;
      vers.cheminX = vers.dx;
      vers.cheminZ = vers.dz;
      // La portée de contact est calculée pour le joueur : on compense la taille de l'allié
      vers.distance = Math.max(0, d - (a.definition.rayon - JOUEUR_BASE.rayon));
      this.allieVise = a;
      contexte = this.contexteAllie;
    } else {
      e.cibleAllie = null;
      this.allieVise = null;
    }

    // Comportement (immobile si le joueur est mort ou s'il est étourdi ; enraciné, il attaque encore)
    if (j.mort || e.etourdi > 0) {
      e.dirX = 0;
      e.dirZ = 0;
    } else {
      const mecheAvant = e.meche;
      e.aExploser = appliquerComportement(e, contexte, dt, vers);
      if (mecheAvant === 0 && e.meche > 0) this.retours.son('meche');
      if (e.enracine > 0) {
        e.dirX = 0;
        e.dirZ = 0;
      }
    }

    // Séparation : repousse doucement les ennemis qui se chevauchent
    let sepX = 0;
    let sepZ = 0;
    const candidats = this.grille.requeter(e.x, e.z, def.rayon + RAYON_MAX, this.voisins);
    for (let k = 0; k < candidats; k++) {
      const autre = this.voisins[k];
      if (autre === index) continue;
      const o = this.pool.elements[autre];
      if (o.mort >= 0) continue;
      const ox = e.x - o.x;
      const oz = e.z - o.z;
      const minimum = def.rayon + o.definition.rayon;
      const d2 = ox * ox + oz * oz;
      if (d2 >= minimum * minimum || d2 < 1e-8) continue;
      const d = Math.sqrt(d2);
      sepX += (ox / d) * (minimum - d);
      sepZ += (oz / d) * (minimum - d);
    }
    const kSeparation = Math.min(1, g.separation * dt);

    // Intégration : marche voulue (ralentie par le sol) + recul amorti + séparation
    const vitesse = def.vitesse * (1 - e.ralenti) * (def.ignoreObstacles ? 1 : carte.vitesseSol(e.x, e.z, def.traverseToiles));
    e.x += (e.dirX * vitesse + e.vx) * dt + sepX * kSeparation;
    e.z += (e.dirZ * vitesse + e.vz) * dt + sepZ * kSeparation;
    const amorti = 1 - facteurLissage(g.amortissementRecul, dt);
    e.vx *= amorti;
    e.vz *= amorti;

    // Jamais dans le joueur
    const px = e.x - j.x;
    const pz = e.z - j.z;
    const contact = def.rayon + JOUEUR_BASE.rayon;
    const dj2 = px * px + pz * pz;
    if (dj2 < contact * contact && dj2 > 1e-8) {
      const dj = Math.sqrt(dj2);
      e.x = j.x + (px / dj) * contact;
      e.z = j.z + (pz / dj) * contact;
    }

    // Décor et bords de l'arène
    if (def.ignoreObstacles) {
      const limiteX = this.etat.arene.largeur / 2 - def.rayon;
      const limiteZ = this.etat.arene.profondeur / 2 - def.rayon;
      e.x = borner(e.x, -limiteX, limiteX);
      e.z = borner(e.z, -limiteZ, limiteZ);
    } else {
      carte.resoudreCollision(e.x, e.z, def.rayon, corrige);
      // Projeté contre un mur : le recul s'arrête net (Lame lourde)
      const choc = (corrige.x - e.x) ** 2 + (corrige.z - e.z) ** 2 > 1e-4 && e.vx * e.vx + e.vz * e.vz > CONFIG.ennemis.chocMur ** 2;
      e.x = corrige.x;
      e.z = corrige.z;
      if (choc) {
        e.vx = 0;
        e.vz = 0;
        this.talents?.surImpactMur(e);
      }
    }

    // Toujours tourné vers sa cible ; animation selon la marche effective
    e.angle += ecartAngle(e.angle, Math.atan2(vers.dx, vers.dz)) * facteurLissage(g.vitesseRotation, dt);
    e.vitesseSol = longueur(e.dirX, e.dirZ) * vitesse;
    // Un grand modèle fait de grands pas : cycle de marche ralenti d'autant
    avancerAnimation(e.animation, this.rendu(e.indexType).modele, dt, e.vitesseSol / (def.echelle ?? 1));
  }

  /** Explosion : particules et dégâts au joueur décroissants avec la distance. */
  private exploser(e: Ennemi): void {
    const c = e.definition.comportement;
    if (c.type !== 'explosif') return;
    this.particules.emettre(e.x, 0.8, e.z, GERBES.explosion);
    this.retours.son('explosion');
    this.retours.secousse(0.45);

    const equipe = this.equipe;
    for (let k = 0; k < (equipe ? equipe.nombre : 1); k++) {
      const j = equipe ? equipe.joueur(k) : this.etat.joueur;
      const distance = longueur(j.x - e.x, j.z - e.z);
      if (distance < c.rayon) {
        (equipe ? equipe.contexte(k) : this.contexte).cible.subirDegats(Math.ceil(c.degats * (1 - distance / c.rayon)), e.definition.boss !== undefined);
      }
    }
    this.allies?.blesserZone(e.x, e.z, c.rayon, c.degats);
  }
}

/** Ajoute des cumuls (plafonnés) et remet la durée à neuf ; les dégâts les plus forts l'emportent. */
function appliquerPeriodique(
  p: DegatsPeriodiques, cumuls: number, degatsParSeconde: number, duree: number, arme: number, cumulsMax: number, intervalle: number,
  source: number,
): void {
  if (p.cumuls === 0) {
    p.tic = intervalle;
    p.temps = 0;
    p.degatsParSeconde = 0;
  }
  p.cumuls = Math.min(cumulsMax, p.cumuls + cumuls);
  p.temps = Math.max(p.temps, duree);
  if (degatsParSeconde >= p.degatsParSeconde) {
    p.degatsParSeconde = degatsParSeconde;
    p.arme = arme;
    p.source = source;
  }
}

function teinter(pose: PoseModele, couleur: readonly [number, number, number], force: number): void {
  pose.surcouche[0] = couleur[0];
  pose.surcouche[1] = couleur[1];
  pose.surcouche[2] = couleur[2];
  pose.surcouche[3] = force;
}
