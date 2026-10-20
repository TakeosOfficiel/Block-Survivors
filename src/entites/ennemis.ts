/**
 * Ennemis : pool unique, déplacement générique (comportement + recul +
 * séparation), dégâts, mort et explosion. Rendu : un ModeleInstancie par type.
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ENNEMIS, IDS_ENNEMIS, type IdEnnemi } from '../donnees/ennemis';
import { JOUEUR_BASE } from '../donnees/joueur';
import { MODELES } from '../donnees/modeles';
import type { Ombres } from '../effets/ombres';
import type { Particules } from '../effets/particules';
import type { EtatPartie } from '../etat';
import { angleChute, avancerAnimation, calculerRotations, reinitialiserAnimation } from '../modeles/animation';
import { compilerModele, type ModeleCompile } from '../modeles/compilation';
import { creerPoseModele, ModeleInstancie, type PoseModele } from '../modeles/rendu';
import { GrilleSpatiale } from '../utils/grilleSpatiale';
import { borner, ecartAngle, facteurLissage } from '../utils/maths';
import { Pool } from '../utils/pool';
import { appliquerComportement, type ContexteComportement } from './comportements';
import { creerEnnemi, type Ennemi } from './ennemi';
import type { DepotXp } from './orbes';
import type { SourceEntites } from './types';

/** Tout ce qu'il faut pour afficher un type d'ennemi. */
interface RenduType {
  readonly modele: ModeleCompile;
  readonly instances: ModeleInstancie;
  readonly pose: PoseModele;
  readonly couleursMort: readonly number[];
}

const RAYON_MAX = Math.max(...IDS_ENNEMIS.map((id) => ENNEMIS[id].rayon));

export class Ennemis implements SourceEntites {
  private readonly etat: EtatPartie;
  private readonly contexte: ContexteComportement;
  private readonly particules: Particules;
  private readonly depotXp: DepotXp;
  private readonly pool = new Pool<Ennemi>(CONFIG.ennemis.capacite, creerEnnemi);
  private readonly grille: GrilleSpatiale;
  private readonly voisins = new Int32Array(64);
  private readonly rendus: RenduType[];
  private prochainId = 1;

  constructor(
    etat: EtatPartie,
    contexte: ContexteComportement,
    particules: Particules,
    depotXp: DepotXp,
    scene: THREE.Scene,
    textures: BibliothequeTextures,
  ) {
    this.etat = etat;
    this.contexte = contexte;
    this.particules = particules;
    this.depotXp = depotXp;
    this.grille = new GrilleSpatiale(etat.arene.taille / 2 + 1, CONFIG.ennemis.celluleGrille, CONFIG.ennemis.capacite);
    this.rendus = IDS_ENNEMIS.map((id) => {
      const def = ENNEMIS[id];
      const modele = compilerModele(MODELES[def.modele]);
      return {
        modele,
        instances: new ModeleInstancie(modele, def, textures.obtenir(def.skin), CONFIG.ennemis.capacite, scene),
        pose: creerPoseModele(modele),
        couleursMort: [0xffffff, 0xd8d8d8, def.couleurParticules],
      };
    });
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  /** Fait apparaître un ennemi ; renvoie faux si le pool est plein. */
  apparaitre(id: IdEnnemi, x: number, z: number, multiplicateurPv = 1): boolean {
    const e = this.pool.acquerir();
    if (!e) return false;
    const def = ENNEMIS[id];
    const limite = this.etat.arene.taille / 2 - def.rayon;
    e.id = this.prochainId++;
    e.indexType = IDS_ENNEMIS.indexOf(id);
    e.definition = def;
    e.x = borner(x, -limite, limite);
    e.z = borner(z, -limite, limite);
    e.vx = 0;
    e.vz = 0;
    e.dirX = 0;
    e.dirZ = 0;
    e.angle = Math.atan2(this.etat.joueur.x - e.x, this.etat.joueur.z - e.z);
    e.vitesseSol = 0;
    e.pvMax = Math.round(def.pv * multiplicateurPv);
    e.pv = e.pvMax;
    e.minuterie = 0.5;
    e.blessure = 0;
    e.meche = 0;
    e.mort = -1;
    e.aExploser = false;
    e.immunites.fill(0);
    reinitialiserAnimation(e.animation);
    return true;
  }

  /** Ennemi actif à un index renvoyé par chercher(). */
  obtenir(index: number): Ennemi {
    return this.pool.elements[index];
  }

  /**
   * Écrit dans `sortie` les index des ennemis vivants qui touchent le cercle
   * (leur propre rayon compris) ; renvoie leur nombre.
   */
  chercher(x: number, z: number, rayon: number, sortie: Int32Array): number {
    const candidats = this.grille.requeter(x, z, rayon + RAYON_MAX, this.voisins);
    let n = 0;
    for (let k = 0; k < candidats && n < sortie.length; k++) {
      const i = this.voisins[k];
      const e = this.pool.elements[i];
      const dx = e.x - x;
      const dz = e.z - z;
      const portee = rayon + e.definition.rayon;
      if (e.mort < 0 && dx * dx + dz * dz <= portee * portee) sortie[n++] = i;
    }
    return n;
  }

  /** Ennemi vivant le plus proche à moins de `portee`, ou null. */
  plusProche(x: number, z: number, portee: number): Ennemi | null {
    let meilleur: Ennemi | null = null;
    let meilleureD2 = portee * portee;
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      if (e.mort >= 0) continue;
      const d2 = (e.x - x) ** 2 + (e.z - z) ** 2;
      if (d2 < meilleureD2) {
        meilleureD2 = d2;
        meilleur = e;
      }
    }
    return meilleur;
  }

