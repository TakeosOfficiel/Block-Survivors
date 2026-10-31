/**
 * Mécaniques des talents de l'Invocateur.
 * Maître-chien : Chef de meute, marques (Morsure marquante, Traque, Curée,
 * Coordination), Meute soudée, Hurlement, Encerclement, Alpha et Gardien
 * alpha, Ruée de la meute, Lune de sang. Braises de blaze est lu par le
 * système des invocations.
 * Évocateur : Lien spirituel, Garde rapprochée, Allay chanteur, Essaim,
 * Évocation supérieure (un ravageur charge la horde), Crocs de l'évocateur.
 * Bosquet (haie de racines entre deux totems) vit dans le système des invocations.
 */
import { ALLIES, CREATURES_TALENTS, IDS_ALLIES } from '../../donnees/allies';
import { MODELES } from '../../donnees/modeles';
import { GERBES } from '../../donnees/particules';
import type { DefinitionAllie } from '../../donnees/types';
import type { Ombres } from '../../effets/ombres';
import type { Ennemi } from '../../entites/ennemi';
import type { NatureCoup } from '../../entites/ennemis';
import { avancerAnimation, calculerRotations, creerEtatAnimation } from '../../modeles/animation';
import { compilerModele, type ModeleCompile } from '../../modeles/compilation';
import { creerPoseModele, ModeleInstancie, type PoseModele } from '../../modeles/rendu';
import type { Point } from '../../monde/carte';
import { longueur } from '../../utils/maths';
import { Pool } from '../../utils/pool';
import type { ArmeEquipee } from '../armeEquipee';
import type { Allie } from '../invocations';
import { indexArme, MecaniquesClasse, type Talents } from './talents';
import { alea } from '../../utils/aleatoire';

/** Valeurs des talents (provisoires), par rang quand il y en a deux. */
const V = {
  chef: { degats: 1.5 },
  morsure: { marque: [0.15, 0.25], duree: 3, traque: 2 },
  reactivite: { avance: [0.25, 0.4] },
  meute: { parAllie: [0.03, 0.04], maximum: [0.24, 0.32] },
  curee: { soin: 0.1, rayon: 6, delai: 8 },
  hurlement: { periode: 10, duree: 3, cadence: [0.1, 0.2] },
  encerclement: { fenetre: 2, duree: 3, degats: [0.15, 0.3] },
  coordination: { degats: [0.1, 0.2] },
  alpha: { pv: 2, degats: [0.3, 0.5] },
  ruee: { coups: 15, duree: 3, vitesse: 2, degats: 0.5, fantomes: 2, delaiInterne: 12 },
  lune: { periode: 60, duree: 8, quota: 12, partPv: 0.5 },
  gardienAlpha: { echelle: 1.25 },
  lien: { bouclier: 2, delaiInterne: 5 },
  garde: { rayon: 4, armure: [1, 2] },
  allay: { rayon: 3.5, periode: 0.25 },
  essaim: { degats: [6, 10], rayon: 1.5 },
  evocation: { declenchements: 8, duree: 8, degats: 40, recul: 18, vitesse: 8, rayon: 1.8, immunite: 0.8, delaiInterne: 30, cap: 1.2 },
  crocs: { periode: 6, nombre: 8, depart: 1.5, ecart: 1.2, decalage: 0.06, vie: 1, claque: 0.4, degats: 20, rayon: 1, portee: 12 },
} as const;

const LOUPS = indexArme('loups');
const VEX = indexArme('vex');
const BLAZE = indexArme('esprit_blaze');
/** Les dégâts du ravageur et des crocs comptent pour l'arme de départ (statistiques). */
const CREDIT = LOUPS;

interface Croc {
  x: number;
  z: number;
  angle: number;
  /** s depuis l'apparition (négatif : attend son tour). */
  t: number;
  frappe: boolean;
}

const groupe: Point = { x: 0, z: 0 };

