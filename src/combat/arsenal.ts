/**
 * Arsenal du joueur : armes possédées, rangs, et déclenchement automatique de
 * chaque arme selon sa mécanique. Les effets sont délégués aux systèmes de
 * combat (balayages, projectiles, orbites, foudre, invocations).
 */
import { ARMES, type IdArme } from '../donnees/armes';
import type { MecaniqueArme } from '../donnees/types';
import type { ArmeEquipee } from './armeEquipee';
import { calculerStats, creerArmeEquipee, rangMax } from './armeEquipee';
import type { Balayages } from './balayages';
import type { ContexteCombat } from './contexte';
import type { Foudre } from './foudre';
import type { Invocations } from './invocations';
import type { Orbites } from './orbites';
import type { ModeProjectile, ProjectilesJoueur } from './projectilesJoueur';

/** Délai avant de réessayer une arme qui n'a trouvé aucune cible (s). */
const NOUVEL_ESSAI = 0.2;

export interface SystemesCombat {
  readonly balayages: Balayages;
  readonly projectiles: ProjectilesJoueur;
  readonly orbites: Orbites;
  readonly foudre: Foudre;
  readonly invocations: Invocations;
}

export class Arsenal {
  readonly armes: ArmeEquipee[] = [];
  /** Incrémenté à chaque changement (pour rafraîchir l'interface). */
  version = 0;

  private readonly contexte: ContexteCombat;
  private readonly systemes: SystemesCombat;

  constructor(contexte: ContexteCombat, systemes: SystemesCombat) {
    this.contexte = contexte;
    this.systemes = systemes;
  }

  /** Rang possédé (0 = arme absente). */
  rang(id: IdArme): number {
    return this.armes.find((a) => a.id === id)?.rang ?? 0;
  }

  /** Ajoute l'arme au rang 1, ou la monte d'un rang ; faux si elle est déjà au maximum. */
  ameliorer(id: IdArme): boolean {
    const modificateurs = this.contexte.etat.modificateurs;
    const existante = this.armes.find((a) => a.id === id);
    if (!existante) {
      this.armes.push(creerArmeEquipee(id, modificateurs));
    } else {
      if (existante.rang >= rangMax(ARMES[id])) return false;
      existante.rang++;
      calculerStats(existante.definition, existante.rang, modificateurs, existante.stats);
    }
    this.version++;
    return true;
  }

  /** À appeler quand les modificateurs des passifs changent. */
  recalculer(): void {
    const modificateurs = this.contexte.etat.modificateurs;
    for (const arme of this.armes) calculerStats(arme.definition, arme.rang, modificateurs, arme.stats);
    this.version++;
  }

  maj(dt: number): void {
    if (this.contexte.etat.joueur.mort || this.contexte.etat.issue !== null) return;
    for (let i = 0; i < this.armes.length; i++) {
      const arme = this.armes[i];
      if (arme.definition.mecanique.type === 'orbite') {
        this.systemes.orbites.maj(arme, dt);
        continue;
      }
      arme.minuterie -= dt;
      if (arme.minuterie > 0) continue;
      const declenchee = this.declencher(arme);
      arme.minuterie = declenchee ? arme.stats.cadence : NOUVEL_ESSAI;
      const son = arme.definition.son;
      if (declenchee && son && arme.definition.mecanique.type !== 'invocation') this.contexte.retours.son(son);
    }
  }

  /** Déclenche l'arme ; faux si elle n'avait aucune cible. */
  private declencher(arme: ArmeEquipee): boolean {
    const m = arme.definition.mecanique;
    const { etat, ennemis } = this.contexte;
    const j = etat.joueur;
    const s = arme.stats;

    switch (m.type) {
      case 'balayage':
        return this.systemes.balayages.frapper(arme);

      case 'tir':
      case 'retour': {
        const cible = ennemis.plusProche(j.x, j.z, s.portee);
        if (!cible) return false;
        const base = Math.atan2(cible.x - j.x, cible.z - j.z);
        for (let k = 0; k < s.nombre; k++) {
          const a = base + (k - (s.nombre - 1) / 2) * m.dispersion;
          this.systemes.projectiles.lancer(arme, m.projectile, modeTir(m, arme.rang), Math.sin(a), Math.cos(a));
        }
        return true;
      }

      case 'explosif': {
        let lance = false;
        for (let k = 0; k < s.nombre; k++) {
          const cible = ennemis.auHasard(j.x, j.z, s.portee);
          if (!cible) break;
          const a = Math.atan2(cible.x - j.x, cible.z - j.z);
          this.systemes.projectiles.lancer(arme, m.projectile, 'ligne', Math.sin(a), Math.cos(a), true);
          lance = true;
        }
        return lance;
      }

      case 'foudre':
        return this.systemes.foudre.frapper(arme);

      case 'invocation':
        // Au maximum d'alliés, on attend simplement la prochaine invocation
        if (this.systemes.invocations.invoquer(arme) && arme.definition.son) this.contexte.retours.son(arme.definition.son);
        return true;

      case 'orbite':
        return true;
    }
  }
}

/** Le trident ne revient qu'avec Loyauté ; avant, il part en ligne et traverse tout. */
function modeTir(m: MecaniqueArme, rang: number): ModeProjectile {
  if (m.type !== 'retour') return 'ligne';
  return rang >= m.loyaute ? 'retour' : 'traversant';
}
