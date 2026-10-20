/**
 * Mécanique « ricochet » (Flèche ricocheuse) : une flèche spectrale file à
 * tête chercheuse vers l'ennemi le plus proche, puis rebondit sur le plus
 * proche qu'elle n'a pas encore touché (dans `distanceRebond`), en perdant
 * une part de ses dégâts à chaque rebond. Rebonds = perforation de l'arme.
 * Talents : Tir rebondissant (rebonds plus loin), Œil de l'aigle (un
 * critique ricoche vers l'ennemi le plus éloigné : `ricocher`).
 */
import { CONFIG } from '../config';
import { GERBES } from '../donnees/particules';
import { PROJECTILES } from '../donnees/projectiles';
import { RenduProjectiles } from '../effets/renduProjectiles';
import type { Ennemi } from '../entites/ennemi';
import type { SourceEntites } from '../entites/types';
import { longueur } from '../utils/maths';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

interface Ricochet {
  arme: ArmeEquipee | null;
  indexRendu: number;
  x: number;
  z: number;
  dirX: number;
  dirZ: number;
  cible: Ennemi | null;
  cibleId: number;
  degats: number;
  rebonds: number;
  /** id des ennemis déjà touchés (pas deux fois le même). */
  touches: Int32Array;
  nbTouches: number;
  vie: number;
  trainee: number;
  /** Lancée par un talent : ses coups ne relancent pas les talents. */
  reaction: boolean;
}

export class Ricochets implements SourceEntites {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(64);
  private readonly fleches: Pool<Ricochet>;

  constructor(contexte: ContexteCombat) {
    const c = CONFIG.armes.ricochet;
    this.contexte = contexte;
    this.fleches = new Pool<Ricochet>(c.capacite, () => ({
      arme: null, indexRendu: 0, x: 0, z: 0, dirX: 0, dirZ: 1, cible: null, cibleId: 0, degats: 0, rebonds: 0,
      touches: new Int32Array(c.rebondsMax + 2), nbTouches: 0, vie: 0, trainee: 0, reaction: false,
    }));
  }

  get actifs(): number {
    return this.fleches.nombre;
  }

  /** Tire les flèches vers les ennemis les plus proches ; faux si personne n'est à portée. */
  frapper(arme: ArmeEquipee): boolean {
    const m = arme.definition.mecanique;
    if (m.type !== 'ricochet') return false;
    const { etat, ennemis } = this.contexte;
    const j = etat.joueur;
    const s = arme.stats;
    if (!ennemis.plusProche(j.x, j.z, s.portee)) return false;
    for (let k = 0; k < s.nombre; k++) {
      const f = this.fleches.acquerir();
      if (!f) break;
      // La première vise le plus proche, les suivantes un ennemi au hasard
      const cible = k === 0 ? ennemis.plusProche(j.x, j.z, s.portee) : ennemis.auHasard(j.x, j.z, s.portee);
      f.arme = arme;
      f.indexRendu = RenduProjectiles.indexDe(m.projectile);
      f.x = j.x;
      f.z = j.z;
      f.cible = cible;
      f.cibleId = cible?.id ?? 0;
      f.degats = s.degats;
      f.rebonds = Math.min(CONFIG.armes.ricochet.rebondsMax, s.perforation);
      f.nbTouches = 0;
      f.vie = CONFIG.armes.ricochet.vieMax;
      f.trainee = 0;
      f.reaction = false;
    }
    return true;
  }

  /** Une flèche spectrale part de (x, z) vers `cible`, sans rebondir (réaction d'un talent). */
  ricocher(arme: ArmeEquipee, x: number, z: number, cible: Ennemi, degats: number): void {
    const f = this.fleches.acquerir();
    if (!f) return;
    f.arme = arme;
    f.indexRendu = RenduProjectiles.indexDe('fleche_spectrale');
    f.x = x;
    f.z = z;
    f.cible = cible;
    f.cibleId = cible.id;
    f.degats = degats;
    f.rebonds = 0;
    f.nbTouches = 0;
    f.vie = CONFIG.armes.ricochet.vieMax;
    f.trainee = 0;
    f.reaction = true;
  }

