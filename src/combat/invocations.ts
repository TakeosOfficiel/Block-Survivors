/**
 * Mécanique « invocation » : alliés invoqués pour un temps limité. Ils ont
 * des PV : les ennemis au corps à corps qu'ils attirent les frappent, et ils
 * peuvent mourir avant la fin de leur temps. Quatre comportements :
 * - corps à corps (golems, loups, vex) : vont frapper l'ennemi le plus proche ;
 * - tir (blazes) : restent à distance et tirent des salves de projectiles ;
 * - soutien (allay) : suit le joueur, lui donne du bouclier puis soigne les alliés ;
 * - aura (totem) : immobile, planté dans le plus gros groupe, ralentit et enracine.
 *
 * Chaque type a son quota : une invocation qui ne trouve pas de place renouvelle
 * la durée de l'allié de ce type qui a le moins de temps restant (ou, pour un
 * type à remplacement comme le totem, le replante sur le nouveau groupe).
 *
 * Talents de l'Invocateur (combat/talents/invocateur) : crochets à la
 * création, à la visée, aux coups et à la mort des alliés ; alliés temporaires
 * (Lune de sang, loups fantômes) ; haie de racines entre deux totems (Bosquet).
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { GERBES } from '../donnees/particules';
import { ALLIES, IDS_ALLIES } from '../donnees/allies';
import { MODELES } from '../donnees/modeles';
import { PLAFONDS } from '../donnees/plafonds';
import type { CheminTexture, DefinitionAllie } from '../donnees/types';
import type { BarresVie } from '../effets/barresVie';
import { CubesTextures } from '../effets/cubesTextures';
import type { Ombres } from '../effets/ombres';
import { PlansCroises } from '../effets/plansCroises';
import type { SourceEntites } from '../entites/types';
import { avancerAnimation, calculerRotations, creerEtatAnimation, declencherAttaque, reinitialiserAnimation, type EtatAnimation } from '../modeles/animation';
import { compilerModele, type ModeleCompile } from '../modeles/compilation';
import { creerPoseModele, ModeleInstancie, type PoseModele } from '../modeles/rendu';
import { ecartAngle, facteurLissage, longueur } from '../utils/maths';
import { Pool } from '../utils/pool';
import type { Point } from '../monde/carte';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';
import { ajouterBouclier, degatsApresDefense } from './defense';
import type { ProjectilesJoueur } from './projectilesJoueur';

export interface Allie {
  /** Identifiant unique de l'invocation (les index du pool changent). */
  id: number;
  indexType: number;        // index dans IDS_ALLIES
  definition: DefinitionAllie;
  arme: ArmeEquipee | null; // arme qui l'a invoqué (ses statistiques courantes)
  x: number;
  z: number;
  angle: number;
  vie: number;              // s restantes
  pv: number;
  pvMax: number;            // PV de la définition, avec les bonus des talents
  vitesse: number;          // blocs/s, avec les bonus des talents
  blessure: number;         // s de surcouche rouge restantes
  minuterie: number;        // s avant le prochain coup, tir, soutien ou pulsation d'aura
  vitesseSol: number;
  /** Temps de jeu du dernier soin reçu (soins limités par allié). */
  dernierSoin: number;
  animation: EtatAnimation;
  // Talents
  /** Sa première morsure est renforcée (Chef de meute). */
  premiereMorsure: boolean;
  /** Loup alpha (Alpha), déjà revenu après sa mort (Gardien alpha), taille d'affichage. */
  alpha: boolean;
  revenu: boolean;
  echelle: number;
  /** Invoqué pour un temps par un talent : hors quota, jamais renouvelé. */
  temporaire: boolean;
}

const corrige: Point = { x: 0, z: 0 };
const groupe: Point = { x: 0, z: 0 };

interface RenduType {
  readonly modele: ModeleCompile;
  readonly instances: ModeleInstancie;
  readonly pose: PoseModele;
}

