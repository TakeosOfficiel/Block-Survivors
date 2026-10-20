/**
 * Projectiles des armes du joueur : pool unique pour les mécaniques
 * « tir » (ligne droite, perforation), « retour » (aller puis retour au
 * joueur, traverse tout) et « explosif » (explose au premier contact).
 */
import { CONFIG } from '../config';
import { GERBES } from '../donnees/particules';
import { IDS_PROJECTILES, PROJECTILES, type IdProjectile } from '../donnees/projectiles';
import type { DefinitionProjectile } from '../donnees/types';
import { RenduProjectiles } from '../effets/renduProjectiles';
import type { SourceEntites } from '../entites/types';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';
import { longueur } from '../utils/maths';

const MAX_TOUCHES = 32;

/** `ligne` : s'arrête après `perforation` ennemis ; `traversant` : traverse tout ; `retour` : traverse tout et revient. */
export type ModeProjectile = 'ligne' | 'traversant' | 'retour';

interface ProjectileJoueur {
  indexRendu: number;
  indexArme: number;        // crédite les dégâts à l'arme dans les statistiques
  definition: DefinitionProjectile;
  retour: boolean;          // mode « retour »
  revient: boolean;         // phase de retour en cours
  x: number;
  z: number;
  dirX: number;
  dirZ: number;
  vitesse: number;
  parcouru: number;
  portee: number;
  degats: number;
  recul: number;
  restants: number;         // ennemis encore traversables
  zone: number;             // rayon d'explosion (0 = n'explose pas)
  touches: Int32Array;      // id des ennemis déjà touchés (pendant cette phase)
  nbTouches: number;
  trainee: number;          // s avant la prochaine particule de traînée
}

export class ProjectilesJoueur implements SourceEntites {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(64);
  private readonly pool = new Pool<ProjectileJoueur>(CONFIG.projectiles.capacite, () => ({
    indexRendu: 0, indexArme: 0, definition: PROJECTILES[IDS_PROJECTILES[0]], retour: false, revient: false,
    x: 0, z: 0, dirX: 0, dirZ: 1, vitesse: 0, parcouru: 0, portee: 0, degats: 0, recul: 0,
    restants: 1, zone: 0, touches: new Int32Array(MAX_TOUCHES), nbTouches: 0, trainee: 0,
  }));

  constructor(contexte: ContexteCombat) {
    this.contexte = contexte;
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  /** Lance un projectile depuis le joueur dans la direction (dirX, dirZ) unitaire. */
  lancer(arme: ArmeEquipee, type: IdProjectile, mode: ModeProjectile, dirX: number, dirZ: number, explose = false): void {
    const p = this.pool.acquerir();
    if (!p) return;
    const j = this.contexte.etat.joueur;
    const s = arme.stats;
    p.indexRendu = RenduProjectiles.indexDe(type);
    p.definition = PROJECTILES[type];
    p.retour = mode === 'retour';
    p.revient = false;
    p.x = j.x;
    p.z = j.z;
    p.dirX = dirX;
    p.dirZ = dirZ;
    p.vitesse = s.vitesse;
    p.parcouru = 0;
    p.portee = s.portee;
    p.degats = s.degats;
    p.indexArme = arme.index;
    p.recul = s.recul;
    p.restants = mode === 'ligne' ? Math.max(1, s.perforation) : Infinity;
    p.zone = explose ? s.zone : 0;
    p.nbTouches = 0;
    p.trainee = 0;
  }

  maj(dt: number): void {
    const { etat, ennemis, particules, carte } = this.contexte;
    const { pasCollision, rayonDecor } = CONFIG.projectiles;
    const j = etat.joueur;
    const limite = etat.arene.taille / 2;

    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const p = this.pool.elements[i];

      // Retour : se dirige vers le joueur et disparaît en l'atteignant
      if (p.revient) {
        const dx = j.x - p.x;
        const dz = j.z - p.z;
        const d = longueur(dx, dz);
        if (d < CONFIG.armes.distanceRetour) {
          this.pool.liberer(i);
          continue;
        }
        p.dirX = dx / d;
        p.dirZ = dz / d;
      }

      // Par pas courts, le décor avant les ennemis : aucun tir ne touche à travers un mur
      // (le trident, magique, traverse le décor)
      const pas = p.vitesse * dt;
      const nbPas = Math.max(1, Math.ceil(pas / pasCollision));
      let detruit = false;
      let sortie = false;
      for (let s = 0; s < nbPas && !detruit && !sortie; s++) {
        p.x += (p.dirX * pas) / nbPas;
        p.z += (p.dirZ * pas) / nbPas;
        p.parcouru += pas / nbPas;
        sortie = p.x < -limite || p.x > limite || p.z < -limite || p.z > limite
          || (!p.retour && carte.toucheTir(p.x, p.z, rayonDecor));
        if (sortie) break;

        // Collisions avec les ennemis
        const n = ennemis.chercher(p.x, p.z, p.definition.rayon, this.tampon);
        for (let k = 0; k < n && !detruit; k++) {
          const e = ennemis.obtenir(this.tampon[k]);
          if (this.dejaTouche(p, e.id)) continue;
          if (p.zone > 0) {
            this.exploser(p);
            detruit = true;
            break;
          }
          ennemis.blesser(e, p.degats, p.x - p.dirX, p.z - p.dirZ, p.recul, p.indexArme);
          if (p.nbTouches < MAX_TOUCHES) p.touches[p.nbTouches++] = e.id;
          if (--p.restants <= 0) detruit = true;
        }
      }

      // Traînée de particules
      const trainee = p.definition.trainee;
      if (trainee && (p.trainee -= dt) <= 0) {
        p.trainee = CONFIG.projectiles.intervalleTrainee;
        particules.emettre(p.x, p.definition.hauteur, p.z, GERBES[trainee]);
      }

      // Fin de course
      if (!detruit && (p.parcouru >= p.portee || sortie)) {
        if (p.retour) {
          if (!p.revient) {
            p.revient = true;
            p.nbTouches = 0;  // peut retoucher les mêmes ennemis au retour
          }
        } else {
          if (p.zone > 0) this.exploser(p);
          detruit = true;
        }
      }

      if (detruit) this.pool.liberer(i);
    }
  }

  majVisuel(rendu: RenduProjectiles, temps: number): void {
    for (let i = 0; i < this.pool.nombre; i++) {
      const p = this.pool.elements[i];
      const rotation = p.definition.rotationPropre;
      const angle = rotation ? temps * rotation : Math.atan2(p.dirX, p.dirZ);
      rendu.ajouter(p.indexRendu, p.x, p.definition.hauteur, p.z, angle);
    }
  }

  private dejaTouche(p: ProjectileJoueur, id: number): boolean {
    for (let k = 0; k < p.nbTouches; k++) if (p.touches[k] === id) return true;
    return false;
  }

  private exploser(p: ProjectileJoueur): void {
    const { ennemis, particules } = this.contexte;
    const n = ennemis.chercher(p.x, p.z, p.zone, this.tampon);
    for (let k = 0; k < n; k++) {
      ennemis.blesser(ennemis.obtenir(this.tampon[k]), p.degats, p.x, p.z, p.recul, p.indexArme);
    }
    particules.emettre(p.x, p.definition.hauteur, p.z, GERBES.explosionArme);
    this.contexte.retours.son('explosion', 1.3);
  }
}
