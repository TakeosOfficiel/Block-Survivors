/**
 * Ennemis dans les instantanés : ce que l'hôte écrit pour un joueur (ceux à
 * moins de `rayonEnvoi` de lui) et ce que le joueur en fait. Chez le joueur,
 * chaque ennemi de l'hôte (par son identifiant) occupe une place du pool local,
 * affiché par Ennemis.majVisuel et interpolé d'un instantané à l'autre.
 */
import { CONFIG } from '../config';
import { ENNEMIS, IDS_ENNEMIS } from '../donnees/ennemis';
import { MODELES } from '../donnees/modeles';
import { GERBES } from '../donnees/particules';
import type { Particules } from '../effets/particules';
import type { Retours } from '../effets/retours';
import { effacerEtats, type Ennemi } from '../entites/ennemi';
import type { Ennemis } from '../entites/ennemis';
import { avancerAnimation, reinitialiserAnimation } from '../modeles/animation';
import { compilerModele, type ModeleCompile } from '../modeles/compilation';
import { ecartAngle } from '../utils/maths';
import {
  ENNEMI_BLESSE, ENNEMI_BRULURE, ENNEMI_MORT, ENNEMI_POISON, ENNEMI_RALENTI, ENNEMI_SAIGNEMENT, type Ecrivain, type Lecteur,
} from './protocole';

/** Hôte : les ennemis à moins de √r2 de (cx, cz), tant qu'il reste de la place. */
export function ecrireEnnemis(w: Ecrivain, ennemis: Ennemis, cx: number, cz: number, r2: number): void {
  const compte = w.reserver16();
  let n = 0;
  const pool = ennemis.pool;
  for (let i = 0; i < pool.nombre && w.reste > 16 * 1024; i++) {
    const e = pool.elements[i];
    if ((e.x - cx) ** 2 + (e.z - cz) ** 2 > r2) continue;
    const comportement = e.definition.comportement;
    w.u32(e.id);
    w.u8(e.indexType);
    w.coordonnee(e.x);
    w.coordonnee(e.z);
    w.angle(e.angle);
    w.part(e.pvMax > 0 ? e.pv / e.pvMax : 0);
    w.u8((e.mort >= 0 ? ENNEMI_MORT : 0) | (e.blessure > 0 ? ENNEMI_BLESSE : 0) | (e.brulure.cumuls > 0 ? ENNEMI_BRULURE : 0)
      | (e.poison.cumuls > 0 ? ENNEMI_POISON : 0) | (e.saignement.cumuls > 0 ? ENNEMI_SAIGNEMENT : 0)
      | (e.ralentiTemps > 0 || e.enracine > 0 ? ENNEMI_RALENTI : 0));
    w.part(comportement.type === 'explosif' && e.meche > 0 ? e.meche / comportement.duree : 0);
    w.u8(e.mort >= 0 ? Math.min(255, Math.round(e.mort * 100)) : 0);
    w.u8(Math.min(255, Math.round(e.vitesseSol * 16)));
    n++;
  }
  w.ecrire16(compte, n);
}

/** Un ennemi affiché, d'après son identifiant chez l'hôte : ses positions aux deux derniers instantanés. */
interface SuiviEnnemi {
  readonly e: Ennemi;
  x0: number; z0: number; a0: number;
  x1: number; z1: number; a1: number;
  /** Numéro du dernier instantané qui le contenait. */
  vu: number;
}

/** Joueur : les ennemis reçus de l'hôte, dans le pool local. */
export class EnnemisRecus {
  private readonly ennemis: Ennemis;
  private readonly particules: Particules;
  private readonly retours: Retours;
  /** Suivi de chaque ennemi affiché, à la même place que lui dans le pool (échangés ensemble). */
  private readonly suivisPool: SuiviEnnemi[] = [];
  private readonly suivis = new Map<number, SuiviEnnemi>();
  private readonly modeles: (ModeleCompile | null)[] = IDS_ENNEMIS.map(() => null);

  constructor(ennemis: Ennemis, particules: Particules, retours: Retours) {
    this.ennemis = ennemis;
    this.particules = particules;
    this.retours = retours;
    for (let i = 0; i < CONFIG.ennemis.capacite; i++) {
      this.suivisPool.push({ e: ennemis.pool.elements[i], x0: 0, z0: 0, a0: 0, x1: 0, z1: 0, a1: 0, vu: 0 });
    }
  }

