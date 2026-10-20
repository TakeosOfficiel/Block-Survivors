/**
 * Mécaniques des talents de l'Archère.
 * Élite : marque du chasseur, premiers coups, Puissance (flèche géante), Œil
 * de l'aigle (ricochets critiques), Exécutrice. Visée stable, Perforation
 * nette, Tir groupé et Tir rebondissant sont lus par les systèmes d'armes.
 * Rôdeuse : mouvement (Pas du chasseur, Course chargée), Venin, pièges,
 * Contagion, Nuage toxique.
 */
import { ETATS } from '../../donnees/etats';
import { GERBES } from '../../donnees/particules';
import type { Ennemi } from '../../entites/ennemi';
import type { NatureCoup } from '../../entites/ennemis';
import { Pool } from '../../utils/pool';
import type { ArmeEquipee } from '../armeEquipee';
import { indexArme, MecaniquesClasse, type Talents } from './talents';

/** Valeurs des talents (provisoires), par rang quand il y en a deux. */
const V = {
  premierSang: { seuil: 0.9, degats: [0.2, 0.35] },
  marque: { duree: 4, degats: [0.1, 0.2] },
  pointFaible: { degats: [0.1, 0.15] },
  concentration: { immobile: 1, delai: [0.92, 0.85] },
  puissance: { tirs: 12, multiplicateur: 3, ajout: 20, portee: 30, delaiInterne: 4 },
  oeilAigle: { portee: 14, part: 1 },
  executrice: { seuil: 0.1, parCritique: 0.02, cumulsMax: 10, duree: 6 },
  pasChasseur: { mouvement: 2, vitesse: 1.1 },
  venin: { chance: [0.2, 0.35], degatsParSeconde: 2, duree: 4 },
  concentre: { degats: 1.5, cumulsMax: 3 },
  virulent: { degats: 0.8, cumulsMax: 8 },
  contagion: { cumuls: [1, 2], rayon: 3 },
  course: { distance: 1.5, charges: 10, chargesSecondVent: 7, fleches: 3, degats: [0.15, 0.3] },
  piegeALoups: { ralenti: 0.3, duree: [1, 2] },
  roncesVenimeuses: { degats: 0.3 },
  nuage: { duree: 2, rayon: 2, periode: 0.5, delaiInterne: 1, maximum: 3 },
} as const;

const ARC = indexArme('arc');

interface Nuage {
  x: number;
  z: number;
  temps: number;
  tic: number;
  arme: number;
}

export class MecaniquesArcher extends MecaniquesClasse {
  private readonly tampon = new Int32Array(128);
  private tirs = 0;
  private delaiPuissance = 0;
  private puissanceAppliquee = false;
  private charges = 0;
  private dernierParcouru = 0;
  private courseAppliquee = false;
  private readonly nuages = new Pool<Nuage>(V.nuage.maximum, () => ({ x: 0, z: 0, temps: 0, tic: 0, arme: -1 }));
  private delaiNuage = 0;

  constructor(talents: Talents) {
    super(talents);
    this.dernierParcouru = talents.parcouru;
  }

  override maj(dt: number): void {
    const t = this.t;
    this.delaiPuissance -= dt;
    this.delaiNuage -= dt;
    // Course chargée : une charge tous les 1,5 bloc parcourus
    if (t.rang('course_chargee') > 0) {
      while (t.parcouru - this.dernierParcouru >= V.course.distance) {
        this.dernierParcouru += V.course.distance;
        if (this.charges < this.chargesMax()) this.charges++;
      }
    } else {
      this.dernierParcouru = t.parcouru;
    }
    // Nuages toxiques : un cumul de poison aux ennemis dedans toutes les 0,5 s
    const { ennemis, particules } = t.contexte;
    for (let i = this.nuages.nombre - 1; i >= 0; i--) {
      const n = this.nuages.elements[i];
      n.temps -= dt;
      if (n.temps <= 0) {
        this.nuages.liberer(i);
        continue;
      }
      if ((n.tic -= dt) > 0) continue;
      n.tic += V.nuage.periode;
      const k = ennemis.chercher(n.x, n.z, V.nuage.rayon, this.tampon);
      for (let q = 0; q < k; q++) this.empoisonner(ennemis.obtenir(this.tampon[q]), 1, n.arme);
      for (let q = 0; q < 6; q++) {
        const a = Math.random() * Math.PI * 2;
        const r = Math.sqrt(Math.random()) * V.nuage.rayon;
        particules.emettre(n.x + Math.sin(a) * r, 0.5, n.z + Math.cos(a) * r, GERBES.nuage);
      }
    }
  }