export class MecaniquesInvocateur extends MecaniquesClasse {
  private readonly tampon = new Int32Array(128);
  private vivants = 0;
  private minuterieComptage = 0;
  private minuterieHurlement: number = V.hurlement.periode;
  private hurlement = 0;
  private minuterieLune: number = V.lune.periode;
  private lune = 0;
  private coupsMarques = 0;
  private ruee = 0;
  private delaiRuee = 0;
  private delaiLien = 0;
  // Évocation supérieure : ravageur
  private declenchements = 0;
  private delaiEvocation = 0;
  private ravageur = { actif: false, x: 0, z: 0, dirX: 0, dirZ: 1, temps: 0, cap: 0, animation: creerEtatAnimation(), vitesseSol: 0, pas: 0 };
  private renduRavageur: { modele: ModeleCompile; instances: ModeleInstancie; pose: PoseModele } | null = null;
  // Crocs de l'évocateur
  private minuterieCrocs: number = V.crocs.periode;
  private readonly crocs = new Pool<Croc>(V.crocs.nombre * 3, () => ({ x: 0, z: 0, angle: 0, t: 0, frappe: false }));
  private renduCrocs: { modele: ModeleCompile; instances: ModeleInstancie; pose: PoseModele; machoireA: number; machoireB: number } | null = null;

  constructor(talents: Talents) {
    super(talents);
    const { scene, textures } = talents.ressources;
    if (talents.rang('evocation_superieure') > 0) {
      const a = CREATURES_TALENTS.ravageur;
      const modele = compilerModele(MODELES[a.modele]);
      this.renduRavageur = { modele, instances: new ModeleInstancie(modele, a, textures, 1, scene), pose: creerPoseModele(modele) };
    }
    if (talents.rang('crocs_evocateur') > 0) {
      const a = CREATURES_TALENTS.crocs;
      const modele = compilerModele(MODELES[a.modele]);
      const index = (nom: string) => modele.parties.findIndex((p) => p.nom === nom);
      this.renduCrocs = {
        modele, instances: new ModeleInstancie(modele, a, textures, this.crocs.capacite, scene), pose: creerPoseModele(modele),
        machoireA: index('machoireA'), machoireB: index('machoireB'),
      };
    }
  }

  override maj(dt: number): void {
    const t = this.t;
    const { particules, retours } = t.contexte;
    const invocations = t.systemes.invocations;
    const j = t.etat.joueur;
    this.delaiRuee -= dt;
    this.delaiLien -= dt;
    this.delaiEvocation -= dt;
    if (this.ruee > 0) this.ruee -= dt;

    // Alliés en vie (Meute soudée), recomptés quatre fois par seconde ; l'allay ramasse l'XP
    if ((this.minuterieComptage -= dt) <= 0) {
      this.minuterieComptage = V.allay.periode;
      let n = 0;
      for (let i = 0; i < invocations.nombre; i++) {
        const a = invocations.liste[i];
        if (!a.arme || a.pv <= 0) continue;
        n++;
        if (a.definition.soutien && t.rang('allay_chanteur') > 0) t.ressources.orbes.aimanter(a.x, a.z, V.allay.rayon);
      }
      this.vivants = n;
    }

    // Hurlement : toutes les 10 s, les alliés attaquent plus vite
    if (t.rang('hurlement') > 0) {
      if (this.hurlement > 0) this.hurlement -= dt;
      if ((this.minuterieHurlement -= dt) <= 0) {
        this.minuterieHurlement = V.hurlement.periode;
        this.hurlement = V.hurlement.duree;
        this.hurler();
      }
    }

    // Lune de sang : toutes les 60 s, le quota de loups double pendant 8 s
    if (t.rang('lune_de_sang') > 0) {
      if (this.lune > 0) this.lune -= dt;
      if ((this.minuterieLune -= dt) <= 0) {
        this.minuterieLune = V.lune.periode;
        this.lune = V.lune.duree;
        const loups = this.arme(LOUPS);
        if (loups) {
          const manquants = V.lune.quota - invocations.presents(IDS_ALLIES.indexOf('loup'));
          invocations.invoquerTemporaire(loups, manquants, V.lune.duree, V.lune.partPv);
        }
        retours.son('loup_hurlement');
        particules.emettre(j.x, 2, j.z, GERBES.critique);
      }
    }

    this.majRavageur(dt);
    this.majCrocs(dt);
  }

