/**
 * Mécanique « heurt » (Heurt de bouclier) : coup de bouclier court en cône
 * vers l'ennemi le plus proche. Il repousse fort, interrompt les attaques en
 * préparation (mèche du creeper, arc bandé) et, à partir du rang `renvoi`,
 * renvoie les projectiles ennemis du cône contre leurs tireurs.
 */
import { CONFIG } from '../config';
import { GERBES } from '../donnees/particules';
import type { ProjectilesEnnemis } from '../entites/projectilesEnnemis';
import { ecartAngle } from '../utils/maths';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

export class Heurts {
  private readonly contexte: ContexteCombat;
  private readonly projectilesEnnemis: ProjectilesEnnemis;
  private readonly tampon = new Int32Array(128);

  constructor(contexte: ContexteCombat, projectilesEnnemis: ProjectilesEnnemis) {
    this.contexte = contexte;
    this.projectilesEnnemis = projectilesEnnemis;
  }

  /** Frappe vers l'ennemi le plus proche ; faux si aucun n'est à portée. */
  frapper(arme: ArmeEquipee): boolean {
    const m = arme.definition.mecanique;
    if (m.type !== 'heurt') return false;
    const { etat, ennemis, particules, retours } = this.contexte;
    const j = etat.joueur;
    const s = arme.stats;
    const cible = ennemis.plusProche(j.x, j.z, s.zone + CONFIG.armes.heurt.marge);
    if (!cible) return false;
    const angle = Math.atan2(cible.x - j.x, cible.z - j.z);

    const n = ennemis.chercher(j.x, j.z, s.zone, this.tampon);
    for (let k = 0; k < n; k++) {
      const e = ennemis.obtenir(this.tampon[k]);
      const dx = e.x - j.x;
      const dz = e.z - j.z;
      const colle = dx * dx + dz * dz < 0.8 * 0.8;
      if (!colle && Math.abs(ecartAngle(angle, Math.atan2(dx, dz))) > m.angle / 2) continue;
      ennemis.toucher(e, arme, j.x, j.z);
      ennemis.interrompre(e);
    }
    if (arme.rang >= m.renvoi) {
      this.projectilesEnnemis.renvoyer(j.x, j.z, angle, m.angle / 2, s.zone + CONFIG.armes.heurt.porteeRenvoi, s.degats, arme.index);
    }

    this.contexte.declencherGesteJoueur(angle);
    const devant = s.zone * 0.5;
    particules.emettre(j.x + Math.sin(angle) * devant, 0.9, j.z + Math.cos(angle) * devant, GERBES.heurt);
    retours.secousse(0.12);
    return true;
  }
}
