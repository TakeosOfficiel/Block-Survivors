/** Projectiles tirés par les ennemis (flèches) : pool, vol en ligne droite, collision avec le joueur. */
import { CONFIG } from '../config';
import { JOUEUR_BASE } from '../donnees/joueur';
import { IDS_PROJECTILES, PROJECTILES, type IdProjectile } from '../donnees/projectiles';
import type { DefinitionProjectile } from '../donnees/types';
import { RenduProjectiles } from '../effets/renduProjectiles';
import type { EtatPartie } from '../etat';
import type { CarteMonde } from '../monde/carte';
import { Pool } from '../utils/pool';
import type { CibleJoueur, SourceEntites, Tireur } from './types';

interface ProjectileEnnemi {
  indexRendu: number;
  definition: DefinitionProjectile;
  x: number;
  z: number;
  vx: number;
  vz: number;
  vie: number;
  degats: number;
}

export class ProjectilesEnnemis implements SourceEntites, Tireur {
  private readonly etat: EtatPartie;
  private readonly carte: CarteMonde;
  private readonly cible: CibleJoueur;
  private readonly pool = new Pool<ProjectileEnnemi>(CONFIG.projectiles.capacite, () => ({
    indexRendu: 0, definition: PROJECTILES[IDS_PROJECTILES[0]], x: 0, z: 0, vx: 0, vz: 0, vie: 0, degats: 0,
  }));

  constructor(etat: EtatPartie, carte: CarteMonde, cible: CibleJoueur) {
    this.etat = etat;
    this.carte = carte;
    this.cible = cible;
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  tirer(type: IdProjectile, x: number, z: number, dirX: number, dirZ: number, degats: number): void {
    const p = this.pool.acquerir();
    if (!p) return;
    const def = PROJECTILES[type];
    p.indexRendu = RenduProjectiles.indexDe(type);
    p.definition = def;
    p.x = x;
    p.z = z;
    p.vx = dirX * def.vitesse;
    p.vz = dirZ * def.vitesse;
    p.vie = def.duree;
    p.degats = degats;
  }

  maj(dt: number): void {
    const j = this.etat.joueur;
    const limite = this.etat.arene.taille / 2;
    const { pasCollision, rayonDecor } = CONFIG.projectiles;
    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const p = this.pool.elements[i];
      p.vie -= dt;

      // Par pas courts, le décor avant le joueur : une flèche ne frappe jamais à travers un mur
      const portee = p.definition.rayon + JOUEUR_BASE.rayon;
      const nbPas = Math.max(1, Math.ceil(Math.hypot(p.vx, p.vz) * dt / pasCollision));
      let fini = p.vie <= 0;
      for (let k = 0; k < nbPas && !fini; k++) {
        p.x += (p.vx * dt) / nbPas;
        p.z += (p.vz * dt) / nbPas;
        const sortie = p.x < -limite || p.x > limite || p.z < -limite || p.z > limite;
        if (sortie || this.carte.toucheTir(p.x, p.z, rayonDecor)) {
          fini = true;
          break;
        }
        const dx = p.x - j.x;
        const dz = p.z - j.z;
        if (!j.mort && dx * dx + dz * dz < portee * portee) {
          this.cible.subirDegats(p.degats);
          fini = true;
        }
      }
      if (fini) this.pool.liberer(i);
    }
  }

  majVisuel(rendu: RenduProjectiles): void {
    for (let i = 0; i < this.pool.nombre; i++) {
      const p = this.pool.elements[i];
      rendu.ajouter(p.indexRendu, p.x, p.definition.hauteur, p.z, Math.atan2(p.vx, p.vz));
    }
  }
}
