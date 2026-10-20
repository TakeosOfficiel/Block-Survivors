/** Projectiles tirés par les ennemis (flèches) : pool, vol en ligne droite, collision avec le joueur. */
import { CONFIG } from '../config';
import { JOUEUR_BASE } from '../donnees/joueur';
import { IDS_PROJECTILES, PROJECTILES, type IdProjectile } from '../donnees/projectiles';
import type { DefinitionProjectile } from '../donnees/types';
import { RenduProjectiles } from '../effets/renduProjectiles';
import type { EtatPartie } from '../etat';
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
  private readonly cible: CibleJoueur;
  private readonly pool = new Pool<ProjectileEnnemi>(CONFIG.projectiles.capacite, () => ({
    indexRendu: 0, definition: PROJECTILES[IDS_PROJECTILES[0]], x: 0, z: 0, vx: 0, vz: 0, vie: 0, degats: 0,
  }));

  constructor(etat: EtatPartie, cible: CibleJoueur) {
    this.etat = etat;
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
    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const p = this.pool.elements[i];
      p.vie -= dt;
      p.x += p.vx * dt;
      p.z += p.vz * dt;

      const dx = p.x - j.x;
      const dz = p.z - j.z;
      const portee = p.definition.rayon + JOUEUR_BASE.rayon;
      const touche = !j.mort && dx * dx + dz * dz < portee * portee;
      if (touche) this.cible.subirDegats(p.degats);

      const sortie = p.x < -limite || p.x > limite || p.z < -limite || p.z > limite;
      if (touche || sortie || p.vie <= 0) this.pool.liberer(i);
    }
  }

  majVisuel(rendu: RenduProjectiles): void {
    for (let i = 0; i < this.pool.nombre; i++) {
      const p = this.pool.elements[i];
      rendu.ajouter(p.indexRendu, p.x, p.definition.hauteur, p.z, Math.atan2(p.vx, p.vz));
    }
  }
}