  // --- Alliés ---------------------------------------------------------------

  override quotaAllie(def: DefinitionAllie): number {
    return this.lune > 0 && def === ALLIES.loup ? V.lune.quota - def.quota : 0;
  }

  override surCreationAllie(a: Allie): void {
    const t = this.t;
    if (a.definition !== ALLIES.loup) return;
    a.premiereMorsure = t.rang('chef_meute') > 0;
    // Pendant la Lune, les loups au-delà du quota normal repartent à la fin
    if (this.lune > 0 && t.systemes.invocations.presents(a.indexType) > ALLIES.loup.quota) {
      a.temporaire = true;
      a.vie = Math.min(a.vie, this.lune);
    }
    // Alpha : le premier loup (s'il n'y en a pas d'autre en vie)
    if (t.rang('alpha') === 0 || a.temporaire) return;
    const invocations = t.systemes.invocations;
    for (let i = 0; i < invocations.nombre; i++) {
      const o = invocations.liste[i];
      if (o !== a && o.alpha && o.arme && o.pv > 0) return;
    }
    a.alpha = true;
    a.pvMax *= V.alpha.pv;
    a.pv = a.pvMax;
    if (t.rang('gardien_alpha') > 0) a.echelle = V.gardienAlpha.echelle;
  }

  override ciblePrioritaire(a: Allie): Ennemi | null {
    const t = this.t;
    const loup = a.definition === ALLIES.loup;
    const tireur = a.definition === ALLIES.vex || a.definition === ALLIES.esprit_blaze;
    if (!(loup && t.rang('traque') > 0) && !(tireur && t.rang('coordination') > 0)) return null;
    const ennemis = t.contexte.ennemis;
    const n = ennemis.chercher(a.x, a.z, a.definition.detection, this.tampon);
    let cible: Ennemi | null = null;
    let meilleure = Infinity;
    for (let k = 0; k < n; k++) {
      const e = ennemis.obtenir(this.tampon[k]);
      if (e.marqueTemps <= 0) continue;
      const d2 = (e.x - a.x) ** 2 + (e.z - a.z) ** 2;
      if (d2 < meilleure) {
        meilleure = d2;
        cible = e;
      }
    }
    return cible;
  }

  override multiplicateurAllie(a: Allie, _e: Ennemi): number {
    let m = 1;
    if (a.premiereMorsure) m *= V.chef.degats;
    if (a.alpha) m *= 1 + this.t.valeur('alpha', V.alpha.degats);
    if (this.ruee > 0 && a.definition === ALLIES.loup) m *= 1 + V.ruee.degats;
    return m;
  }

  override cadenceAllie(_a: Allie): number {
    return this.hurlement > 0 ? 1 - this.t.valeur('hurlement', V.hurlement.cadence) : 1;
  }

  override vitesseAllie(a: Allie): number {
    return this.ruee > 0 && a.definition === ALLIES.loup ? V.ruee.vitesse : 1;
  }

  override surMorsure(a: Allie, e: Ennemi): void {
    const t = this.t;
    if (a.definition !== ALLIES.loup || e.mort >= 0) return;
    // Morsure marquante (Traque : plus longue)
    const marque = t.valeur('morsure_marquante', V.morsure.marque);
    if (marque > 0) t.contexte.ennemis.marquer(e, marque, V.morsure.duree + (t.rang('traque') > 0 ? V.morsure.traque : 0));
    // Encerclement : 3 loups différents en moins de 2 s
    if (t.rang('encerclement') > 0 && a.id !== e.morsureA && a.id !== e.morsureB) {
      const maintenant = t.etat.tempsJeu;
      if (e.tempsMorsureA >= 0 && e.tempsMorsureB >= 0 && maintenant - Math.min(e.tempsMorsureA, e.tempsMorsureB) <= V.encerclement.fenetre) {
        e.encercle = V.encerclement.duree;
      }
      e.morsureB = e.morsureA;
      e.tempsMorsureB = e.tempsMorsureA;
      e.morsureA = a.id;
      e.tempsMorsureA = maintenant;
    }
  }