  /** Lit la section des ennemis de l'instantané `numero` ; ceux qui n'y sont plus disparaissent. */
  lire(l: Lecteur, numero: number): void {
    const n = l.u16();
    for (let i = 0; i < n; i++) {
      const id = l.u32();
      const type = l.u8();
      const x = l.coordonnee();
      const z = l.coordonnee();
      const angle = l.angle();
      const pv = l.part();
      const drapeaux = l.u8();
      const meche = l.part();
      const mort = l.u8() / 100;
      const vitesse = l.u8() / 16;
      const s = this.suivis.get(id) ?? this.nouveau(id, type, x, z, angle);
      if (!s) continue;
      const e = s.e;
      s.x0 = e.x;
      s.z0 = e.z;
      s.a0 = e.angle;
      s.x1 = x;
      s.z1 = z;
      s.a1 = angle;
      s.vu = numero;
      e.pv = pv;
      e.pvMax = 1;
      e.mort = (drapeaux & ENNEMI_MORT) !== 0 ? Math.max(e.mort, mort) : -1;
      e.blessure = (drapeaux & ENNEMI_BLESSE) !== 0 ? 0.1 : 0;
      e.brulure.cumuls = (drapeaux & ENNEMI_BRULURE) !== 0 ? 1 : 0;
      e.poison.cumuls = (drapeaux & ENNEMI_POISON) !== 0 ? 1 : 0;
      e.saignement.cumuls = (drapeaux & ENNEMI_SAIGNEMENT) !== 0 ? 1 : 0;
      e.ralentiTemps = (drapeaux & ENNEMI_RALENTI) !== 0 ? 1 : 0;
      const c = e.definition.comportement;
      e.meche = c.type === 'explosif' ? meche * c.duree : 0;
      e.vitesseSol = vitesse;
    }
    this.retirerDisparus(numero);
  }

  /** Une image : position entre les deux derniers instantanés (`t` de 0 à 1), animation, grille des armes. */
  interpoler(t: number, dtReel: number): void {
    const pool = this.ennemis.pool;
    for (let i = 0; i < pool.nombre; i++) {
      const s = this.suivisPool[i];
      const e = s.e;
      e.x = s.x0 + (s.x1 - s.x0) * t;
      e.z = s.z0 + (s.z1 - s.z0) * t;
      e.angle = s.a0 + ecartAngle(s.a0, s.a1) * t;
      if (e.mort >= 0) e.mort += dtReel;
      avancerAnimation(e.animation, this.modele(e.indexType), dtReel, e.vitesseSol / (e.definition.echelle ?? 1));
    }
    // Les armes (pour l'effet) cherchent leurs cibles dans la grille
    this.ennemis.reconstruireGrille();
  }

  /** Disparus de l'instantané : morts (gerbe), explosés (creeper) ou trop loin. */
  private retirerDisparus(numero: number): void {
    const pool = this.ennemis.pool;
    for (let i = pool.nombre - 1; i >= 0; i--) {
      const s = this.suivisPool[i];
      if (s.vu === numero) continue;
      const e = s.e;
      if (e.mort >= 0) this.particules.emettre(e.x, 0.6, e.z, GERBES.mort);
      else if (e.meche > 0) {
        this.particules.emettre(e.x, 0.8, e.z, GERBES.explosion);
        this.retours.son('explosion');
      }
      this.suivis.delete(e.id);
      // Le suivi suit son ennemi dans l'échange du pool
      const dernier = pool.nombre - 1;
      this.suivisPool[i] = this.suivisPool[dernier];
      this.suivisPool[dernier] = s;
      pool.liberer(i);
    }
  }

  private nouveau(id: number, type: number, x: number, z: number, angle: number): SuiviEnnemi | null {
    if (type >= IDS_ENNEMIS.length) return null;
    const pool = this.ennemis.pool;
    const e = pool.acquerir();
    if (!e) return null;
    const s = this.suivisPool[pool.nombre - 1];
    e.id = id;
    e.indexType = type;
    e.definition = ENNEMIS[IDS_ENNEMIS[type]];
    e.x = x;
    e.z = z;
    e.angle = angle;
    e.vx = 0;
    e.vz = 0;
    e.mort = -1;
    e.meche = 0;
    e.blessure = 0;
    e.immunites.fill(0);
    effacerEtats(e);
    reinitialiserAnimation(e.animation);
    this.suivis.set(id, s);
    return s;
  }

  private modele(indexType: number): ModeleCompile {
    let modele = this.modeles[indexType];
    if (!modele) {
      modele = compilerModele(MODELES[ENNEMIS[IDS_ENNEMIS[indexType]].modele]);
      this.modeles[indexType] = modele;
    }
    return modele;
  }
}
