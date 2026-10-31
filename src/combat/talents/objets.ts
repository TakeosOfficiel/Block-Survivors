/**
 * Effets propres de l'équipement porté (donnees/equipement.ts, champ `effet`) :
 * branchés sur les mêmes événements que les talents, après la classe
 * (combat/talents/talents.ts). Règles de cumul : les multiplicateurs de dégâts
 * se multiplient (classe × objets) ; à 0 PV, un talent passe avant le totem.
 *
 * Aussi le coup qui enflamme (Bague de magma, garnitures du Nether :
 * modificateur `chanceBrulure`). Les dégâts des objets sont des réactions :
 * ils ne relancent ni les talents ni les objets (pas d'explosions en chaîne).
 */
import { GERBES } from '../../donnees/particules';
import type { Ennemi } from '../../entites/ennemi';
import type { NatureCoup } from '../../entites/ennemis';
import type { Equipement } from '../../progression/equipement';
import type { Talents } from './talents';
import { alea } from '../../utils/aleatoire';

/** Brûlure d'un coup qui enflamme : dégâts par seconde (× dégâts des armes) et durée. */
const BRULURE_OBJET = { degatsParSeconde: 3, duree: 3 };
const TAMPON = new Int32Array(256);

export class MecaniquesObjets {
  private readonly t: Talents;
  private readonly equipement: Equipement;
  /** s écoulées depuis la dernière cloche / la dernière couronne ; s avant le prochain Cœur de magma. */
  private cloche = 0;
  private couronne = 0;
  private rechargeMagma = 0;

  constructor(talents: Talents, equipement: Equipement) {
    this.t = talents;
    this.equipement = equipement;
  }

  maj(dt: number): void {
    if (this.rechargeMagma > 0) this.rechargeMagma -= dt;
    const effets = this.equipement.effets;
    for (let i = 0; i < effets.length; i++) {
      const { effet, facteur } = effets[i];
      if (effet.type === 'cloche') {
        this.cloche += dt;
        if (this.cloche < effet.intervalle / facteur) continue;
        this.cloche = 0;
        const j = this.t.etat.joueur;
        const ennemis = this.t.contexte.ennemis;
        ennemis.repousser(j.x, j.z, effet.rayon, effet.force);
        const n = ennemis.chercher(j.x, j.z, effet.rayon, TAMPON);
        for (let k = 0; k < n; k++) ennemis.etourdir(ennemis.obtenir(TAMPON[k]), effet.etourdi);
        this.t.cercle(j.x, j.z, effet.rayon, 'repousse', 28);
        this.t.contexte.retours.son('bouclier');
      } else if (effet.type === 'couronne') {
        this.couronne += dt;
        if (this.couronne < effet.intervalle) continue;
        this.couronne = 0;
        const j = this.t.etat.joueur;
        this.exploser(j.x, j.z, effet.rayon, effet.degats * this.t.etat.modificateurs.degats, 'nuage');
        this.empoisonner(j.x, j.z, effet.rayon, effet.poison, effet.duree);
        this.t.contexte.retours.son('explosion');
        this.t.contexte.retours.secousse(0.3);
      }
    }
  }

  /** Multiplicateur des dégâts d'un coup (Amulette du bastion). */
  multiplicateur(_e: Ennemi, _arme: number, _nature: NatureCoup): number {
    let m = 1;
    const effets = this.equipement.effets;
    const j = this.t.etat.joueur;
    for (let i = 0; i < effets.length; i++) {
      const { effet, facteur } = effets[i];
      if (effet.type === 'bastion' && j.pv < effet.seuil * j.pvMax) m *= 1 + effet.bonus * facteur;
    }
    return m;
  }

  /** Coup d'arme qui enflamme (chanceBrulure). */
  apresCoup(e: Ennemi, _degats: number, arme: number, _critique: boolean, nature: NatureCoup): void {
    const chance = this.t.etat.modificateurs.chanceBrulure;
    if (chance <= 0 || arme < 0 || nature !== 'arme' || e.mort >= 0 || alea() >= chance) return;
    this.t.contexte.ennemis.bruler(e, 1, BRULURE_OBJET.degatsParSeconde * this.t.etat.modificateurs.degats, BRULURE_OBJET.duree, arme);
  }

  /** Ennemi tué : éclat de diamant (Cœur du filon), spores (Amulette de spores). */
  surMort(e: Ennemi): void {
    if (this.t.reaction) return;
    const effets = this.equipement.effets;
    for (let i = 0; i < effets.length; i++) {
      const { effet, facteur } = effets[i];
      if (effet.type === 'filon' && alea() < effet.chance) {
        this.exploser(e.x, e.z, effet.rayon, effet.degats * this.t.etat.modificateurs.degats, 'critique');
        this.t.contexte.particules.emettre(e.x, 0.8, e.z, GERBES.eboulis);
        this.t.contexte.retours.son('explosion');
      } else if (effet.type === 'spores' && alea() < Math.min(0.5, effet.chance * facteur)) {
        this.empoisonner(e.x, e.z, effet.rayon, effet.poison * this.t.etat.modificateurs.degats, effet.duree);
        this.t.contexte.particules.emettre(e.x, 0.6, e.z, GERBES.nuage);
      }
    }
  }

  /** Gros coup reçu : nova de feu et invulnérabilité (Cœur de magma). */
  apresCoupRecu(subis: number): void {
    if (this.rechargeMagma > 0) return;
    const effets = this.equipement.effets;
    const j = this.t.etat.joueur;
    for (let i = 0; i < effets.length; i++) {
      const { effet } = effets[i];
      if (effet.type !== 'magma' || subis < effet.seuil * j.pvMax) continue;
      this.rechargeMagma = effet.recharge;
      j.protection = Math.max(j.protection, effet.protection);
      this.exploser(j.x, j.z, effet.rayon, effet.degats * this.t.etat.modificateurs.degats, 'brulure');
      const ennemis = this.t.contexte.ennemis;
      const n = ennemis.chercher(j.x, j.z, effet.rayon, TAMPON);
      for (let k = 0; k < n; k++) ennemis.bruler(ennemis.obtenir(TAMPON[k]), 1, BRULURE_OBJET.degatsParSeconde * this.t.etat.modificateurs.degats, BRULURE_OBJET.duree);
      this.t.contexte.retours.son('nova');
      return;
    }
  }

  private exploser(x: number, z: number, rayon: number, degats: number, gerbe: 'critique' | 'nuage' | 'brulure'): void {
    this.t.blesserZone(x, z, rayon, degats, -1, 3);
    this.t.cercle(x, z, rayon, gerbe === 'brulure' ? 'brulure' : 'anneau', 20);
    this.t.contexte.particules.emettre(x, 0.8, z, GERBES[gerbe]);
  }

  private empoisonner(x: number, z: number, rayon: number, degatsParSeconde: number, duree: number): void {
    const ennemis = this.t.contexte.ennemis;
    const n = ennemis.chercher(x, z, rayon, TAMPON);
    for (let k = 0; k < n; k++) ennemis.empoisonner(ennemis.obtenir(TAMPON[k]), 1, degatsParSeconde, duree);
  }
}
