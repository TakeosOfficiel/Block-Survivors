/**
 * Projectiles tirés par les ennemis (flèches) : pool, vol en ligne droite,
 * collision avec le joueur. Un coup de bouclier peut les renvoyer : ils
 * blessent alors les ennemis.
 */
import { CONFIG } from '../config';
import { JOUEUR_BASE } from '../donnees/joueur';
import { IDS_PROJECTILES, PROJECTILES, type IdProjectile } from '../donnees/projectiles';
import type { DefinitionProjectile } from '../donnees/types';
import { RenduProjectiles } from '../effets/renduProjectiles';
import type { EtatPartie } from '../etat';
import type { CarteMonde } from '../monde/carte';
import { ecartAngle, longueur } from '../utils/maths';
import type { Ennemis } from './ennemis';
import { Pool } from '../utils/pool';
import type { CibleJoueur, EquipeJoueurs, SourceEntites, Tireur } from './types';

interface ProjectileEnnemi {
  indexRendu: number;
  definition: DefinitionProjectile;
  x: number;
  z: number;
  vx: number;
  vz: number;
  vie: number;
  degats: number;
  /** Tiré par un boss. */
  boss: boolean;
  /** Renvoyé par le joueur : il blesse les ennemis, au nom de l'arme `arme`. */
  renvoye: boolean;
  arme: number;
}

export class ProjectilesEnnemis implements SourceEntites, Tireur {
  private readonly etat: EtatPartie;
  private readonly carte: CarteMonde;
  private readonly cible: CibleJoueur;
  /** Public pour le multijoueur : l'hôte le lit pour les instantanés. */
  readonly pool = new Pool<ProjectileEnnemi>(CONFIG.projectiles.capacite, () => ({
    indexRendu: 0, definition: PROJECTILES[IDS_PROJECTILES[0]], x: 0, z: 0, vx: 0, vz: 0, vie: 0, degats: 0, boss: false, renvoye: false, arme: -1,
  }));
  private ennemis: Ennemis | null = null;
  private readonly touches = new Int32Array(4);
  /** Multijoueur (hôte) : les flèches touchent n'importe quel joueur de l'équipe. */
  equipe: EquipeJoueurs | null = null;

  constructor(etat: EtatPartie, carte: CarteMonde, cible: CibleJoueur) {
    this.etat = etat;
    this.carte = carte;
    this.cible = cible;
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  /** Les projectiles renvoyés blessent ces ennemis. */
  definirEnnemis(ennemis: Ennemis): void {
    this.ennemis = ennemis;
  }

  /**
   * Renvoie les projectiles du cône (sommet en x, z ; demi-angle en rad ;
   * longueur `portee`) : ils repartent vers l'avant et blessent les ennemis
   * de `degats`. Renvoie leur nombre.
   */
  renvoyer(x: number, z: number, angle: number, demiAngle: number, portee: number, degats: number, arme: number): number {
    let n = 0;
    for (let i = 0; i < this.pool.nombre; i++) {
      const p = this.pool.elements[i];
      const dx = p.x - x;
      const dz = p.z - z;
      const d = longueur(dx, dz);
      if (p.renvoye || d > portee || (d > 0.3 && Math.abs(ecartAngle(angle, Math.atan2(dx, dz))) > demiAngle)) continue;
      const vitesse = p.definition.vitesse * 1.2;
      p.vx = Math.sin(angle) * vitesse;
      p.vz = Math.cos(angle) * vitesse;
      p.vie = p.definition.duree;
      p.degats = degats;
      p.renvoye = true;
      p.arme = arme;
      n++;
    }
    return n;
  }

  tirer(type: IdProjectile, x: number, z: number, dirX: number, dirZ: number, degats: number, boss = false): void {
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
    p.boss = boss;
    p.renvoye = false;
    p.arme = -1;
  }

  maj(dt: number): void {
    const j = this.etat.joueur;
    const limiteX = this.etat.arene.largeur / 2;
    const limiteZ = this.etat.arene.profondeur / 2;
    const { pasCollision, rayonDecor } = CONFIG.projectiles;
    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const p = this.pool.elements[i];
      p.vie -= dt;

      // Par pas courts, le décor avant la cible : un projectile ne frappe jamais à travers un mur
      const portee = p.definition.rayon + JOUEUR_BASE.rayon;
      const nbPas = Math.max(1, Math.ceil(Math.hypot(p.vx, p.vz) * dt / pasCollision));
      let fini = p.vie <= 0;
      for (let k = 0; k < nbPas && !fini; k++) {
        p.x += (p.vx * dt) / nbPas;
        p.z += (p.vz * dt) / nbPas;
        const sortie = p.x < -limiteX || p.x > limiteX || p.z < -limiteZ || p.z > limiteZ;
        if (sortie || this.carte.toucheTir(p.x, p.z, rayonDecor)) {
          fini = true;
          break;
        }
        if (p.renvoye) {
          // Renvoyé : blesse le premier ennemi rencontré
          if (this.ennemis && this.ennemis.chercher(p.x, p.z, p.definition.rayon, this.touches) > 0) {
            this.ennemis.blesser(this.ennemis.obtenir(this.touches[0]), p.degats, p.x - p.vx, p.z - p.vz, 4, p.arme);
            fini = true;
          }
        } else {
          const equipe = this.equipe;
          for (let m = 0; m < (equipe ? equipe.nombre : 1) && !fini; m++) {
            const cible = equipe ? equipe.joueur(m) : j;
            const dx = p.x - cible.x;
            const dz = p.z - cible.z;
            if (!cible.mort && dx * dx + dz * dz < portee * portee) {
              (equipe ? equipe.contexte(m).cible : this.cible).subirDegats(p.degats, p.boss);
              fini = true;
            }
          }
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