  override surMortAllie(a: Allie): boolean {
    const t = this.t;
    const { particules, retours } = t.contexte;
    // Gardien alpha : l'alpha revient une fois par invocation
    if (a.alpha && !a.revenu && t.rang('gardien_alpha') > 0) {
      a.revenu = true;
      a.pv = a.pvMax;
      particules.emettre(a.x, 1, a.z, GERBES.totem);
      retours.son('totem');
      return true;
    }
    // Lien spirituel : un allié tué donne du bouclier
    if (t.rang('lien_spirituel') > 0 && this.delaiLien <= 0) {
      this.delaiLien = V.lien.delaiInterne;
      t.bouclier(V.lien.bouclier);
      const j = t.etat.joueur;
      particules.emettre(j.x, 1.4, j.z, GERBES.bouclier);
    }
    // Réactivité : son arme revient plus tôt
    const avance = t.valeur('reactivite', V.reactivite.avance);
    if (avance > 0 && a.arme) a.arme.minuterie *= 1 - avance;
    // Essaim : un vex tué explose
    const essaim = t.valeur('essaim', V.essaim.degats);
    if (essaim > 0 && a.definition === ALLIES.vex && a.arme) {
      t.blesserZone(a.x, a.z, V.essaim.rayon, essaim, VEX, 3, this.tampon);
      particules.emettre(a.x, 1, a.z, GERBES.explosionArme);
    }
    return false;
  }

  // --- Coups et défense -----------------------------------------------------

  override multiplicateur(e: Ennemi, arme: number, nature: NatureCoup): number {
    const t = this.t;
    let m = 1;
    if (e.encercle > 0) m += t.valeur('encerclement', V.encerclement.degats);
    if (nature !== 'allie') return m;
    const meute = t.valeur('meute_soudee', V.meute.parAllie);
    if (meute > 0) m += Math.min(t.valeur('meute_soudee', V.meute.maximum), meute * this.vivants);
    if ((arme === VEX || arme === BLAZE) && e.marqueTemps > 0) m += t.valeur('coordination', V.coordination.degats);
    return m;
  }

  override apresCoup(e: Ennemi, _degats: number, _arme: number, _critique: boolean, nature: NatureCoup): void {
    // Ruée de la meute : 15 coups sur des cibles marquées
    const t = this.t;
    if (nature !== 'allie' || e.marqueTemps <= 0 || t.rang('ruee_meute') === 0 || this.delaiRuee > 0) return;
    if (++this.coupsMarques < V.ruee.coups) return;
    this.coupsMarques = 0;
    this.delaiRuee = V.ruee.delaiInterne;
    this.ruee = V.ruee.duree;
    this.hurler();
    const loups = this.arme(LOUPS);
    if (loups) t.systemes.invocations.invoquerTemporaire(loups, V.ruee.fantomes, V.ruee.duree, 1);
  }

  override surMort(e: Ennemi, _arme: number, _nature: NatureCoup): void {
    // Curée : tuer une cible marquée soigne les alliés proches
    const t = this.t;
    if (e.marqueTemps <= 0 || t.rang('curee') === 0) return;
    const invocations = t.systemes.invocations;
    const maintenant = t.etat.tempsJeu;
    for (let i = 0; i < invocations.nombre; i++) {
      const a = invocations.liste[i];
      if (!a.arme || a.pv <= 0 || maintenant - a.dernierSoin < V.curee.delai) continue;
      if ((a.x - e.x) ** 2 + (a.z - e.z) ** 2 > V.curee.rayon ** 2) continue;
      a.pv = Math.min(a.pvMax, a.pv + a.pvMax * V.curee.soin);
      a.dernierSoin = maintenant;
      t.contexte.particules.emettre(a.x, 1, a.z, GERBES.niveau);
    }
  }