  maj(dt: number): void {
    const { ennemis, particules, retours } = this.contexte;
    for (let i = this.fleches.nombre - 1; i >= 0; i--) {
      const f = this.fleches.elements[i];
      const arme = f.arme;
      const m = arme?.definition.mecanique;
      f.vie -= dt;
      if (!arme || (m?.type !== 'ricochet' && !f.reaction) || f.vie <= 0) {
        this.terminer(i);
        continue;
      }
      // Cible morte en route : la plus proche que la flèche n'a pas encore touchée
      const distanceRebond = (m?.type === 'ricochet' ? m.distanceRebond : 4) + (this.contexte.talents.rang('tir_rebondissant') > 0 ? CONFIG.talents.tirRebondissant : 0);
      if (!f.cible || f.cible.mort >= 0 || f.cible.id !== f.cibleId) {
        f.cible = this.prochaine(f, distanceRebond * 2);
        f.cibleId = f.cible?.id ?? 0;
        if (!f.cible) {
          this.terminer(i);
          continue;
        }
      }
      const dx = f.cible.x - f.x;
      const dz = f.cible.z - f.z;
      const d = longueur(dx, dz) || 1e-6;
      f.dirX = dx / d;
      f.dirZ = dz / d;
      const pas = (m?.type === 'ricochet' ? arme.stats.vitesse : PROJECTILES.fleche_spectrale.vitesse) * dt;
      if (d <= pas + f.cible.definition.rayon) {
        // Touche, puis rebondit
        const e = f.cible;
        f.x = e.x;
        f.z = e.z;
        if (f.reaction) this.contexte.talents.blesser(e, f.degats, f.x - f.dirX, f.z - f.dirZ, arme.stats.recul, arme.index);
        else ennemis.blesser(e, f.degats, f.x - f.dirX, f.z - f.dirZ, arme.stats.recul, arme.index);
        if (f.nbTouches < f.touches.length) f.touches[f.nbTouches++] = e.id;
        particules.emettre(f.x, 1.2, f.z, GERBES.impactFleche);
        retours.son('impact_fleche');
        if (f.rebonds <= 0) {
          this.terminer(i);
          continue;
        }
        f.rebonds--;
        if (m?.type === 'ricochet') f.degats *= 1 - m.perte;
        f.cible = this.prochaine(f, distanceRebond);
        f.cibleId = f.cible?.id ?? 0;
        if (!f.cible) this.terminer(i);
        continue;
      }
      f.x += f.dirX * pas;
      f.z += f.dirZ * pas;
      if ((f.trainee -= dt) <= 0) {
        f.trainee = CONFIG.projectiles.intervalleTrainee;
        particules.emettre(f.x, PROJECTILES.fleche_spectrale.hauteur, f.z, GERBES.spectrale);
      }
    }
  }

  majVisuel(rendu: RenduProjectiles): void {
    for (let i = 0; i < this.fleches.nombre; i++) {
      const f = this.fleches.elements[i];
      if (f.arme) rendu.ajouter(f.indexRendu, f.x, PROJECTILES.fleche_spectrale.hauteur, f.z, Math.atan2(f.dirX, f.dirZ));
    }
  }

  /** L'ennemi le plus proche, à moins de `rayon`, que la flèche n'a pas encore touché. */
  private prochaine(f: Ricochet, rayon: number): Ennemi | null {
    const ennemis = this.contexte.ennemis;
    const n = ennemis.chercher(f.x, f.z, rayon, this.tampon);
    let meilleur: Ennemi | null = null;
    let meilleureD2 = Infinity;
    for (let k = 0; k < n; k++) {
      const e = ennemis.obtenir(this.tampon[k]);
      let deja = false;
      for (let q = 0; q < f.nbTouches; q++) if (f.touches[q] === e.id) deja = true;
      if (deja) continue;
      const d2 = (e.x - f.x) ** 2 + (e.z - f.z) ** 2;
      if (d2 < meilleureD2) {
        meilleureD2 = d2;
        meilleur = e;
      }
    }
    return meilleur;
  }

  private terminer(i: number): void {
    const f = this.fleches.elements[i];
    f.arme = null;
    f.cible = null;
    this.fleches.liberer(i);
  }
}
