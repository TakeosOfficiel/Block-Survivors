/**
 * Mécaniques des talents du Guerrier.
 * Gladiateur : enchaînements, Frénésie, critiques de mêlée, exécutions.
 * Gardien : réactions aux coups reçus (épines, posture, bastion), défense,
 * Onde de choc (Répliques, Séisme dans le système des fissures).
 */
import { GERBES } from '../../donnees/particules';
import type { Ennemi } from '../../entites/ennemi';
import type { NatureCoup } from '../../entites/ennemis';
import type { ArmeEquipee } from '../armeEquipee';
import { indexArme, MecaniquesClasse, type Talents } from './talents';

/** Valeurs des talents (provisoires), par rang quand il y en a deux. */
const V = {
  enchainement: { touches: 3, delai: 0.85 },
  frenesie: { eliminations: 10, fenetre: 3, duree: 4, delai: [0.85, 0.75], soin: 1 },
  champion: { eliminations: 6, duree: 6 },
  fureur: { duree: 5, degats: [0.05, 0.1] },
  bourreau: { seuil: 0.3, degats: [0.2, 0.35] },
  elan: { zone: 1.4, degats: 1.3, delaiInterne: 1.5 },
  saignee: { cumuls: 1, degatsParSeconde: 4, duree: 3 },
  lameLourde: { degats: [10, 20] },
  soif: { soin: [0.1, 0.2] },
  fureurGladiateur: { attaques: 5, rayon: 3.5, part: 0.6, delaiInterne: 3 },
  execution: { seuil: 0.15, delaiInterne: 2, bonusBoss: 0.3 },
  posture: { rayon: 3, force: 12, delaiInterne: 3 },
  epines: { degats: [10, 18], rayon: 2.5, delaiInterne: 2 },
  roncesAcier: { rayon: [0.5, 1], ralenti: 0.3, duree: 1 },
  rempart: { periode: 12, bouclier: 4 },
  inebranlable: { boss: 0.9 },
  peauDeFer: { seuil: 0.5, armure: [1, 2] },
  tenirLaLigne: { immobile: 1, reduction: [0.1, 0.2] },
  bastion: { charge: 30, rayon: 4, degats: 40, maximum: 80, delaiInterne: 6 },
  dernierRempart: { seuil: 0.25, protection: 3, rayon: 5, force: 16 },
  seisme: { ondes: 3 },
} as const;

const EPEE = indexArme('epee');
const ONDE = indexArme('onde_choc');
const HEURT = indexArme('heurt_bouclier');

export class MecaniquesGuerrier extends MecaniquesClasse {
  private readonly tampon = new Int32Array(128);
  // Frénésie : temps de jeu des dernières éliminations (anneau)
  private readonly eliminations = new Float32Array(16);
  private prochaineElimination = 0;
  private frenesie = 0;
  private fureur = 0;
  // Coups d'épée : touches du même coup (même instant)
  private instantEpee = -1;
  private touchesEpee = 0;
  private enchainementPret = false;
  // Élan critique : prochain balayage renforcé
  private elanPret = false;
  private elanApplique = false;
  private delaiElan = 0;
  // Fureur du gladiateur : attaques de mêlée qui touchent
  private instantMelee = -1;
  private armeMelee = -1;
  private attaquesMelee = 0;
  private delaiFureur = 0;
  private delaiExecution = 0;
  // Gardien
  private delaiPosture = 0;
  private delaiEpines = 0;
  private minuterieRempart: number = V.rempart.periode;
  private chargeBastion = 0;
  private delaiBastion = 0;
  private dernierRempartUtilise = false;
  private ondes = 0;

  constructor(talents: Talents) {
    super(talents);
  }