  override armure(): number {
    const t = this.t;
    if (t.rang('garde_rapprochee') === 0) return 0;
    const j = t.etat.joueur;
    return t.systemes.invocations.allieProche(j.x, j.z, V.garde.rayon) ? t.valeur('garde_rapprochee', V.garde.armure) : 0;
  }

  override apresDeclenchement(arme: ArmeEquipee, declenchee: boolean): void {
    // Évocation supérieure : chaque déclenchement d'une invocation compte (pas l'allay)
    const t = this.t;
    const m = arme.definition.mecanique;
    if (!declenchee || m.type !== 'invocation' || ALLIES[m.allie].soutien || t.rang('evocation_superieure') === 0) return;
    if (this.declenchements < V.evocation.declenchements) this.declenchements++;
    if (this.declenchements >= V.evocation.declenchements && !this.ravageur.actif && this.delaiEvocation <= 0) {
      this.declenchements = 0;
      this.delaiEvocation = V.evocation.delaiInterne;
      this.invoquerRavageur();
    }
  }

  // --- Ravageur ---------------------------------------------------------------

  private invoquerRavageur(): void {
    const t = this.t;
    const j = t.etat.joueur;
    const r = this.ravageur;
    r.actif = true;
    r.temps = 0;
    r.cap = 0;
    r.x = j.x - Math.sin(j.angle) * 2;
    r.z = j.z - Math.cos(j.angle) * 2;
    this.viserGroupe();
    t.contexte.particules.emettre(r.x, 1, r.z, GERBES.invocation);
    t.contexte.retours.son('ravageur');
  }

  private viserGroupe(): void {
    const r = this.ravageur;
    const ennemis = this.t.contexte.ennemis;
    if (!ennemis.plusGrosGroupe(r.x, r.z, 14, 3, groupe)) return;
    const d = longueur(groupe.x - r.x, groupe.z - r.z);
    if (d < 0.5) return;
    r.dirX = (groupe.x - r.x) / d;
    r.dirZ = (groupe.z - r.z) / d;
  }

  private majRavageur(dt: number): void {
    const r = this.ravageur;
    if (!r.actif) return;
    const t = this.t;
    const { ennemis, carte, particules, retours, etat } = t.contexte;
    r.temps += dt;
    if (r.temps >= V.evocation.duree) {
      r.actif = false;
      particules.emettre(r.x, 1, r.z, GERBES.invocation);
      return;
    }
    // Charge en ligne droite, change de cap vers le plus gros groupe de temps en temps ou contre un mur
    if ((r.cap -= dt) <= 0) {
      r.cap = V.evocation.cap;
      this.viserGroupe();
    }
    const nx = r.x + r.dirX * V.evocation.vitesse * dt;
    const nz = r.z + r.dirZ * V.evocation.vitesse * dt;
    if (carte.bloqueMarche(nx, nz)) {
      r.dirX = -r.dirX;
      r.dirZ = -r.dirZ;
      r.cap = 0.3;
    } else {
      r.x = nx;
      r.z = nz;
    }
    r.vitesseSol = V.evocation.vitesse;
    if ((r.pas -= dt) <= 0) {
      r.pas = 0.35;
      retours.son('ravageur_pas');
      particules.emettre(r.x, 0.2, r.z, GERBES.repousse);
    }
    // Renverse tout sur son passage (une fois par ennemi et par instant)
    const n = ennemis.chercher(r.x, r.z, V.evocation.rayon, this.tampon);
    for (let k = 0; k < n; k++) {
      const e = ennemis.obtenir(this.tampon[k]);
      if (e.immunites[CREDIT] > etat.tempsJeu) continue;
      e.immunites[CREDIT] = etat.tempsJeu + V.evocation.immunite;
      // Projeté sur le côté de la trajectoire
      const cote = (e.x - r.x) * r.dirZ - (e.z - r.z) * r.dirX >= 0 ? 1 : -1;
      t.blesser(e, V.evocation.degats, e.x - r.dirZ * cote - r.dirX, e.z + r.dirX * cote - r.dirZ, V.evocation.recul, CREDIT);
    }
  }

  // --- Crocs de l'évocateur --------------------------------------------------

