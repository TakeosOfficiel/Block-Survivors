/**
 * Projectiles des armes du joueur : pool unique pour les mécaniques
 * « tir » (ligne droite, perforation), « retour » (aller puis retour au
 * joueur, traverse tout) et « explosif » (explose au premier contact).
 */
import { CONFIG } from '../config';
import { GERBES } from '../donnees/particules';
import { IDS_PROJECTILES, PROJECTILES, type IdProjectile } from '../donnees/projectiles';
import type { IdSon } from '../donnees/sons';
import type { DefinitionGerbe, DefinitionProjectile } from '../donnees/types';
import { RenduProjectiles } from '../effets/renduProjectiles';
import type { NatureCoup } from '../entites/ennemis';
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
  perte: number;            // part des dégâts perdue à chaque ennemi traversé
  visee: boolean;           // Visée stable : plus de dégâts au-delà d'une distance
  zone: number;             // rayon d'explosion (0 = n'explose pas)
  gerbe: DefinitionGerbe;   // particules et son de l'explosion
  sonExplosion: IdSon;
  eclats: number;           // petites explosions lancées autour à l'explosion
  brulure: number;          // dégâts par seconde de la brûlure infligée (0 : aucune)
  nature: NatureCoup;       // tiré par le joueur ou par un allié (marque)
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
    restants: 1, perte: 0, visee: false, zone: 0, gerbe: GERBES.explosionArme, sonExplosion: 'explosion', eclats: 0, brulure: 0, nature: 'arme',
    touches: new Int32Array(MAX_TOUCHES), nbTouches: 0, trainee: 0,
  }));

  constructor(contexte: ContexteCombat) {
    this.contexte = contexte;
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  /**
   * Lance un projectile dans la direction (dirX, dirZ) unitaire, depuis le
   * joueur ou depuis (origineX, origineZ) pour un allié qui tire. `brulure` :
   * dégâts par seconde d'une brûlure infligée à chaque ennemi touché.
   */
  lancer(
    arme: ArmeEquipee, type: IdProjectile, mode: ModeProjectile, dirX: number, dirZ: number, explose = false,
    origineX = this.contexte.etat.joueur.x, origineZ = this.contexte.etat.joueur.z, brulure = 0, nature: NatureCoup = 'arme',
  ): void {
    const p = this.pool.acquerir();
    if (!p) return;
    const s = arme.stats;
    p.indexRendu = RenduProjectiles.indexDe(type);
    p.definition = PROJECTILES[type];
    p.retour = mode === 'retour';
    p.revient = false;
    p.x = origineX;
    p.z = origineZ;
    p.dirX = dirX;
    p.dirZ = dirZ;
    p.vitesse = s.vitesse;
    p.parcouru = 0;
    p.portee = s.portee;
    p.degats = s.degats * this.contexte.talents.facteurDeclenchement;
    p.indexArme = arme.index;
    p.recul = s.recul;
    // Une flèche perd une part de ses dégâts à chaque ennemi traversé (moins avec Perforation nette)
    const talents = this.contexte.talents;
    const c = CONFIG.talents.perforationNette;
    const fleche = mode === 'ligne' && arme.definition.famille === 'projectile' && nature === 'arme';
    const nette = talents.rang('perforation_nette');
    p.perte = !fleche ? 0 : nette > 0 ? talents.valeur('perforation_nette', c.pertes) : CONFIG.projectiles.perteTraversee;
    p.restants = mode === 'ligne' ? Math.max(1, s.perforation) + (fleche && nette >= c.rangPerforation ? 1 : 0) : Infinity;
    p.visee = arme.id === 'arc' && talents.rang('visee_stable') > 0;
    p.zone = explose ? s.zone : 0;
    const m = arme.definition.mecanique;
    p.gerbe = m.type === 'explosif' && m.gerbe ? GERBES[m.gerbe] : GERBES.explosionArme;
    p.sonExplosion = m.type === 'explosif' && m.son ? m.son : 'explosion';
    p.eclats = m.type === 'explosif' && m.eclatsAuRang !== undefined && arme.rang >= m.eclatsAuRang ? CONFIG.projectiles.eclats.nombre : 0;
    p.nbTouches = 0;
    p.trainee = 0;
    p.brulure = brulure;
    p.nature = nature;
  }

  /** Flèche spéciale d'un talent (Puissance) : traverse tout sur `portee` blocs. */
  lancerSpecial(arme: ArmeEquipee, type: IdProjectile, dirX: number, dirZ: number, degats: number, portee: number): void {
    this.lancer(arme, type, 'traversant', dirX, dirZ);
    const p = this.pool.elements[this.pool.nombre - 1];
    if (!p || p.definition !== PROJECTILES[type]) return;
    p.degats = degats;
    p.portee = portee;
    p.vitesse = PROJECTILES[type].vitesse;
    p.perte = 0;
    p.visee = false;
  }

  maj(dt: number): void {
    const { etat, ennemis, particules, carte } = this.contexte;
    const { pasCollision, rayonDecor } = CONFIG.projectiles;
    const j = etat.joueur;
    const limiteX = etat.arene.largeur / 2;
    const limiteZ = etat.arene.profondeur / 2;

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
        sortie = p.x < -limiteX || p.x > limiteX || p.z < -limiteZ || p.z > limiteZ
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
          const v = CONFIG.talents.viseeStable;
          const degats = p.visee && p.parcouru > v.distance ? p.degats * (1 + v.bonus) : p.degats;
          ennemis.blesser(e, degats, p.x - p.dirX, p.z - p.dirZ, p.recul, p.indexArme, p.nature);
          p.degats *= 1 - p.perte;
          if (p.brulure > 0) ennemis.bruler(e, 1, p.brulure, CONFIG.armes.brulureTirs, p.indexArme);
          if (p.nbTouches < MAX_TOUCHES) p.touches[p.nbTouches++] = e.id;
          if (--p.restants <= 0) detruit = true;
        }
      }

      // Traînée de particules
      const trainee = p.definition.trainee;
      if (trainee && (p.trainee -= dt) <= 0) {
        p.trainee = CONFIG.projectiles.intervalleTrainee;
        const recul = p.definition.decalageTrainee ?? 0;
        particules.emettre(p.x - p.dirX * recul, p.definition.hauteur, p.z - p.dirZ * recul, GERBES[trainee]);
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
    const particules = this.contexte.particules;
    this.blesserZone(p.x, p.z, p.zone, p.degats, p.recul, p.indexArme);
    particules.emettre(p.x, p.definition.hauteur, p.z, p.gerbe);
    this.contexte.retours.son(p.sonExplosion, 1.3);
    // Éclats : petites explosions tout autour (dernier rang de la Fusée explosive)
    const e = CONFIG.projectiles.eclats;
    for (let k = 0; k < p.eclats; k++) {
      const a = (k / p.eclats) * Math.PI * 2 + Math.random();
      const x = p.x + Math.sin(a) * e.distance;
      const z = p.z + Math.cos(a) * e.distance;
      this.blesserZone(x, z, p.zone * e.rayon, p.degats * e.degats, p.recul * 0.5, p.indexArme);
      particules.emettre(x, p.definition.hauteur, z, GERBES.petitArtifice);
    }
  }

  private blesserZone(x: number, z: number, rayon: number, degats: number, recul: number, arme: number): void {
    const ennemis = this.contexte.ennemis;
    const n = ennemis.chercher(x, z, rayon, this.tampon);
    for (let k = 0; k < n; k++) ennemis.blesser(ennemis.obtenir(this.tampon[k]), degats, x, z, recul, arme);
  }
}