/** Alliés en blocs (totem) : cubes empilés et racines, par texture. */
interface RenduBlocs {
  readonly cubes: Map<CheminTexture, CubesTextures>;
  readonly racines: Map<CheminTexture, PlansCroises>;
}

export class Invocations implements SourceEntites {
  private readonly contexte: ContexteCombat;
  private readonly projectiles: ProjectilesJoueur;
  private readonly tampon = new Int32Array(128);
  private readonly pool = new Pool<Allie>(CONFIG.armes.golems.capacite, () => ({
    id: 0, indexType: 0, definition: ALLIES[IDS_ALLIES[0]], arme: null,
    x: 0, z: 0, angle: 0, vie: 0, pv: 1, pvMax: 1, vitesse: 0, blessure: 0, minuterie: 0, vitesseSol: 0, dernierSoin: -Infinity, animation: creerEtatAnimation(),
    premiereMorsure: false, alpha: false, revenu: false, echelle: 1, temporaire: false,
  }));
  /** Rendu par type ; null pour un allié en blocs. */
  private readonly rendus: (RenduType | null)[];
  private readonly blocs: RenduBlocs = { cubes: new Map(), racines: new Map() };
  private readonly barres: BarresVie;
  private prochainId = 1;
  /** s avant le prochain ralentissement de la haie du Bosquet. */
  private minuterieHaie = 0;

  constructor(
    contexte: ContexteCombat, scene: THREE.Scene, textures: BibliothequeTextures, barres: BarresVie, projectiles: ProjectilesJoueur,
  ) {
    this.contexte = contexte;
    this.projectiles = projectiles;
    this.barres = barres;
    const capacite = CONFIG.armes.golems.capacite;
    this.rendus = IDS_ALLIES.map((id) => {
      const def = ALLIES[id];
      if (def.blocs) {
        for (const texture of def.blocs.empilement) {
          if (!this.blocs.cubes.has(texture)) this.blocs.cubes.set(texture, new CubesTextures(scene, textures, texture, capacite * def.blocs.empilement.length, false, def.nom));
        }
        if (!this.blocs.racines.has(def.blocs.racines)) {
          this.blocs.racines.set(def.blocs.racines, new PlansCroises(scene, textures, def.blocs.racines, capacite * CONFIG.armes.invocations.racines, def.nom));
        }
        return null;
      }
      const modele = compilerModele(MODELES[def.modele]);
      return {
        modele,
        instances: new ModeleInstancie(modele, def, textures, capacite, scene),
        pose: creerPoseModele(modele),
      };
    });
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  /**
   * Invoque `nombre` alliés ; au quota, chacun renouvelle à la place l'allié
   * de ce type qui a le moins de temps restant (ou le replante, pour un type à
   * remplacement). Renvoie le nombre de nouveaux alliés.
   */
  invoquer(arme: ArmeEquipee): number {
    const m = arme.definition.mecanique;
    if (m.type !== 'invocation') return 0;
    const def = ALLIES[m.allie];
    const indexType = IDS_ALLIES.indexOf(m.allie);
    // Un totem se plante dans le plus gros groupe proche ; sans ennemi, pas de totem
    if (def.aura) {
      const j = this.contexte.etat.joueur;
      if (!this.contexte.ennemis.plusGrosGroupe(j.x, j.z, CONFIG.armes.invocations.porteeTotem, arme.stats.zone, groupe)) return 0;
    }
    let nouveaux = 0;
    for (let k = 0; k < arme.stats.nombre; k++) {
      const quota = def.quota + this.contexte.talents.quotaAllie(def);
      if (this.presents(indexType) < quota && this.creer(arme, indexType)) nouveaux++;
      else if (def.remplacement) this.replanter(indexType, arme.stats.duree);
      else this.renouveler(indexType, arme.stats.duree);
    }
    return nouveaux;
  }

  /** Alliés vivants de ce type. */
  presents(indexType: number): number {
    let n = 0;
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      if (a.indexType === indexType && a.arme && a.pv > 0) n++;
    }
    return n;
  }