  private chargesMax(): number {
    return this.t.rang('second_vent') > 0 ? V.course.chargesSecondVent : V.course.charges;
  }

  // --- Déclenchements -------------------------------------------------------

  override avantDeclenchement(arme: ArmeEquipee): void {
    const t = this.t;
    // Puissance : après 12 tirs, la prochaine flèche de l'Arc devient géante
    if (arme.index === ARC && t.rang('puissance') > 0 && this.tirs >= V.puissance.tirs && this.delaiPuissance <= 0) {
      t.flecheGeante = arme.stats.degats * V.puissance.multiplicateur + V.puissance.ajout;
      t.porteeFlecheGeante = V.puissance.portee;
      this.puissanceAppliquee = true;
      return;
    }
    // Course chargée : la volée pleine tire 3 flèches de plus, plus fortes (et vers l'arrière avec Vent arrière)
    if (arme.definition.mecanique.type === 'tir' && t.rang('course_chargee') > 0 && this.charges >= this.chargesMax()) {
      this.courseAppliquee = true;
      arme.stats.nombre += V.course.fleches;
      arme.stats.degats *= 1 + t.valeur('course_chargee', V.course.degats);
      t.voleeArriere = t.rang('vent_arriere') > 0;
    }
  }

  override apresDeclenchement(arme: ArmeEquipee, declenchee: boolean): void {
    const t = this.t;
    if (this.puissanceAppliquee) {
      this.puissanceAppliquee = false;
      t.flecheGeante = 0;
      if (declenchee) {
        this.tirs = 0;
        this.delaiPuissance = V.puissance.delaiInterne;
        t.contexte.retours.son('fusee');
      }
    } else if (declenchee && t.famille(arme.index) === 'projectile') {
      this.tirs++;
    }
    if (this.courseAppliquee) {
      this.courseAppliquee = false;
      arme.stats.nombre -= V.course.fleches;
      arme.stats.degats /= 1 + t.valeur('course_chargee', V.course.degats);
      t.voleeArriere = false;
      if (declenchee) this.charges = 0;
    }
  }

  override delai(arme: ArmeEquipee): number {
    const t = this.t;
    if (t.famille(arme.index) === 'projectile' && t.tempsImmobile >= V.concentration.immobile) {
      return t.valeur('concentration', V.concentration.delai) || 1;
    }
    return 1;
  }

  override vitesse(): number {
    return this.t.rang('pas_chasseur') > 0 && this.t.tempsMouvement >= V.pasChasseur.mouvement ? V.pasChasseur.vitesse : 1;
  }

  // --- Coups portés ---------------------------------------------------------

  override multiplicateur(e: Ennemi, arme: number, nature: NatureCoup): number {
    const t = this.t;
    if (arme < 0 || nature !== 'arme') return 1;
    let m = 1;
    if (e.pv > V.premierSang.seuil * e.pvMax) m += t.valeur('premier_sang', V.premierSang.degats);
    if (e.marqueChasseur > 0) {
      m += t.valeur('marque_chasseur', V.marque.degats);
      if (e.definition.boss) m += t.valeur('point_faible', V.pointFaible.degats);
    }
    if (e.definition.boss && e.cumulsTalent > 0) m += e.cumulsTalent * V.executrice.parCritique;
    if (e.dansPiege && t.famille(arme) === 'projectile' && t.rang('ronces_venimeuses') > 0) m += V.roncesVenimeuses.degats;
    return m;
  }

