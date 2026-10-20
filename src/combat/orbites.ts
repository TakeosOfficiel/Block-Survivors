/**
 * Mécanique « orbite » (haches tournoyantes) : des objets tournent en
 * permanence autour du joueur et blessent ce qu'ils traversent, avec un délai
 * avant de pouvoir retoucher le même ennemi.
 */
import { IDS_ARMES, ARMES } from '../donnees/armes';
import { PROJECTILES } from '../donnees/projectiles';
import { RenduProjectiles } from '../effets/renduProjectiles';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

const TOUR = Math.PI * 2;

export class Orbites {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(64);
  /** Angle courant de l'orbite, par index d'arme. */
  private readonly angles = new Float32Array(IDS_ARMES.length);
  /** Index de rendu du projectile affiché, par index d'arme (−1 si l'arme n'orbite pas). */
  private readonly rendus: Int32Array;

  constructor(contexte: ContexteCombat) {
    this.contexte = contexte;
    this.rendus = Int32Array.from(IDS_ARMES, (id) => {
      const m = ARMES[id].mecanique;
      return m.type === 'orbite' ? RenduProjectiles.indexDe(m.projectile) : -1;
    });
  }

  maj(arme: ArmeEquipee, dt: number): void {
    const m = arme.definition.mecanique;
    if (m.type !== 'orbite') return;
    const { etat, ennemis } = this.contexte;
    const j = etat.joueur;
    const s = arme.stats;
    const rayon = PROJECTILES[m.projectile].rayon;

    const angle = (this.angles[arme.index] + s.vitesse * dt) % TOUR;
    this.angles[arme.index] = angle;
    for (let k = 0; k < s.nombre; k++) {
      const a = angle + (k * TOUR) / s.nombre;
      const x = j.x + Math.sin(a) * s.zone;
      const z = j.z + Math.cos(a) * s.zone;
      const n = ennemis.chercher(x, z, rayon, this.tampon);
      for (let i = 0; i < n; i++) {
        ennemis.blesserAvecDelai(ennemis.obtenir(this.tampon[i]), arme.index, s.cadence, s.degats, j.x, j.z, s.recul);
      }
    }
  }

  majVisuel(armes: readonly ArmeEquipee[], rendu: RenduProjectiles, temps: number): void {
    const j = this.contexte.etat.joueur;
    if (j.mort) return;
    for (let i = 0; i < armes.length; i++) {
      const arme = armes[i];
      const indexRendu = this.rendus[arme.index];
      if (indexRendu < 0) continue;
      const m = arme.definition.mecanique;
      if (m.type !== 'orbite') continue;
      const def = PROJECTILES[m.projectile];
      const s = arme.stats;
      const rotation = temps * (def.rotationPropre ?? 0);
      for (let k = 0; k < s.nombre; k++) {
        const a = this.angles[arme.index] + (k * TOUR) / s.nombre;
        rendu.ajouter(indexRendu, j.x + Math.sin(a) * s.zone, def.hauteur, j.z + Math.cos(a) * s.zone, rotation);
      }
    }
  }
}
