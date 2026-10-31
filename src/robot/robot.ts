/**
 * Robot d'équilibrage : joue une partie seul, sans écran ni souris, à pas fixe
 * (1/60 s) et avec l'aléatoire de la simulation semé (utils/aleatoire.ts). Même
 * graine, même personnage, même arène : même partie. Il fuit les ennemis, ramasse
 * l'XP, choisit ses cartes par une règle fixe, et rend un bilan chiffré
 * (scripts/robot.mjs le lance sur N graines et en tire les statistiques de référence).
 *
 * Adresse : ?jouer=1&robot=1&graine=7&personnage=archer&arene=prairie[&duree=900]
 */
import { IDS_ARMES } from '../donnees/armes';
import type { Ennemis } from '../entites/ennemis';
import type { Orbes } from '../entites/orbes';
import type { ProjectilesEnnemis } from '../entites/projectilesEnnemis';
import type { Coffres } from '../entites/coffres';
import type { Entrees } from '../controles/entrees';
import type { EtatPartie } from '../etat';
import type { Carte, Ameliorations } from '../progression/ameliorations';
import type { Equipement } from '../progression/equipement';
import type { CarteMonde } from '../monde/carte';

/** Ce que la partie (jeu.ts) laisse voir et faire au robot. */
export interface AccesRobot {
  readonly etat: EtatPartie;
  readonly entrees: Entrees;
  readonly ennemis: Ennemis;
  readonly orbes: Orbes;
  readonly coffres: Coffres;
  readonly carte: CarteMonde;
  readonly projectiles: ProjectilesEnnemis;
  readonly ameliorations: Ameliorations;
  readonly equipement: Equipement;
  /** Un pas de simulation, caméra comprise (les ennemis apparaissent hors de sa vue). */
  pas(dt: number): void;
  /** Rend le joueur invincible (mesure de la puissance). */
  rendreInvincible(): void;
  /** Dessine une image (option `voir=1`, pour regarder jouer le robot). */
  dessiner(): void;
}

/** Bilan d'une partie du robot (JSON, lu par scripts/robot.mjs). */
export interface BilanRobot {
  readonly graine: number;
  readonly personnage: string;
  readonly arene: string;
  readonly issue: string;
  readonly temps: number;
  readonly niveau: number;
  readonly xp: number;
  readonly tues: number;
  readonly boss: number;
  readonly degatsSubis: number;
  /** Dégâts infligés par seconde, minute par minute (la « puissance »). */
  readonly puissanceParMinute: number[];
  /** Par arme obtenue : dégâts totaux et dégâts par seconde depuis son obtention. */
  readonly armes: Record<string, { degats: number; dps: number }>;
  /** Secondes réelles de calcul. */
  readonly calcul: number;
}

const PAS = 1 / 60;
/** Pas calculés avant de rendre la main au navigateur. */
const PAS_PAR_TRANCHE = 600;
const RAYON_DANGER = 9;
const DIRECTIONS = 16;
/** Distance (blocs) à laquelle le danger d'une direction est évalué. */
const ANTICIPATION = 2;
/** Durée (s) du trajet des projectiles ennemis pris en compte. */
const HORIZON_TIRS = 0.6;
/** Attrait de l'orbe d'XP la plus proche face au danger. */
const POIDS_XP = 0.08;
const RAYON_ORBES = 14;
const TOUCHES = { haut: 'KeyW', bas: 'KeyS', gauche: 'KeyA', droite: 'KeyD' } as const;

export class Robot {
  private readonly acces: AccesRobot;
  private readonly proches = new Int32Array(128);
  /** Direction suivie (rad). */
  private cap = 0;
  private degatsMinute = 0;
  private readonly puissance: number[] = [];

  constructor(acces: AccesRobot) {
    this.acces = acces;
  }

  /** Joue jusqu'à la fin de la partie (ou `duree` s de jeu), par tranches pour ne pas figer la page. */
  async jouer({ graine, duree, voir, invincible }: ParametresRobot): Promise<BilanRobot> {
    if (invincible) this.acces.rendreInvincible();
    const { etat } = this.acces;
    const debut = performance.now();
    let pas = 0;
    while (etat.issue === null && etat.tempsJeu < duree) {
      this.choisirCartes();
      this.diriger();
      this.acces.pas(PAS);
      if (++pas % 3600 === 0) this.noterMinute();
      if (voir && pas % 4 === 0) {
        this.acces.dessiner();
        await new Promise((r) => requestAnimationFrame(r));
      } else if (pas % PAS_PAR_TRANCHE === 0) await new Promise((r) => setTimeout(r, 0));
    }
    const reste = (pas % 3600) / 60;
    if (reste >= 1) this.noterMinute(reste);
    return this.bilan(graine, (performance.now() - debut) / 1000);
  }