  /** L'allié vivant de ce type qui a le moins de temps restant. */
  private plusAncien(indexType: number): Allie | null {
    let choisi: Allie | null = null;
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      if (a.indexType === indexType && a.arme && a.pv > 0 && !a.temporaire && (!choisi || a.vie < choisi.vie)) choisi = a;
    }
    return choisi;
  }

  /** Remet à `duree` le temps restant de l'allié de ce type qui en a le moins. */
  private renouveler(indexType: number, duree: number): void {
    const a = this.plusAncien(indexType);
    if (a && a.vie < duree) a.vie = duree;
  }

  /** Replante le plus ancien sur le groupe visé (totem), avec toute sa durée. */
  private replanter(indexType: number, duree: number): void {
    const a = this.plusAncien(indexType);
    if (!a) return;
    this.contexte.particules.emettre(a.x, 1, a.z, GERBES.invocation);
    a.x = groupe.x;
    a.z = groupe.z;
    a.vie = duree;
    this.contexte.particules.emettre(a.x, 1, a.z, GERBES.invocation);
  }

  private creer(arme: ArmeEquipee, indexType: number, temporaire = false): boolean {
    const m = arme.definition.mecanique;
    if (m.type !== 'invocation') return false;
    const a = this.pool.acquerir();
    if (!a) return false;
    const j = this.contexte.etat.joueur;
    const def = ALLIES[m.allie];
    const angle = Math.random() * Math.PI * 2;
    a.indexType = indexType;
    a.definition = def;
    a.arme = arme;
    // Le totem se plante sur le groupe ; les autres apparaissent autour du joueur
    a.x = def.aura ? groupe.x + Math.sin(angle) * 0.6 : j.x + Math.sin(angle) * 1.5;
    a.z = def.aura ? groupe.z + Math.cos(angle) * 0.6 : j.z + Math.cos(angle) * 1.5;
    a.angle = j.angle;
    a.id = this.prochainId++;
    a.vie = arme.stats.duree;
    const mods = this.contexte.etat.modificateurs;
    a.pvMax = def.pv * (1 + mods.pvAllies + (arme.talent?.pv ?? 0));
    a.pv = a.pvMax;
    a.vitesse = def.vitesse * (1 + mods.vitesseAllies + (arme.talent?.vitesse ?? 0));
    a.blessure = 0;
    a.minuterie = def.soutien ? 1 : 0;
    a.vitesseSol = 0;
    a.dernierSoin = -Infinity;
    a.premiereMorsure = false;
    a.alpha = false;
    a.revenu = false;
    a.echelle = 1;
    a.temporaire = temporaire;
    reinitialiserAnimation(a.animation);
    this.contexte.talents.surCreationAllie(a);
    this.contexte.particules.emettre(a.x, 1, a.z, GERBES.invocation);
    return true;
  }

  /**
   * Invoque pour `duree` s des alliés de l'arme hors quota (Lune de sang,
   * loups fantômes), avec une part de leurs PV ; ils ne sont jamais renouvelés
   * et leur fin n'est pas une mort.
   */
  invoquerTemporaire(arme: ArmeEquipee, nombre: number, duree: number, partPv: number): void {
    const m = arme.definition.mecanique;
    if (m.type !== 'invocation') return;
    const indexType = IDS_ALLIES.indexOf(m.allie);
    for (let k = 0; k < nombre; k++) {
      const a = this.pool.elements[this.pool.nombre];
      if (!a || !this.creer(arme, indexType, true)) return;
      a.vie = duree;
      a.pvMax *= partPv;
      a.pv = a.pvMax;
    }
  }

  /** Un allié vivant à moins de `rayon` de (x, z) ? */
  allieProche(x: number, z: number, rayon: number): boolean {
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      if (a.arme && a.pv > 0 && (a.x - x) ** 2 + (a.z - z) ** 2 <= rayon * rayon) return true;
    }
    return false;
  }

  /** Alliés en place (les `nombre` premiers éléments). */
  get liste(): readonly Allie[] {
    return this.pool.elements;
  }

  get nombre(): number {
    return this.pool.nombre;
  }

  maj(dt: number): void {
    const particules = this.contexte.particules;
    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const a = this.pool.elements[i];
      const arme = a.arme;
      a.vie -= dt;
      if (!arme || a.vie <= 0) {
        particules.emettre(a.x, 1, a.z, GERBES.invocation);
        a.arme = null;
        this.pool.liberer(i);
        continue;
      }
      if (a.minuterie > 0) a.minuterie -= dt;
      if (a.blessure > 0) a.blessure -= dt;
      const def = a.definition;
      if (def.aura) this.pulserAura(a, arme);
      else if (def.soutien) this.soutenir(a, arme, dt);
      else if (def.tir) this.tirer(a, arme, dt);
      else this.combattre(a, arme, dt);
      const rendu = this.rendus[a.indexType];
      if (rendu) avancerAnimation(a.animation, rendu.modele, dt, a.vitesseSol);
    }
    if (this.contexte.talents.rang('bosquet') > 0 && (this.minuterieHaie -= dt) <= 0) {
      this.minuterieHaie = CONFIG.armes.invocations.haie.periode;
      this.ralentirHaies();
    }
  }

  /** Bosquet : deux totems assez proches font pousser entre eux une haie de racines qui ralentit. */
  private ralentirHaies(): void {
    const h = CONFIG.armes.invocations.haie;
    const ennemis = this.contexte.ennemis;
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      if (!a.definition.aura || !a.arme) continue;
      for (let k = i + 1; k < this.pool.nombre; k++) {
        const b = this.pool.elements[k];
        if (!b.definition.aura || !b.arme) continue;
        const d = longueur(b.x - a.x, b.z - a.z);
        if (d > h.distance) continue;
        const pas = Math.max(1, Math.ceil(d / h.espacement));
        for (let s = 0; s <= pas; s++) {
          const x = a.x + ((b.x - a.x) * s) / pas;
          const z = a.z + ((b.z - a.z) * s) / pas;
          const n = ennemis.chercher(x, z, h.largeur, this.tampon);
          for (let q = 0; q < n; q++) ennemis.ralentir(ennemis.obtenir(this.tampon[q]), h.ralenti, h.periode * 2);
        }
      }
    }
  }

  /** Racines de la haie entre deux totems proches (Bosquet). */
  private dessinerHaies(): void {
    const h = CONFIG.armes.invocations.haie;
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      const blocs = a.definition.blocs;
      if (!a.definition.aura || !a.arme || !blocs) continue;
      const racines = this.blocs.racines.get(blocs.racines);
      for (let k = i + 1; k < this.pool.nombre; k++) {
        const b = this.pool.elements[k];
        if (!b.definition.aura || !b.arme) continue;
        const d = longueur(b.x - a.x, b.z - a.z);
        if (d > h.distance) continue;
        const pas = Math.max(1, Math.ceil(d / h.espacement));
        for (let s = 1; s < pas; s++) {
          const x = a.x + ((b.x - a.x) * s) / pas;
          const z = a.z + ((b.z - a.z) * s) / pas;
          racines?.ajouter(x, 0.35, z, 0.75, s * 1.7 + a.id);
        }
      }
    }
  }

  /** Corps à corps : va frapper l'ennemi le plus proche, sinon reste près du joueur. */
  private combattre(a: Allie, arme: ArmeEquipee, dt: number): void {
    const def = a.definition;
    const talents = this.contexte.talents;
    const cible = talents.ciblePrioritaire(a) ?? this.contexte.ennemis.plusProche(a.x, a.z, def.detection);
    const j = this.contexte.etat.joueur;
    const arret = cible ? def.rayon + cible.definition.rayon + arme.stats.zone : CONFIG.armes.golems.distanceSuivi;
    const distance = this.allerVers(a, cible ? cible.x : j.x, cible ? cible.z : j.z, arret, dt);
    if (cible && distance <= arret && a.minuterie <= 0) {
      const degats = arme.stats.degats * talents.multiplicateurAllie(a, cible);
      a.premiereMorsure = false;
      this.contexte.ennemis.blesser(cible, degats, a.x, a.z, arme.stats.recul, arme.index, 'allie');
      talents.surMorsure(a, cible);
      this.contexte.retours.son(def.sonCoup);
      const rendu = this.rendus[a.indexType];
      if (rendu) declencherAttaque(a.animation, rendu.modele);
      a.minuterie = def.cadenceCoups * talents.cadenceAllie(a);
    }
  }

  /** Tir : se place à distance de l'ennemi le plus proche et tire des salves ; sans cible, suit le joueur. */
  private tirer(a: Allie, arme: ArmeEquipee, dt: number): void {
    const def = a.definition;
    const tir = def.tir;
    if (!tir) return;
    const j = this.contexte.etat.joueur;
    const talents = this.contexte.talents;
    const cible = talents.ciblePrioritaire(a) ?? this.contexte.ennemis.plusProche(a.x, a.z, Math.min(def.detection, arme.stats.portee));
    if (!cible) {
      this.allerVers(a, j.x, j.z, CONFIG.armes.golems.distanceSuivi, dt);
      return;
    }
    const distance = this.allerVers(a, cible.x, cible.z, tir.distance, dt);
    if (distance > arme.stats.portee || a.minuterie > 0) return;
    const m = arme.definition.mecanique;
    // Brûlure au dernier rang, ou dès le rang 1 avec Braises de blaze (1 / 2 cumuls : dégâts doublés au rang 2)
    const braises = talents.rang('braises_blaze');
    const brulure = braises > 0 ? CONFIG.armes.invocations.degatsBrulure * braises
      : m.type === 'invocation' && m.brulureAuRang !== undefined && arme.rang >= m.brulureAuRang ? CONFIG.armes.invocations.degatsBrulure : 0;
    const base = Math.atan2(cible.x - a.x, cible.z - a.z);
    for (let k = 0; k < tir.salve; k++) {
      const angle = base + (k - (tir.salve - 1) / 2) * tir.ecart;
      this.projectiles.lancer(arme, tir.projectile, 'ligne', Math.sin(angle), Math.cos(angle), false, a.x, a.z, brulure, 'allie');
    }
    this.contexte.retours.son(def.sonCoup, 1.3);
    a.minuterie = def.cadenceCoups * talents.cadenceAllie(a);
  }

  /** Soutien : suit le joueur ; à chaque délai de l'arme, bouclier au joueur, sinon soin de l'allié le plus blessé. */
  private soutenir(a: Allie, arme: ArmeEquipee, dt: number): void {
    const { etat, particules, retours } = this.contexte;
    const j = etat.joueur;
    this.allerVers(a, j.x, j.z, CONFIG.armes.golems.distanceSuivi * 0.6, dt);
    if (a.minuterie > 0) return;
    a.minuterie = arme.stats.cadence;
    if (j.bouclier < PLAFONDS.bouclier * j.pvMax - 0.05) {
      ajouterBouclier(j, arme.stats.degats);
      particules.emettre(j.x, 1.4, j.z, GERBES.bouclier);
      retours.son(a.definition.sonCoup);
      return;
    }
    // Bouclier plein : l'allié le plus blessé (une fois par allié toutes les `delaiSoin` s)
    let blesse: Allie | null = null;
    for (let i = 0; i < this.pool.nombre; i++) {
      const b = this.pool.elements[i];
      if (!b.arme || b.pv <= 0 || b.pv >= b.pvMax || etat.tempsJeu - b.dernierSoin < CONFIG.armes.invocations.delaiSoin) continue;
      if (!blesse || b.pv / b.pvMax < blesse.pv / blesse.pvMax) blesse = b;
    }
    if (!blesse) return;
    blesse.pv = Math.min(blesse.pvMax, blesse.pv + blesse.pvMax * arme.stats.zone);
    blesse.dernierSoin = etat.tempsJeu;
    particules.emettre(blesse.x, 1, blesse.z, GERBES.niveau);
  }

  /** Aura du totem : ralentit (et blesse) autour de lui, enracine ceux qui entrent. */
  private pulserAura(a: Allie, arme: ArmeEquipee): void {
    const aura = a.definition.aura;
    if (!aura || a.minuterie > 0) return;
    a.minuterie = aura.intervalle;
    const { ennemis, etat } = this.contexte;
    const s = arme.stats;
    const n = ennemis.chercher(a.x, a.z, s.zone, this.tampon);
    for (let k = 0; k < n; k++) {
      const e = ennemis.obtenir(this.tampon[k]);
      ennemis.ralentir(e, s.ralenti, aura.intervalle * 1.5);
      if (s.degats > 0) ennemis.blesser(e, s.degats, a.x, a.z, 0, arme.index, 'allie');
      if (e.immunites[arme.index] <= etat.tempsJeu) {
        ennemis.enraciner(e, aura.enracinement);
        e.immunites[arme.index] = etat.tempsJeu + aura.delai;
      }
    }
  }

  /** S'approche de (x, z) jusqu'à `arret` blocs ; renvoie la distance restante. */
  private allerVers(a: Allie, x: number, z: number, arret: number, dt: number): number {
    const def = a.definition;
    let versX = x - a.x;
    let versZ = z - a.z;
    const distance = longueur(versX, versZ) || 1e-6;
    versX /= distance;
    versZ /= distance;
    a.vitesseSol = 0;
    const vitesse = a.vitesse * this.contexte.talents.vitesseAllie(a);
    if (distance > arret && vitesse > 0) {
      const nx = a.x + versX * vitesse * dt;
      const nz = a.z + versZ * vitesse * dt;
      if (def.vol !== undefined) {
        // Volant : traverse le décor
        a.x = nx;
        a.z = nz;
      } else {
        this.contexte.carte.resoudreCollision(nx, nz, def.rayon, corrige);
        a.x = corrige.x;
        a.z = corrige.z;
      }
      a.vitesseSol = vitesse;
    }
    a.angle += ecartAngle(a.angle, Math.atan2(versX, versZ)) * facteurLissage(10, dt);
    return distance;
  }

  /**
   * Allié qui détourne un ennemi en (x, z) : le plus proche à portée de sa
   * provocation, et plus proche que le joueur (à `distanceJoueur`). Null sinon.
   */
  provocateur(x: number, z: number, distanceJoueur: number): Allie | null {
    let meilleur: Allie | null = null;
    let meilleure = distanceJoueur;
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      if (a.pv <= 0 || !a.arme) continue;
      const d = longueur(a.x - x, a.z - z);
      const provocation = a.definition.provocation * (a.alpha && a.echelle > 1 ? CONFIG.armes.invocations.provocationAlpha : 1);
      if (d < provocation && d < meilleure) {
        meilleure = d;
        meilleur = a;
      }
    }
    return meilleur;
  }

  /** L'allié est-il toujours la même invocation, et en vie ? */
  vivant(a: Allie, id: number): boolean {
    return a.id === id && a.arme !== null && a.pv > 0;
  }

  blesser(a: Allie, degats: number): void {
    if (a.pv <= 0 || !a.arme) return;
    a.pv -= degatsApresDefense(degats, a.definition.armure ?? 0);
    a.blessure = CONFIG.surcouche.blessure.duree;
    const retours = this.contexte.retours;
    if (a.pv > 0) {
      retours.son(a.definition.sonBlessure);
      return;
    }
    // Mort (0 PV) : un talent peut le ramener, sinon il disparaît au prochain pas (poof d'invocation)
    if (this.contexte.talents.surMortAllie(a)) return;
    retours.son(a.definition.sonMort);
    a.vie = 0;
  }

  /** Explosion (creeper) : blesse les alliés dans le rayon, moins au bord. */
  blesserZone(x: number, z: number, rayon: number, degats: number): void {
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      const d = longueur(a.x - x, a.z - z);
      if (d < rayon) this.blesser(a, Math.ceil(degats * (1 - d / rayon)));
    }
  }

  majVisuel(ombres: Ombres): void {
    for (let t = 0; t < this.rendus.length; t++) this.rendus[t]?.instances.commencer();
    for (const cubes of this.blocs.cubes.values()) cubes.commencer();
    for (const racines of this.blocs.racines.values()) racines.commencer();
    this.barres.commencer();
    const blessure = CONFIG.surcouche.blessure;
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      const rendu = this.rendus[a.indexType];
      ombres.ajouter(a.x, a.z, a.definition.rayonOmbre);
      if (!rendu) {
        const hauteur = this.dessinerBlocs(a);
        if (a.pv < a.pvMax) this.barres.ajouter(a.x, hauteur + 0.35, a.z, a.pv / a.pvMax);
        continue;
      }
      const pose = rendu.pose;
      pose.x = a.x;
      pose.z = a.z;
      pose.angle = a.angle;
      pose.echelleXZ = a.echelle;
      pose.echelleY = a.echelle;
      // Les volants flottent en ondulant doucement
      pose.y = a.definition.vol !== undefined ? a.definition.vol + Math.sin(a.vie * 4) * 0.12 : 0;
      calculerRotations(rendu.modele, a.animation, pose.rotations);
      // Rouge quand il est touché, comme les mobs
      pose.surcouche[0] = blessure.couleur[0];
      pose.surcouche[1] = blessure.couleur[1];
      pose.surcouche[2] = blessure.couleur[2];
      pose.surcouche[3] = a.blessure > 0 ? blessure.force : 0;
      rendu.instances.ajouter(pose);
      // Barre de vie dès la première blessure
      if (a.pv < a.pvMax) this.barres.ajouter(a.x, pose.y + rendu.modele.hauteur + 0.35, a.z, a.pv / a.pvMax);
    }
    if (this.contexte.talents.rang('bosquet') > 0) this.dessinerHaies();
    for (let t = 0; t < this.rendus.length; t++) this.rendus[t]?.instances.terminer();
    for (const cubes of this.blocs.cubes.values()) cubes.terminer();
    for (const racines of this.blocs.racines.values()) racines.terminer();
    this.barres.terminer();
  }

  /** Totem : blocs empilés qui sortent de terre, entourés de racines ; renvoie sa hauteur. */
  private dessinerBlocs(a: Allie): number {
    const blocs = a.definition.blocs;
    if (!blocs) return 0;
    const c = CONFIG.armes.invocations;
    const duree = a.arme?.stats.duree ?? 1;
    const sortie = Math.min(1, (duree - a.vie) / 0.3, a.vie / 0.4);
    const tremble = a.blessure > 0 ? (Math.random() - 0.5) * 0.08 : 0;
    for (let k = 0; k < blocs.empilement.length; k++) {
      this.blocs.cubes.get(blocs.empilement[k])?.ajouter(a.x + tremble, (k + 0.5) * sortie - (1 - sortie) * 0.5, a.z, 0.9, a.id * 0.7);
    }
    const racines = this.blocs.racines.get(blocs.racines);
    for (let k = 0; k < c.racines; k++) {
      const angle = (k / c.racines) * Math.PI * 2 + a.id;
      const r = 0.75 + (k % 2) * 0.35;
      racines?.ajouter(a.x + Math.sin(angle) * r, 0.4 * sortie, a.z + Math.cos(angle) * r, 0.8 * sortie, angle);
    }
    return blocs.empilement.length * sortie;
  }
}