  private majCrocs(dt: number): void {
    const t = this.t;
    if (t.rang('crocs_evocateur') === 0) return;
    const { ennemis, particules, retours } = t.contexte;
    const j = t.etat.joueur;
    if ((this.minuterieCrocs -= dt) <= 0) {
      this.minuterieCrocs = V.crocs.periode;
      // Une rangée vers le groupe le plus proche
      if (ennemis.plusGrosGroupe(j.x, j.z, V.crocs.portee, 3, groupe)) {
        const angle = Math.atan2(groupe.x - j.x, groupe.z - j.z);
        for (let k = 0; k < V.crocs.nombre; k++) {
          const c = this.crocs.acquerir();
          if (!c) break;
          const d = V.crocs.depart + k * V.crocs.ecart;
          c.x = j.x + Math.sin(angle) * d;
          c.z = j.z + Math.cos(angle) * d;
          c.angle = angle + (alea() - 0.5) * 0.6;
          c.t = -k * V.crocs.decalage;
          c.frappe = false;
        }
        retours.son('incantation');
      }
    }
    for (let i = this.crocs.nombre - 1; i >= 0; i--) {
      const c = this.crocs.elements[i];
      c.t += dt;
      if (c.t < 0) continue;
      if (!c.frappe && c.t >= V.crocs.claque) {
        c.frappe = true;
        t.blesserZone(c.x, c.z, V.crocs.rayon, V.crocs.degats, CREDIT, 2, this.tampon);
        particules.emettre(c.x, 0.4, c.z, GERBES.critique);
        retours.son('crocs');
      }
      if (c.t >= V.crocs.vie) this.crocs.liberer(i);
    }
  }

  override majVisuel(ombres: Ombres): void {
    const r = this.ravageur;
    const rendu = this.renduRavageur;
    if (rendu) {
      rendu.instances.commencer();
      if (r.actif) {
        avancerAnimation(r.animation, rendu.modele, 1 / 60, r.vitesseSol);
        const p = rendu.pose;
        p.x = r.x;
        p.z = r.z;
        p.angle = Math.atan2(r.dirX, r.dirZ);
        calculerRotations(rendu.modele, r.animation, p.rotations);
        rendu.instances.ajouter(p);
        ombres.ajouter(r.x, r.z, CREATURES_TALENTS.ravageur.rayonOmbre);
      }
      rendu.instances.terminer();
    }
    const crocs = this.renduCrocs;
    if (!crocs) return;
    crocs.instances.commencer();
    const p = crocs.pose;
    for (let i = 0; i < this.crocs.nombre; i++) {
      const c = this.crocs.elements[i];
      if (c.t < 0) continue;
      // Sort de terre, mâchoires grandes ouvertes, claque, puis s'enfonce
      const vie = V.crocs.vie;
      const sortie = Math.min(1, c.t / 0.12, (vie - c.t) / 0.25);
      const ouverture = c.t < V.crocs.claque ? 1 : Math.max(0, 1 - (c.t - V.crocs.claque) / 0.08);
      p.x = c.x;
      p.z = c.z;
      p.y = -0.9 * (1 - sortie);
      p.angle = c.angle;
      p.rotations.set(crocs.modele.poseBase);
      p.rotations[crocs.machoireA * 3 + 2] = -ouverture * 1.1;
      p.rotations[crocs.machoireB * 3 + 2] = ouverture * 1.1;
      crocs.instances.ajouter(p);
    }
    crocs.instances.terminer();
  }

  // --- Outils -----------------------------------------------------------------

  private hurler(): void {
    const t = this.t;
    const invocations = t.systemes.invocations;
    for (let i = 0; i < invocations.nombre; i++) {
      const a = invocations.liste[i];
      if (a.arme && a.pv > 0 && a.definition === ALLIES.loup) t.contexte.particules.emettre(a.x, 1.2, a.z, GERBES.critique);
    }
    t.contexte.retours.son('loup_hurlement');
  }

  private arme(index: number): ArmeEquipee | null {
    for (const a of this.t.arsenal.armes) if (a.index === index) return a;
    return null;
  }

}