  /** Ennemi vivant tiré au hasard à moins de `portee`, ou null (tirage uniforme en un passage). */
  auHasard(x: number, z: number, portee: number): Ennemi | null {
    let choisi: Ennemi | null = null;
    let vus = 0;
    const p2 = portee * portee;
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      if (e.mort >= 0 || (e.x - x) ** 2 + (e.z - z) ** 2 > p2) continue;
      if (Math.random() * ++vus < 1) choisi = e;
    }
    return choisi;
  }

  /** Blesse l'ennemi si l'arme `source` ne l'a pas touché depuis moins de `delai` secondes. */
  blesserAvecDelai(e: Ennemi, source: number, delai: number, degats: number, sourceX: number, sourceZ: number, recul: number): boolean {
    if (e.mort >= 0 || e.immunites[source] > this.etat.tempsJeu) return false;
    e.immunites[source] = this.etat.tempsJeu + delai;
    this.blesser(e, degats, sourceX, sourceZ, recul);
    return true;
  }

  /** Blesse l'ennemi et le repousse depuis (sourceX, sourceZ). */
  blesser(e: Ennemi, degats: number, sourceX: number, sourceZ: number, recul: number): void {
    if (e.mort >= 0) return;
    e.pv -= degats;
    e.blessure = CONFIG.surcouche.blessure.duree;
    const dx = e.x - sourceX;
    const dz = e.z - sourceZ;
    const d = Math.hypot(dx, dz) || 1;
    e.vx += (dx / d) * recul;
    e.vz += (dz / d) * recul;
    if (e.pv <= 0) {
      e.mort = 0;
      e.meche = 0;
      this.etat.ennemisTues++;
      this.depotXp.deposer(e.x, e.z, e.definition.xp);
    }
  }

  /** Blesse tous les ennemis vivants dans un cercle. */
  frapperZone(x: number, z: number, rayon: number, degats: number, recul: number): void {
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      const dx = e.x - x;
      const dz = e.z - z;
      if (e.mort < 0 && dx * dx + dz * dz <= (rayon + e.definition.rayon) ** 2) this.blesser(e, degats, x, z, recul);
    }
  }

  maj(dt: number): void {
    const n = this.pool.nombre;
    const elements = this.pool.elements;

    // 1. Comportement et déplacement (la grille date de la fin du pas précédent :
    //    ses index sont valides, seuls des ajouts ont pu avoir lieu depuis)
    for (let i = 0; i < n; i++) {
      const e = elements[i];
      if (e.mort >= 0) {
        e.mort += dt;
        e.vitesseSol = 0;
        continue;
      }
      this.deplacer(e, i, dt);
    }

    // 2. Retrait des ennemis explosés ou morts depuis assez longtemps
    for (let i = n - 1; i >= 0; i--) {
      const e = elements[i];
      if (e.aExploser) {
        this.exploser(e);
        this.pool.liberer(i);
      } else if (e.mort >= CONFIG.ennemis.dureeMort) {
        this.particules.emettre(e.x, 0.6, e.z, CONFIG.particules.mort, this.rendus[e.indexType].couleursMort);
        this.pool.liberer(i);
      }
    }

    // 3. Grille des ennemis vivants, utilisée par les armes et au pas suivant
    this.grille.vider();
    for (let i = 0; i < this.pool.nombre; i++) {
      if (elements[i].mort < 0) this.grille.inserer(i, elements[i].x, elements[i].z);
    }
  }

  majVisuel(ombres: Ombres): void {
    for (let t = 0; t < this.rendus.length; t++) this.rendus[t].instances.commencer();

    const blessure = CONFIG.surcouche.blessure;
    const flash = CONFIG.surcouche.flash;
    const g = CONFIG.ennemis;
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      const rendu = this.rendus[e.indexType];
      const pose = rendu.pose;
      pose.x = e.x;
      pose.z = e.z;
      pose.angle = e.angle;
      pose.inclinaison = e.mort >= 0 ? angleChute(e.mort, g.dureeMort) : 0;
      pose.echelleXZ = 1;
      pose.echelleY = 1;
      pose.surcouche[3] = 0;
      calculerRotations(rendu.modele, e.animation, pose.rotations);

      // Creeper : gonfle et clignote en blanc avant d'exploser (courbes du jeu)
      const c = e.definition.comportement;
      if (c.type === 'explosif' && e.meche > 0) {
        const progression = Math.min(e.meche / c.duree, 1);
        const tremblement = 1 + Math.sin(progression * 100) * progression * 0.01;
        const gonflement = progression ** 3;
        pose.echelleXZ = (1 + gonflement * g.gonflementLargeur) * tremblement;
        pose.echelleY = (1 + gonflement * g.gonflementHauteur) / tremblement;
        if (Math.floor(progression * 10) % 2 === 1) {
          pose.surcouche[0] = flash.couleur[0];
          pose.surcouche[1] = flash.couleur[1];
          pose.surcouche[2] = flash.couleur[2];
          pose.surcouche[3] = flash.force * borner(progression, 0.5, 1);
        }
      }

      // Rouge pendant la blessure et la mort
      if (e.blessure > 0 || e.mort >= 0) {
        pose.surcouche[0] = blessure.couleur[0];
        pose.surcouche[1] = blessure.couleur[1];
        pose.surcouche[2] = blessure.couleur[2];
        pose.surcouche[3] = blessure.force;
      }

      rendu.instances.ajouter(pose);
      ombres.ajouter(e.x, e.z, e.definition.rayonOmbre * pose.echelleXZ);
    }

    for (let t = 0; t < this.rendus.length; t++) this.rendus[t].instances.terminer();
  }

  // -----------------------------------------------------------------------

  private deplacer(e: Ennemi, index: number, dt: number): void {
    const g = CONFIG.ennemis;
    const j = this.etat.joueur;
    const def = e.definition;
    if (e.minuterie > 0) e.minuterie -= dt;
    if (e.blessure > 0) e.blessure -= dt;

    // Comportement (immobile si le joueur est mort)
    const versX = j.x - e.x;
    const versZ = j.z - e.z;
    const distance = Math.hypot(versX, versZ) || 1e-6;
    const dx = versX / distance;
    const dz = versZ / distance;
    if (j.mort) {
      e.dirX = 0;
      e.dirZ = 0;
    } else {
      e.aExploser = appliquerComportement(e, this.contexte, dt, dx, dz, distance);
    }

    // Séparation : repousse doucement les ennemis qui se chevauchent
    let sepX = 0;
    let sepZ = 0;
    const candidats = this.grille.requeter(e.x, e.z, def.rayon + RAYON_MAX, this.voisins);
    for (let k = 0; k < candidats; k++) {
      const autre = this.voisins[k];
      if (autre === index) continue;
      const o = this.pool.elements[autre];
      if (o.mort >= 0) continue;
      const ox = e.x - o.x;
      const oz = e.z - o.z;
      const minimum = def.rayon + o.definition.rayon;
      const d2 = ox * ox + oz * oz;
      if (d2 >= minimum * minimum || d2 < 1e-8) continue;
      const d = Math.sqrt(d2);
      sepX += (ox / d) * (minimum - d);
      sepZ += (oz / d) * (minimum - d);
    }
    const kSeparation = Math.min(1, g.separation * dt);

    // Intégration : marche voulue + recul amorti + séparation
    e.x += (e.dirX * def.vitesse + e.vx) * dt + sepX * kSeparation;
    e.z += (e.dirZ * def.vitesse + e.vz) * dt + sepZ * kSeparation;
    const amorti = 1 - facteurLissage(g.amortissementRecul, dt);
    e.vx *= amorti;
    e.vz *= amorti;

    // Jamais dans le joueur
    const px = e.x - j.x;
    const pz = e.z - j.z;
    const contact = def.rayon + JOUEUR_BASE.rayon;
    const dj2 = px * px + pz * pz;
    if (dj2 < contact * contact && dj2 > 1e-8) {
      const dj = Math.sqrt(dj2);
      e.x = j.x + (px / dj) * contact;
      e.z = j.z + (pz / dj) * contact;
    }

    // Bords de l'arène
    const limite = this.etat.arene.taille / 2 - def.rayon;
    e.x = borner(e.x, -limite, limite);
    e.z = borner(e.z, -limite, limite);

    // Toujours tourné vers le joueur ; animation selon la marche effective
    e.angle += ecartAngle(e.angle, Math.atan2(dx, dz)) * facteurLissage(g.vitesseRotation, dt);
    e.vitesseSol = Math.hypot(e.dirX, e.dirZ) * def.vitesse;
    avancerAnimation(e.animation, this.rendus[e.indexType].modele, dt, e.vitesseSol);
  }

  /** Explosion : particules et dégâts au joueur décroissants avec la distance. */
  private exploser(e: Ennemi): void {
    const c = e.definition.comportement;
    if (c.type !== 'explosif') return;
    const p = CONFIG.particules.explosion;
    this.particules.emettre(e.x, 0.8, e.z, p, p.couleurs);

    const j = this.etat.joueur;
    const distance = Math.hypot(j.x - e.x, j.z - e.z);
    if (distance < c.rayon) {
      this.contexte.cible.subirDegats(Math.ceil(c.degats * (1 - distance / c.rayon)));
    }
  }
}