  override maj(dt: number): void {
    const t = this.t;
    if (this.frenesie > 0 && (this.frenesie -= dt) <= 0 && t.rang('fureur') > 0) this.fureur = V.fureur.duree;
    if (this.fureur > 0) this.fureur -= dt;
    this.delaiElan -= dt;
    this.delaiFureur -= dt;
    this.delaiExecution -= dt;
    this.delaiPosture -= dt;
    this.delaiEpines -= dt;
    this.delaiBastion -= dt;
    // Rempart : un bouclier toutes les 12 s
    if (t.rang('rempart') > 0 && (this.minuterieRempart -= dt) <= 0) {
      this.minuterieRempart = V.rempart.periode;
      t.bouclier(V.rempart.bouclier);
      const j = t.etat.joueur;
      t.contexte.particules.emettre(j.x, 1, j.z, GERBES.bouclier);
      t.contexte.retours.son('bouclier');
    }
  }

  // --- Déclenchements -------------------------------------------------------

  override avantDeclenchement(arme: ArmeEquipee): void {
    // Élan critique : ce balayage est plus large et plus fort
    if (arme.index === EPEE && this.elanPret) {
      this.elanPret = false;
      this.elanApplique = true;
      arme.stats.zone *= V.elan.zone;
      arme.stats.degats *= V.elan.degats;
    }
    // Séisme : toutes les 3 Ondes de choc, la suivante part dans 6 directions
    if (arme.index === ONDE && this.t.rang('seisme') > 0 && this.ondes >= V.seisme.ondes) {
      this.t.ondeCirculaire = true;
    }
  }

  override apresDeclenchement(arme: ArmeEquipee, declenchee: boolean): void {
    if (this.elanApplique && arme.index === EPEE) {
      this.elanApplique = false;
      arme.stats.zone /= V.elan.zone;
      arme.stats.degats /= V.elan.degats;
    }
    if (arme.index === ONDE && declenchee && this.t.rang('seisme') > 0) {
      this.ondes = this.t.ondeCirculaire ? 0 : this.ondes + 1;
    }
    this.t.ondeCirculaire = false;
  }

  override delai(arme: ArmeEquipee): number {
    let delai = 1;
    if (arme.index === EPEE && this.enchainementPret) {
      this.enchainementPret = false;
      delai *= V.enchainement.delai;
    }
    if (this.frenesie > 0 && this.t.famille(arme.index) === 'melee') delai *= this.t.valeur('frenesie', V.frenesie.delai) || 1;
    return delai;
  }

  // --- Coups portés ---------------------------------------------------------

  override multiplicateur(e: Ennemi, arme: number, _nature: NatureCoup): number {
    let m = 1;
    if (this.fureur > 0) m += this.t.valeur('fureur', V.fureur.degats);
    // Exécution : +30 % contre un boss presque vaincu
    if (arme === EPEE && e.definition.boss && e.pv < V.execution.seuil * e.pvMax && this.t.rang('execution') > 0) {
      m += V.execution.bonusBoss;
    }
    return m;
  }

  override facteurCritique(e: Ennemi, _arme: number, _nature: NatureCoup, base: number): number {
    // Œil du bourreau : critiques plus forts contre un ennemi affaibli
    if (e.pv < V.bourreau.seuil * e.pvMax) return base * (1 + this.t.valeur('bourreau', V.bourreau.degats));
    return base;
  }