  private noterMinute(secondes = 60): void {
    const total = this.degatsTotaux();
    this.puissance.push(Math.round((total - this.degatsMinute) / secondes));
    this.degatsMinute = total;
  }

  private degatsTotaux(): number {
    const d = this.acces.etat.stats.degatsParArme;
    let total = 0;
    for (let i = 0; i < d.length; i++) total += d[i];
    return total;
  }

  // --- Déplacement -------------------------------------------------------

  /**
   * Note 16 directions et prend la meilleure : loin des ennemis proches (là où le
   * joueur sera dans un instant), jamais dans un mur ni vers les bords, vers l'XP
   * quand la voie est libre, et sans changer de cap pour un rien (pas de tremblote).
   */
  private diriger(): void {
    const { etat, ennemis, orbes, carte, entrees, projectiles } = this.acces;
    const j = etat.joueur;
    const n = ennemis.chercher(j.x, j.z, RAYON_DANGER, this.proches);
    // Orbe d'XP la plus proche
    let ox = 0;
    let oz = 0;
    let orbe = false;
    let meilleure = RAYON_ORBES * RAYON_ORBES;
    for (let i = 0; i < orbes.pool.nombre; i++) {
      const o = orbes.pool.elements[i];
      const d2 = (o.x - j.x) ** 2 + (o.z - j.z) ** 2;
      if (d2 < meilleure) {
        meilleure = d2;
        ox = o.x;
        oz = o.z;
        orbe = true;
      }
    }
    const marge = 5;
    const demiX = etat.arene.largeur / 2 - marge;
    const demiZ = etat.arene.profondeur / 2 - marge;
    let meilleurAngle = this.cap;
    let meilleureNote = Infinity;
    for (let d = 0; d < DIRECTIONS; d++) {
      const angle = (d / DIRECTIONS) * Math.PI * 2;
      const cx = Math.cos(angle);
      const cz = Math.sin(angle);
      // Murs sur le chemin : direction exclue
      if (carte.bloqueMarche(j.x + cx, j.z + cz) || carte.bloqueMarche(j.x + cx * 2.5, j.z + cz * 2.5)) continue;
      const qx = j.x + cx * ANTICIPATION;
      const qz = j.z + cz * ANTICIPATION;
      let note = 0;
      for (let k = 0; k < n; k++) {
        const e = ennemis.obtenir(this.proches[k]);
        const d2 = (qx - e.x) ** 2 + (qz - e.z) ** 2;
        // Creeper : son explosion (rayon 3,5) fait la moitié des PV, on le fuit de loin
        note += e.definition.comportement.type === 'explosif' ? 12 / Math.max(1, d2) : 1 / Math.max(0.2, d2);
      }
      // Projectiles ennemis : distance au trajet qu'ils feront dans l'instant qui vient
      const tirs = projectiles.pool;
      for (let i = 0; i < tirs.nombre; i++) {
        const p = tirs.elements[i];
        if (p.renvoye) continue;
        const vx = p.vx * HORIZON_TIRS;
        const vz = p.vz * HORIZON_TIRS;
        const l2 = vx * vx + vz * vz + 1e-6;
        const s = Math.min(1, Math.max(0, ((qx - p.x) * vx + (qz - p.z) * vz) / l2));
        note += 1.5 / Math.max(0.2, (qx - p.x - vx * s) ** 2 + (qz - p.z - vz * s) ** 2);
      }
      // Bords de l'arène et obstacles un peu plus loin
      note += 0.05 * (Math.max(0, Math.abs(qx) - demiX) ** 2 + Math.max(0, Math.abs(qz) - demiZ) ** 2);
      if (carte.bloqueMarche(j.x + cx * 5, j.z + cz * 5)) note += 0.15;
      if (orbe) note -= POIDS_XP * ((ox - j.x) * cx + (oz - j.z) * cz) / Math.sqrt(meilleure + 0.01);
      note -= 0.02 * Math.cos(angle - this.cap);
      if (note < meilleureNote) {
        meilleureNote = note;
        meilleurAngle = angle;
      }
    }
    this.cap = meilleurAngle;
    const fx = Math.cos(meilleurAngle);
    const fz = Math.sin(meilleurAngle);
    entrees.simulerTouche(TOUCHES.droite, fx > 0.38);  // ≈ sin(22,5°) : huit directions au clavier
    entrees.simulerTouche(TOUCHES.gauche, fx < -0.38);
    entrees.simulerTouche(TOUCHES.bas, fz > 0.38);
    entrees.simulerTouche(TOUCHES.haut, fz < -0.38);
  }

