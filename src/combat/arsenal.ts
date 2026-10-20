/**
 * Arsenal du joueur : armes possédées, rangs, et déclenchement automatique de
 * chaque arme selon sa mécanique. Les effets sont délégués aux systèmes de
 * combat (balayages, projectiles, orbites, foudre, invocations).
 */
import { ARMES, type IdArme } from '../donnees/armes';
import type { ArmeEquipee } from './armeEquipee';
import { creerArmeEquipee, rangMax, recalculerStats } from './armeEquipee';
import type { Balayages } from './balayages';
import type { ContexteCombat } from './contexte';
import type { Foudre } from './foudre';
import type { Invocations } from './invocations';
import type { Orbites } from './orbites';
import type { ProjectilesJoueur } from './projectilesJoueur';

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

  rang(id: IdArme): number {
    return this.armes.find((a) => a.id === id)?.rang ?? 0;
  }

  /** Ajoute l'arme au rang 1, ou la monte d'un rang ; faux si elle est déjà au maximum. */
  ameliorer(id: IdArme): boolean {
    const existante = this.armes.find((a) => a.id === id);
    if (!existante) {
      this.armes.push(creerArmeEquipee(id));
    } else {
      if (existante.rang >= rangMax(ARMES[id])) return false;
      existante.rang++;
      recalculerStats(existante);
    }
    this.version++;
    return true;
  }

  maj(dt: number): void {
    if (this.contexte.etat.joueur.mort) return;
    for (let i = 0; i < this.armes.length; i++) {
      const arme = this.armes[i];
      if (arme.definition.mecanique.type === 'orbite') {
        this.systemes.orbites.maj(arme, dt);
        continue;
      }
      arme.minuterie -= dt;
      if (arme.minuterie > 0) continue;
      arme.minuterie = this.declencher(arme) ? arme.stats.cadence : NOUVEL_ESSAI;
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
          this.systemes.projectiles.lancer(arme, m.projectile, m.type === 'retour' ? 'retour' : 'ligne', Math.sin(a), Math.cos(a));
        }
        return true;
      }

      case 'explosif': {
        let lance = false;
        for (let k = 0; k < s.nombre; k++) {
          const cible = ennemis.auHasard(j.x, j.z, s.portee);
          if (!cible) break;
          const a = Math.atan2(cible.x - j.x, cible.z - j.z);
          this.systemes.projectiles.lancer(arme, m.projectile, 'ligne', Math.sin(a), Math.cos(a), m.couleurs);
          lance = true;
        }
        return lance;
      }

      case 'foudre':
        return this.systemes.foudre.frapper(arme);

      case 'invocation':
        // Au maximum d'alliés, on attend simplement la prochaine invocation
        this.systemes.invocations.invoquer(arme);
        return true;

      case 'orbite':
        return true;
    }
  }
}