  override apresCoup(e: Ennemi, _degats: number, arme: number, critique: boolean, nature: NatureCoup): void {
    const t = this.t;
    if (nature !== 'arme') return;
    const instant = t.etat.tempsJeu;
    const melee = t.famille(arme) === 'melee';

    // Enchaînement : un coup d'épée qui touche 3 ennemis
    if (arme === EPEE && t.rang('enchainement') > 0) {
      if (instant !== this.instantEpee) {
        this.instantEpee = instant;
        this.touchesEpee = 0;
      }
      if (++this.touchesEpee === V.enchainement.touches) this.enchainementPret = true;
    }

    if (critique && melee) {
      if (t.rang('elan_critique') > 0 && this.delaiElan <= 0) {
        this.elanPret = true;
        this.delaiElan = V.elan.delaiInterne;
      }
      if (t.rang('saignee') > 0) t.contexte.ennemis.saigner(e, V.saignee.cumuls, V.saignee.degatsParSeconde, V.saignee.duree, arme);
    }

    // Fureur du gladiateur : une attaque de mêlée qui touche (une fois par arme et par instant)
    if (melee && t.rang('fureur_gladiateur') > 0 && (instant !== this.instantMelee || arme !== this.armeMelee)) {
      this.instantMelee = instant;
      this.armeMelee = arme;
      this.attaquesMelee++;
      if (this.attaquesMelee >= V.fureurGladiateur.attaques && this.delaiFureur <= 0) {
        this.attaquesMelee = 0;
        this.delaiFureur = V.fureurGladiateur.delaiInterne;
        this.anneauFureur();
      }
    }

    // Exécution : l'épée achève un ennemi normal affaibli
    if (arme === EPEE && e.mort < 0 && !e.definition.boss && e.pv < V.execution.seuil * e.pvMax
      && this.delaiExecution <= 0 && t.rang('execution') > 0) {
      this.delaiExecution = V.execution.delaiInterne;
      t.contexte.particules.emettre(e.x, 1, e.z, GERBES.impactLame);
      t.blesser(e, e.pv, t.etat.joueur.x, t.etat.joueur.z, 0, arme);
    }
  }

  override surMort(_e: Ennemi, arme: number, _nature: NatureCoup): void {
    const t = this.t;
    if (t.famille(arme) === 'melee' && t.rang('soif_combat') > 0) t.soignerPlafonne(t.valeur('soif_combat', V.soif.soin));
    if (t.rang('frenesie') === 0) return;
    // Frénésie : assez d'éliminations dans la fenêtre
    const champion = t.rang('champion_arene') > 0;
    const seuil = champion ? V.champion.eliminations : V.frenesie.eliminations;
    const maintenant = t.etat.tempsJeu;
    this.eliminations[this.prochaineElimination] = maintenant;
    this.prochaineElimination = (this.prochaineElimination + 1) % this.eliminations.length;
    if (this.frenesie > 0) return;
    let recentes = 0;
    for (let k = 0; k < this.eliminations.length; k++) {
      if (this.eliminations[k] > 0 && maintenant - this.eliminations[k] <= V.frenesie.fenetre) recentes++;
    }
    if (recentes >= seuil) {
      this.eliminations.fill(0);
      this.frenesie = champion ? V.champion.duree : V.frenesie.duree;
      this.fureur = 0;
      t.soigner(V.frenesie.soin);
      const j = t.etat.joueur;
      t.contexte.particules.emettre(j.x, 1.2, j.z, GERBES.critique);
      t.cercle(j.x, j.z, 1.2, 'anneau', 12);
    }
  }

  override surImpactMur(e: Ennemi): void {
    const degats = this.t.valeur('lame_lourde', V.lameLourde.degats);
    if (degats <= 0) return;
    this.t.contexte.particules.emettre(e.x, 0.8, e.z, GERBES.heurt);
    this.t.blesser(e, degats, e.x, e.z, 0, EPEE);
  }

  // --- Coups reçus ----------------------------------------------------------

  override degatsRecus(degats: number, boss: boolean): number {
    const t = this.t;
    if (boss && t.rang('inebranlable') > 0) degats *= V.inebranlable.boss;
    if (t.tempsImmobile >= V.tenirLaLigne.immobile) degats *= 1 - t.valeur('tenir_la_ligne', V.tenirLaLigne.reduction);
    return degats;
  }

  override armure(): number {
    const j = this.t.etat.joueur;
    return j.pv < V.peauDeFer.seuil * j.pvMax ? this.t.valeur('peau_de_fer', V.peauDeFer.armure) : 0;
  }