  // --- Cartes ------------------------------------------------------------

  /** Cartes de niveau puis objets des coffres, choisis par une règle fixe (pas de hasard en plus). */
  private choisirCartes(): void {
    const { etat, ameliorations, equipement, coffres } = this.acces;
    if (etat.issue !== null || etat.joueur.mort) return;
    while (etat.niveauxEnAttente > 0) {
      const carte = this.meilleure(ameliorations.tirer());
      const a = carte.amelioration;
      if (a.type === 'arme' || a.type === 'eveil' || a.type === 'passif' || a.type === 'soin') ameliorations.appliquer(a);
      etat.niveauxEnAttente--;
    }
    const choix = etat.choixCoffre;
    while (choix.restants > 0) {
      const ender = choix.ender > 0;
      let carte = this.meilleure(equipement.tirer(ender, coffres.payantsOuverts));
      let a = carte.amelioration;
      if (a.type === 'objet' && equipement.remplacementNecessaire(a.id)) {
        carte = equipement.cartesRemplacement(a.id)[0];
        a = carte.amelioration;
      }
      if (a.type === 'objet' || a.type === 'renfort' || a.type === 'remplacer' || a.type === 'emeraudes') equipement.appliquer(a);
      choix.restants--;
      if (ender) choix.ender--;
    }
  }

  private meilleure(cartes: readonly Carte[]): Carte {
    let meilleure = cartes[0];
    let note = -Infinity;
    for (const c of cartes) {
      const n = this.noter(c);
      if (n > note) {
        note = n;
        meilleure = c;
      }
    }
    return meilleure;
  }

  /** Éveil > arme déjà portée > nouvelle arme > objet > passif ; soin si les PV sont bas. */
  private noter(c: Carte): number {
    const j = this.acces.etat.joueur;
    const a = c.amelioration;
    switch (a.type) {
      case 'eveil': return 6;
      case 'arme': return this.acces.etat.stats.acquisitionArme[IDS_ARMES.indexOf(a.id)] >= 0 ? 5 : 4;
      case 'renfort': return 4;
      case 'objet': return 3;
      case 'passif': return 2;
      case 'soin': return j.pv < j.pvMax * 0.4 ? 7 : 0;
      case 'remplacer': return 1;
      case 'emeraudes': return 0;
      default: return -1;
    }
  }

  // --- Bilan -------------------------------------------------------------

  private bilan(graine: number, calcul: number): BilanRobot {
    const { etat } = this.acces;
    const s = etat.stats;
    const armes: Record<string, { degats: number; dps: number }> = {};
    for (let i = 0; i < IDS_ARMES.length; i++) {
      if (s.acquisitionArme[i] < 0) continue;
      const duree = Math.max(1, etat.tempsJeu - s.acquisitionArme[i]);
      armes[IDS_ARMES[i]] = { degats: Math.round(s.degatsParArme[i]), dps: Math.round(s.degatsParArme[i] / duree) };
    }
    return {
      graine,
      personnage: etat.idPersonnage,
      arene: etat.arene.id,
      issue: etat.issue ?? 'limite',
      temps: Math.round(etat.tempsJeu * 10) / 10,
      niveau: etat.joueur.niveau,
      xp: Math.round(s.xpTotale),
      tues: etat.ennemisTues,
      boss: s.bossVaincus,
      degatsSubis: Math.round(s.degatsSubis),
      puissanceParMinute: this.puissance,
      armes,
      calcul: Math.round(calcul * 10) / 10,
    };
  }
}

/** Paramètres du robot lus dans l'adresse, ou null si ce n'est pas une partie du robot. */
export interface ParametresRobot {
  readonly graine: number;
  /** s de jeu au plus. */
  readonly duree: number;
  /** Dessine la partie (pour la regarder). */
  readonly voir: boolean;
  /** Joueur invincible : mesure la puissance des armes sur toute la partie, sans dépendre de l'esquive. */
  readonly invincible: boolean;
}

export function parametresRobot(): ParametresRobot | null {
  const adresse = new URLSearchParams(window.location.search);
  if (adresse.get('robot') !== '1') return null;
  return {
    graine: Number(adresse.get('graine') ?? 1) >>> 0,
    duree: Number(adresse.get('duree') ?? 3600), voir: adresse.get('voir') === '1',
    invincible: adresse.get('invincible') === '1',
  };
}