  override apresCoup(e: Ennemi, degats: number, arme: number, critique: boolean, nature: NatureCoup): void {
    const t = this.t;
    if (nature !== 'arme' || arme < 0) return;
    const fleche = t.famille(arme) === 'projectile';

    if (critique) {
      if (t.rang('marque_chasseur') > 0) e.marqueChasseur = V.marque.duree;
      // Exécutrice : chaque critique sur un boss ajoute un cumul
      if (e.definition.boss && t.rang('executrice') > 0) {
        e.cumulsTalent = Math.min(V.executrice.cumulsMax, e.cumulsTalent + 1);
        e.tempsCumuls = V.executrice.duree;
      }
      // Œil de l'aigle : le critique ricoche vers l'ennemi le plus éloigné
      if (t.rang('oeil_aigle') > 0) this.oeilAigle(e, degats, arme);
    }

    // Venin : une part des flèches empoisonne
    if (fleche && e.mort < 0 && Math.random() < t.valeur('venin', V.venin.chance)) this.empoisonner(e, 1, arme);

    // Exécutrice : les flèches achèvent un ennemi normal presque vaincu
    if (fleche && e.mort < 0 && !e.definition.boss && e.pv < V.executrice.seuil * e.pvMax && t.rang('executrice') > 0) {
      t.blesser(e, e.pv, e.x, e.z, 0, arme);
    }
  }

  override surMort(e: Ennemi, _arme: number, _nature: NatureCoup): void {
    const t = this.t;
    const poison = e.poison;
    if (poison.cumuls === 0) return;
    // Contagion : un voisin reçoit des cumuls
    const contagion = t.valeur('contagion', V.contagion.cumuls);
    if (contagion > 0) {
      const ennemis = t.contexte.ennemis;
      const n = ennemis.chercher(e.x, e.z, V.contagion.rayon, this.tampon);
      let voisin: Ennemi | null = null;
      let meilleur = Infinity;
      for (let k = 0; k < n; k++) {
        const o = ennemis.obtenir(this.tampon[k]);
        const d2 = (o.x - e.x) ** 2 + (o.z - e.z) ** 2;
        if (o !== e && o.mort < 0 && d2 < meilleur) {
          meilleur = d2;
          voisin = o;
        }
      }
      if (voisin) this.empoisonner(voisin, contagion, poison.arme);
    }
    // Nuage toxique : mort avec le maximum de cumuls
    if (t.rang('nuage_toxique') > 0 && poison.cumuls >= this.cumulsPoison() && this.delaiNuage <= 0) {
      const nuage = this.nuages.acquerir();
      if (nuage) {
        this.delaiNuage = V.nuage.delaiInterne;
        nuage.x = e.x;
        nuage.z = e.z;
        nuage.temps = V.nuage.duree;
        nuage.tic = 0;
        nuage.arme = poison.arme;
      }
    }
  }

  override surSortiePiege(e: Ennemi): void {
    const duree = this.t.valeur('piege_a_loups', V.piegeALoups.duree);
    if (duree > 0) this.t.contexte.ennemis.ralentir(e, V.piegeALoups.ralenti, duree);
  }

  override cumulsPoison(): number {
    if (this.t.rang('venin_concentre') > 0) return V.concentre.cumulsMax;
    if (this.t.rang('venin_virulent') > 0) return V.virulent.cumulsMax;
    return ETATS.poison.cumulsMax;
  }

  /** Poison du Venin (options comprises). */
  private empoisonner(e: Ennemi, cumuls: number, arme: number): void {
    let dps: number = V.venin.degatsParSeconde;
    if (this.t.rang('venin_concentre') > 0) dps *= V.concentre.degats;
    if (this.t.rang('venin_virulent') > 0) dps *= V.virulent.degats;
    this.t.contexte.ennemis.empoisonner(e, cumuls, dps, V.venin.duree, arme, this.cumulsPoison());
  }

  private oeilAigle(source: Ennemi, degats: number, arme: number): void {
    const t = this.t;
    const ennemis = t.contexte.ennemis;
    const n = ennemis.chercher(source.x, source.z, V.oeilAigle.portee, this.tampon);
    let cible: Ennemi | null = null;
    let meilleur = -1;
    for (let k = 0; k < n; k++) {
      const o = ennemis.obtenir(this.tampon[k]);
      const d2 = (o.x - source.x) ** 2 + (o.z - source.z) ** 2;
      if (o !== source && o.mort < 0 && d2 > meilleur) {
        meilleur = d2;
        cible = o;
      }
    }
    if (!cible) return;
    for (const a of t.arsenal.armes) {
      if (a.index === arme) {
        t.systemes.ricochets.ricocher(a, source.x, source.z, cible, degats * V.oeilAigle.part);
        return;
      }
    }
  }
}