  override apresCoupRecu(_subis: number, evites: number): void {
    const t = this.t;
    const j = t.etat.joueur;
    const { ennemis, particules } = t.contexte;
    // Posture défensive : repousse les ennemis proches
    if (t.rang('posture_defensive') > 0 && this.delaiPosture <= 0) {
      this.delaiPosture = V.posture.delaiInterne;
      ennemis.repousser(j.x, j.z, V.posture.rayon, V.posture.force);
      t.cercle(j.x, j.z, V.posture.rayon * 0.6, 'repousse', 12);
    }
    // Épines (et Ronces d'acier) : des pointes autour du Guerrier
    const epines = t.valeur('epines', V.epines.degats);
    if (epines > 0 && this.delaiEpines <= 0) {
      this.delaiEpines = V.epines.delaiInterne;
      const rayon = V.epines.rayon + t.valeur('ronces_acier', V.roncesAcier.rayon);
      const n = ennemis.chercher(j.x, j.z, rayon, this.tampon);
      for (let k = 0; k < n; k++) {
        const e = ennemis.obtenir(this.tampon[k]);
        t.blesser(e, epines, j.x, j.z, 2, HEURT);
        if (t.rang('ronces_acier') > 0) ennemis.ralentir(e, V.roncesAcier.ralenti, V.roncesAcier.duree);
      }
      t.cercle(j.x, j.z, rayon * 0.7, 'epines', 16);
    }
    // Contre-attaque du bastion : les dégâts évités chargent une frappe au sol
    if (t.rang('contre_attaque_bastion') > 0) {
      this.chargeBastion += evites;
      if (this.chargeBastion >= V.bastion.charge && this.delaiBastion <= 0) {
        const degats = Math.min(V.bastion.maximum, V.bastion.degats + this.chargeBastion);
        this.chargeBastion = 0;
        this.delaiBastion = V.bastion.delaiInterne;
        t.blesserZone(j.x, j.z, V.bastion.rayon, degats, HEURT, 6, this.tampon);
        particules.emettre(j.x, 0.3, j.z, GERBES.impactLame);
        t.cercle(j.x, j.z, V.bastion.rayon, 'repousse', 20);
        t.contexte.retours.son('lame_colossale');
        t.contexte.retours.secousse(0.4);
      }
    }
    // Dernier rempart : une fois par partie, sous 25 % de PV
    if (j.pv < V.dernierRempart.seuil * j.pvMax) this.dernierRempart();
  }

  override eviterMort(): boolean {
    // Le Dernier rempart sauve aussi d'un coup mortel, s'il n'a pas encore servi
    if (this.t.rang('dernier_rempart') === 0 || this.dernierRempartUtilise) return false;
    this.t.etat.joueur.pv = 1;
    this.dernierRempart();
    return true;
  }

  private dernierRempart(): void {
    const t = this.t;
    if (t.rang('dernier_rempart') === 0 || this.dernierRempartUtilise) return;
    this.dernierRempartUtilise = true;
    const j = t.etat.joueur;
    j.protection = Math.max(j.protection, V.dernierRempart.protection);
    t.contexte.ennemis.repousser(j.x, j.z, V.dernierRempart.rayon, V.dernierRempart.force);
    t.contexte.particules.emettre(j.x, 1, j.z, GERBES.totem);
    t.cercle(j.x, j.z, V.dernierRempart.rayon * 0.6, 'repousse', 24);
    t.contexte.retours.son('totem');
    t.contexte.retours.secousse(0.5);
  }

  /** Fureur du gladiateur : anneau rouge autour du Guerrier, 60 % des dégâts de l'épée. */
  private anneauFureur(): void {
    const t = this.t;
    const j = t.etat.joueur;
    let degatsEpee = 0;
    for (const a of t.arsenal.armes) if (a.index === EPEE) degatsEpee = a.stats.degats;
    if (degatsEpee <= 0) return;
    const r = V.fureurGladiateur.rayon;
    t.blesserZone(j.x, j.z, r, degatsEpee * V.fureurGladiateur.part, EPEE, 4, this.tampon);
    t.cercle(j.x, j.z, r, 'anneau', 32);
    t.cercle(j.x, j.z, r * 0.6, 'anneau', 20);
    t.contexte.retours.son('balayage');
  }
}
