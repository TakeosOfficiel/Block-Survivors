/**
 * Partie à plusieurs, côté hôte : il simule tout. Les joueurs distants sont
 * des combattants (combattant.ts) mis à jour juste après le joueur local ; les
 * ennemis, leurs flèches, les orbes, les objets et les coffres visent le joueur
 * vivant le plus proche (EquipeJoueurs). Plusieurs fois par seconde, chaque
 * joueur reçoit un instantané de ce qui l'entoure.
 *
 * Règles à plusieurs : XP et niveau communs ; à chaque niveau et à chaque
 * coffre, tout le monde choisit ses cartes, sans attendre les autres (celui
 * qui choisit est protégé, les ennemis l'ignorent, ses armes se reposent) ;
 * un joueur tombé reste à terre un moment, un allié qui reste près de lui le
 * relève ; défaite quand toute l'équipe est tombée.
 */
import { IDS_ARMES } from '../donnees/armes';
import { ENNEMIS } from '../donnees/ennemis';
import { facteurEquipe, REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import { GERBES } from '../donnees/particules';
import { IDS_PASSIFS } from '../donnees/passifs';
import { PROGRESSION } from '../donnees/progression';
import type { StyleChiffre } from '../effets/chiffresDegats';
import type { Ombres } from '../effets/ombres';
import type { RenduProjectiles } from '../effets/renduProjectiles';
import type { Retours } from '../effets/retours';
import type { ContexteComportement } from '../entites/comportements';
import type { EquipeJoueurs } from '../entites/types';
import { minutesSansFin, type EtatJoueur } from '../etat';
import type { ChampDeFlux } from '../monde/navigation';
import type { Options } from '../options';
import type { Amelioration } from '../progression/ameliorations';
import { Bandeau, texteATerre } from './bandeau';
import { Combattant, Commutateur, type PartageCombat } from './combattant';
import type { CrochetsReseau, ElementsPartie } from './partie';
import {
  BITS_ACTIONS, ENNEMI_BLESSE, ENNEMI_BRULURE, ENNEMI_MORT, ENNEMI_POISON, ENNEMI_RALENTI, ENNEMI_SAIGNEMENT, Ecrivain,
  JOUEUR_EN_CHOIX, JOUEUR_MORT, JOUEUR_PROTEGE, Lecteur, STYLES_CHIFFRES, TYPE_ENTREE, TYPE_INSTANTANE,
  statistiquesReseau, type MessageFiable,
} from './protocole';
import type { ObservateurPartie, SessionReseau } from './session';

/** Projectiles ennemis et chiffres de dégâts retenus d'une image à l'envoi suivant. */
const CAPACITE_PROJECTILES = 2048;
const CAPACITE_CHIFFRES = 256;
/** Sons d'un joueur distant qu'on n'entend pas chez l'hôte (ils concernent son corps à lui). */
const SONS_PERSONNELS = new Set(['blessure_joueur', 'totem', 'orbe', 'emeraude', 'manger', 'niveau']);

export class PartieHote implements CrochetsReseau, ObservateurPartie, EquipeJoueurs {
  readonly client = false;
  readonly facteurApparitions: number;
  pauseLocale = false;

  private readonly session: SessionReseau;
  private readonly elements: ElementsPartie;
  private readonly commutateur: Commutateur;
  /** Joueurs distants, et le même par index de joueur (0 : l'hôte, toujours null). */
  readonly combattants: Combattant[] = [];
  private readonly parIndex: (Combattant | null)[] = [];
  private readonly contextes: ContexteComportement[] = [];
  /** Réanimation en cours de chaque joueur de l'équipe (s), par k (0 : l'hôte). */
  private readonly reanimations = new Float32Array(REGLES_MULTIJOUEUR.joueursMax);
  private readonly bandeau = new Bandeau();
  private readonly ecrivain = new Ecrivain(96 * 1024);
  private readonly lecteur = new Lecteur();
  /** Projectiles ennemis dessinés cette image : index de rendu, x, y, z, angle. */
  private readonly projectiles = new Float32Array(CAPACITE_PROJECTILES * 5);
  private nbProjectiles = 0;
  private capture = false;
  private captureBranchee = false;
  /** Chiffres de dégâts depuis le dernier envoi : x, y, z, valeur, style. */
  private readonly chiffres = new Float32Array(CAPACITE_CHIFFRES * 5);
  private nbChiffres = 0;
  private numero = 0;
  private prochainEnvoi = 0;
  private prochainesStats = 0;
  private finEnvoyee = false;
  /** Cartes de coffre déjà distribuées (pour repérer un coffre qui vient de s'ouvrir). */
  private coffresVus = 0;
  private effetsAllies = true;
  private dtReel = 0;
  private minuterieBouclier = 0;

  constructor(session: SessionReseau, elements: ElementsPartie) {
    this.session = session;
    this.elements = elements;
    const { etat, ennemis, projectilesEnnemis } = elements;
    this.commutateur = new Commutateur(etat, ennemis, elements.mecaniques);
    const partage: PartageCombat = {
      etat,
      carte: elements.terrain.carte,
      ennemis,
      projectilesEnnemis,
      orbes: elements.orbes,
      particules: elements.particules,
      scene: elements.scene,
      camera: elements.camera,
      textures: elements.textures,
      commutateur: this.commutateur,
    };
    // Ce qui arrive aux joueurs distants ne secoue pas la caméra de l'hôte
    const retours = elements.retours;
    const retoursDistants: Retours = {
      son: (id, hauteur) => {
        if (!SONS_PERSONNELS.has(id)) retours.son(id, hauteur);
      },
      secousse: () => {},
      degats: (x, y, z, valeur, style) => retours.degats(x, y, z, valeur, style),
    };

    this.contextes.push({ cible: elements.cible, tireur: projectilesEnnemis });
    for (const info of session.joueurs) {
      if (info.index === 0) continue;
      const c = new Combattant(info, partage, retoursDistants);
      this.combattants.push(c);
      this.parIndex[info.index] = c;
      etat.joueurs.push(c.etatJoueur);
      // Coups des ennemis : de plus en plus forts en mode sans fin, comme pour le joueur local
      this.contextes.push({
        cible: { subirDegats: (degats, boss) => c.subirDegats(degats * (1 + minutesSansFin(etat) * PROGRESSION.sansFin.degatsParMinute), boss) },
        tireur: projectilesEnnemis,
      });
    }
    ennemis.equipe = this;
    projectilesEnnemis.equipe = this;
    elements.orbes.equipe = this;
    elements.objets.equipe = this;
    elements.coffres.equipe = this;

    // Difficulté selon le nombre de joueurs
    const n = this.nombre;
    const r = REGLES_MULTIJOUEUR.parJoueur;
    this.facteurApparitions = facteurEquipe(r.apparitions, n);
    const pvEnnemis = facteurEquipe(r.pvEnnemis, n);
    const pvBoss = facteurEquipe(r.pvBoss, n);
    // Prototype : les méthodes de l'instance sont enveloppées plutôt que de modifier ces systèmes
    const apparaitre = ennemis.apparaitre.bind(ennemis);
    ennemis.apparaitre = (id, x, z, multiplicateurPv = 1) => {
      const def = ENNEMIS[id];
      return apparaitre(id, x, z, multiplicateurPv * (def.boss || def.miniBoss ? pvBoss : pvEnnemis));
    };
    // Les ennemis n'apparaissent pas sous les yeux d'un joueur distant non plus
    const camera = elements.cameraJeu;
    const estVisible = camera.estVisible.bind(camera);
    const rayonVue = REGLES_MULTIJOUEUR.reseau.rayonVue;
    camera.estVisible = (x, z, rayon) => {
      if (estVisible(x, z, rayon)) return true;
      for (let i = 0; i < this.combattants.length; i++) {
        const j = this.combattants[i].etatJoueur;
        if (!j.mort && (j.x - x) ** 2 + (j.z - z) ** 2 < (rayonVue + rayon) ** 2) return true;
      }
      return false;
    };
    // Seuls les projectiles ennemis partent dans les instantanés : chaque joueur simule les effets des armes
    const majVisuel = projectilesEnnemis.majVisuel.bind(projectilesEnnemis);
    projectilesEnnemis.majVisuel = (rendu) => {
      this.capture = true;
      majVisuel(rendu);
      this.capture = false;
    };
    session.partie = this;
  }

  // --- EquipeJoueurs -------------------------------------------------------

  get nombre(): number {
    return 1 + this.combattants.length;
  }

  joueur(k: number): EtatJoueur {
    return k === 0 ? this.commutateur.local.joueur : this.combattants[k - 1].etatJoueur;
  }

  contexte(k: number): ContexteComportement {
    return this.contextes[k];
  }

  flux(k: number): ChampDeFlux {
    return k === 0 ? this.elements.terrain.flux : this.combattants[k - 1].flux;
  }

  /** Le plus proche des joueurs debout qui ne choisissent pas leurs cartes. */
  plusProche(x: number, z: number): number {
    let meilleur = -1;
    let meilleureD2 = Infinity;
    for (let k = 0; k < this.nombre; k++) {
      const j = this.joueur(k);
      if (j.mort || this.enChoix(k)) continue;
      const d2 = (j.x - x) ** 2 + (j.z - z) ** 2;
      if (d2 < meilleureD2) {
        meilleureD2 = d2;
        meilleur = k;
      }
    }
    return meilleur;
  }

  private enChoix(k: number): boolean {
    return k === 0 ? this.pauseLocale : this.combattants[k - 1].enChoix;
  }

  // --- Simulation ----------------------------------------------------------

  majJoueurs(dt: number): void {
    const maintenant = performance.now() / 1000;
    const protection = REGLES_MULTIJOUEUR.choixCartes.protection;
    const local = this.commutateur.local.joueur;
    if (this.pauseLocale) local.protection = Math.max(local.protection, protection);
    for (let i = 0; i < this.combattants.length; i++) {
      const c = this.combattants[i];
      if (c.parti) continue;
      if (maintenant - c.instantEntree > REGLES_MULTIJOUEUR.reseau.silenceEntrees) c.commandes.bits = 0;
      c.enChoix = c.cartesEnAttente > 0;
      if (c.enChoix) {
        c.etatJoueur.protection = Math.max(c.etatJoueur.protection, protection);
        c.commandes.bits = 0;
      }
      c.majJoueur(dt);
    }
  }

  majArmes(dt: number): void {
    for (let i = 0; i < this.combattants.length; i++) {
      const c = this.combattants[i];
      if (!c.parti && !c.enChoix) c.majArmes(dt);
    }
  }

  finPas(dt: number): void {
    const etat = this.elements.etat;
    // Niveau commun : l'XP va au joueur local (orbes), les autres le suivent et choisissent leurs cartes
    const local = this.commutateur.local.joueur;
    for (let i = 0; i < this.combattants.length; i++) {
      const c = this.combattants[i];
      const j = c.etatJoueur;
      if (!c.parti && local.niveau > j.niveau) c.cartesEnAttente += local.niveau - j.niveau;
      j.niveau = local.niveau;
      j.xp = local.xp;
      j.xpNiveau = local.xpNiveau;
    }
    // Coffre ouvert (par n'importe qui) : des cartes pour tout le monde
    const coffre = etat.choixCoffre;
    if (coffre.restants > this.coffresVus) {
      const nombre = coffre.restants - this.coffresVus;
      for (let i = 0; i < this.combattants.length; i++) if (!this.combattants[i].parti) this.combattants[i].cartesEnAttente += nombre;
      this.session.envoyer({ type: 'coffre', nombre, titre: coffre.titre });
    }
    this.coffresVus = coffre.restants;

    this.reanimer(dt);

    if (etat.issue !== null && !this.finEnvoyee) {
      this.finEnvoyee = true;
      for (let i = 0; i < this.combattants.length; i++) {
        const c = this.combattants[i];
        this.session.envoyer({ type: 'fin', issue: etat.issue, stats: statistiquesReseau(c.profil.stats) }, c.info.index);
      }
    } else if (etat.issue === null) {
      this.finEnvoyee = false;
    }
  }

  /** Un joueur à terre se relève quand un allié debout reste assez longtemps près de lui. */
  private reanimer(dt: number): void {
    const r = REGLES_MULTIJOUEUR.reanimation;
    for (let k = 0; k < this.nombre; k++) {
      const j = this.joueur(k);
      if (!j.mort || j.tempsMort >= r.aTerre || (k > 0 && this.combattants[k - 1].parti)) {
        this.reanimations[k] = 0;
        continue;
      }
      let aide = false;
      for (let m = 0; m < this.nombre && !aide; m++) {
        const a = this.joueur(m);
        aide = m !== k && !a.mort && (a.x - j.x) ** 2 + (a.z - j.z) ** 2 <= r.rayon * r.rayon;
      }
      this.reanimations[k] = aide ? this.reanimations[k] + dt : Math.max(0, this.reanimations[k] - dt);
      if (this.reanimations[k] < r.duree) continue;
      this.reanimations[k] = 0;
      j.mort = false;
      j.tempsMort = 0;
      j.pv = Math.max(1, Math.ceil(j.pvMax * r.pv));
      j.protection = r.protection;
      this.elements.particules.emettre(j.x, 1.2, j.z, GERBES.totem);
      this.elements.retours.son('totem');
    }
    for (let i = 0; i < this.combattants.length; i++) this.combattants[i].reanimation = this.reanimations[i + 1] / r.duree;
  }

  simulerClient(): void {}

  vitesseTemps(cartesOuvertes: boolean): number {
    // La partie ne s'arrête plus pour les cartes : seul le joueur qui choisit se met à l'abri
    const etat = this.elements.etat;
    this.pauseLocale = etat.issue === null && (cartesOuvertes || etat.niveauxEnAttente > 0 || etat.choixCoffre.restants > 0);
    return 1;
  }

  surCarte(amelioration: Amelioration): void {
    this.session.envoyer({ type: 'carteJouee', index: 0, amelioration });
  }

  surContinuer(): void {
    this.session.envoyer({ type: 'reprise' });
  }

  appliquerOptions(options: Options): void {
    this.effetsAllies = options.effetsAllies;
  }

  surDegats(x: number, y: number, z: number, valeur: number, style: StyleChiffre): void {
    if (this.nbChiffres >= CAPACITE_CHIFFRES) return;
    const o = this.nbChiffres++ * 5;
    this.chiffres[o] = x;
    this.chiffres[o + 1] = y;
    this.chiffres[o + 2] = z;
    this.chiffres[o + 3] = valeur;
    this.chiffres[o + 4] = STYLES_CHIFFRES.indexOf(style);
  }

  // --- Messages des joueurs ------------------------------------------------

  surDonnees(index: number, donnees: ArrayBuffer): void {
    const c = this.parIndex[index];
    const l = this.lecteur;
    l.lire(donnees);
    if (!c || donnees.byteLength < 10 || l.u8() !== TYPE_ENTREE) return;
    const numero = l.u32();
    const bits = l.u8();
    const lacet = l.f32();
    if (numero <= c.derniereEntree) return;  // arrivée dans le désordre : périmée
    c.derniereEntree = numero;
    c.instantEntree = performance.now() / 1000;
    c.commandes.bits = bits & 15;
    if (Number.isFinite(lacet)) c.joueur.lacetCommandes = lacet;
  }

  surMessage(index: number, message: MessageFiable): void {
    const c = this.parIndex[index];
    if (!c) return;
    if (message.type === 'continuer') {
      if (this.elements.etat.issue === 'victoire') this.elements.continuerSansFin();
      return;
    }
    if (message.type !== 'carte' || c.cartesEnAttente <= 0 || !ameliorationValide(message.amelioration)) return;
    c.cartesEnAttente--;
    try {
      c.appliquerCarte(message.amelioration);
    } catch {
      return;  // carte devenue impossible (désaccord avec le tirage du joueur) : ignorée
    }
    // Les autres gardent son arsenal à jour (ils simulent les effets de ses armes)
    for (const autre of this.combattants) {
      if (autre !== c && !autre.parti) this.session.envoyer({ type: 'carteJouee', index, amelioration: message.amelioration }, autre.info.index);
    }
  }

  /** Un joueur parti : il tombe et ne compte plus (ni cible, ni instantanés). */
  surDepart(index: number): void {
    const c = this.parIndex[index];
    if (!c) return;
    c.parti = true;
    c.cartesEnAttente = 0;
    c.commandes.bits = 0;
    const j = c.etatJoueur;
    j.mort = true;
    j.tempsMort = REGLES_MULTIJOUEUR.reanimation.aTerre;
    j.x = 1e4;
    j.z = 1e4;
  }

  // --- Affichage -----------------------------------------------------------

  avantImage(dtReel: number): void {
    this.dtReel = dtReel;
    const local = this.commutateur.local.joueur;
    const r = REGLES_MULTIJOUEUR.reanimation;
    this.bandeau.afficher(texteATerre(local.mort, local.tempsMort, r.aTerre, this.reanimations[0] / r.duree));
  }

  majVisuelOmbres(ombres: Ombres): void {
    for (let i = 0; i < this.combattants.length; i++) {
      const c = this.combattants[i];
      if (!c.parti) c.majVisuelOmbres(ombres, this.dtReel, this.effetsAllies);
    }
    // Bouclier du joueur local pendant qu'il choisit ses cartes
    if (this.pauseLocale && (this.minuterieBouclier -= this.dtReel) <= 0) {
      this.minuterieBouclier = REGLES_MULTIJOUEUR.choixCartes.periodeBouclier;
      const j = this.commutateur.local.joueur;
      this.elements.particules.emettre(j.x, 1, j.z, GERBES.arcane);
    }
  }

  majVisuelProjectiles(rendu: RenduProjectiles): void {
    if (!this.captureBranchee) this.brancherCapture(rendu);
    if (!this.effetsAllies) return;
    for (let i = 0; i < this.combattants.length; i++) if (!this.combattants[i].parti) this.combattants[i].majVisuelProjectiles(rendu);
  }

  majVisuelEffets(dtJeu: number): void {
    if (this.effetsAllies) {
      for (let i = 0; i < this.combattants.length; i++) if (!this.combattants[i].parti) this.combattants[i].majVisuelEffets(dtJeu);
    }
    const maintenant = performance.now() / 1000;
    // Bilan de chaque joueur (son compteur de dégâts)
    if (maintenant >= this.prochainesStats) {
      this.prochainesStats = maintenant + REGLES_MULTIJOUEUR.reseau.periodeStats;
      for (const c of this.combattants) {
        if (!c.parti) this.session.envoyer({ type: 'stats', stats: statistiquesReseau(c.profil.stats) }, c.info.index);
      }
    }
    if (maintenant < this.prochainEnvoi) return;
    this.prochainEnvoi = Math.max(this.prochainEnvoi + 1 / REGLES_MULTIJOUEUR.reseau.instantanesParSeconde, maintenant - 0.05);
    this.numero++;
    for (let i = 0; i < this.combattants.length; i++) {
      const c = this.combattants[i];
      if (c.parti) continue;
      this.ecrireInstantane(c);
      this.session.envoyerDonnees(this.ecrivain.contenu(), c.info.index);
    }
    this.nbChiffres = 0;
  }

  /** Retient chaque projectile ennemi dessiné (le rendu des projectiles sert de liste commune). */
  private brancherCapture(rendu: RenduProjectiles): void {
    this.captureBranchee = true;
    const ajouter = rendu.ajouter.bind(rendu);
    const commencer = rendu.commencer.bind(rendu);
    rendu.ajouter = (indexType, x, y, z, angle) => {
      if (this.capture && this.nbProjectiles < CAPACITE_PROJECTILES) {
        const o = this.nbProjectiles++ * 5;
        this.projectiles[o] = indexType;
        this.projectiles[o + 1] = x;
        this.projectiles[o + 2] = y;
        this.projectiles[o + 3] = z;
        this.projectiles[o + 4] = angle;
      }
      ajouter(indexType, x, y, z, angle);
    };
    rendu.commencer = () => {
      this.nbProjectiles = 0;
      commencer();
    };
  }

  /** Ce que voit le joueur `c` : l'équipe entière, puis ce qui est à moins de `rayonEnvoi` de lui. */
  private ecrireInstantane(c: Combattant): void {
    const w = this.ecrivain;
    const { etat, ennemis, orbes, objets, coffres } = this.elements;
    const cx = c.etatJoueur.x;
    const cz = c.etatJoueur.z;
    const r2 = REGLES_MULTIJOUEUR.reseau.rayonEnvoi ** 2;
    const local = this.commutateur.local.joueur;
    const dureeReanimation = REGLES_MULTIJOUEUR.reanimation.duree;

    w.position = 0;
    w.u8(TYPE_INSTANTANE);
    w.u32(this.numero);
    w.u32(c.derniereEntree);
    w.f32(etat.tempsJeu);
    w.u32(etat.ennemisTues);
    w.u16(local.niveau);
    w.f32(local.xp);
    w.f32(local.xpNiveau);
    w.u32(etat.emeraudes);

    // Équipe
    w.u8(this.nombre);
    for (let k = 0; k < this.nombre; k++) {
      const distant = k === 0 ? null : this.combattants[k - 1];
      const j = this.joueur(k);
      w.u8(distant ? distant.info.index : 0);
      w.f32(j.x);
      w.f32(j.z);
      w.angle(j.angle);
      w.f32(j.pv);
      w.f32(j.pvMax);
      w.f32(j.bouclier);
      w.u8((j.mort ? JOUEUR_MORT : 0) | (j.protection > 0 ? JOUEUR_PROTEGE : 0) | (this.enChoix(k) ? JOUEUR_EN_CHOIX : 0));
      w.f32(j.tempsMort);
      w.f32(j.invulnerabilite);
      w.u8(distant ? distant.commandes.bits : this.bitsLocaux());
      w.f32(distant ? distant.joueur.lacetCommandes : this.elements.joueur.lacetCommandes);
      w.part(Math.min(1, this.reanimations[k] / dureeReanimation));
    }

    // Ennemis
    let compte = w.reserver16();
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

    // Orbes d'XP, objets au sol et coffres
    compte = w.reserver16();
    n = 0;
    for (let i = 0; i < orbes.pool.nombre && w.reste > 12 * 1024; i++) {
      const o = orbes.pool.elements[i];
      if ((o.x - cx) ** 2 + (o.z - cz) ** 2 > r2) continue;
      w.coordonnee(o.x);
      w.coordonnee(o.z);
      w.u16(Math.min(65535, Math.round(o.valeur)));
      n++;
    }
    w.ecrire16(compte, n);
    compte = w.reserver16();
    n = 0;
    for (let i = 0; i < objets.pool.nombre && w.reste > 8 * 1024; i++) {
      const o = objets.pool.elements[i];
      if ((o.x - cx) ** 2 + (o.z - cz) ** 2 > r2) continue;
      w.u8(o.indexType);
      w.coordonnee(o.x);
      w.coordonnee(o.z);
      w.u8(Math.min(255, o.valeur));
      n++;
    }
    w.ecrire16(compte, n);
    w.u16(coffres.ouverts);
    compte = w.reserver16();
    n = 0;
    for (let i = 0; i < coffres.pool.nombre && w.reste > 6 * 1024; i++) {
      const o = coffres.pool.elements[i];
      w.u8(o.indexType);
      w.coordonnee(o.x);
      w.coordonnee(o.z);
      w.angle(o.angle);
      w.f32(o.ouvert);
      n++;
    }
    w.ecrire16(compte, n);

    // Projectiles ennemis, tels que dessinés
    compte = w.reserver16();
    n = 0;
    for (let i = 0; i < this.nbProjectiles && w.reste > 2 * 1024; i++) {
      const o = i * 5;
      const x = this.projectiles[o + 1];
      const z = this.projectiles[o + 3];
      if ((x - cx) ** 2 + (z - cz) ** 2 > r2) continue;
      w.u8(this.projectiles[o]);
      w.coordonnee(x);
      w.u8(Math.min(255, Math.round(this.projectiles[o + 2] * 32)));
      w.coordonnee(z);
      w.angle(this.projectiles[o + 4]);
      n++;
    }
    w.ecrire16(compte, n);

    // Chiffres de dégâts
    compte = w.reserver16();
    n = 0;
    for (let i = 0; i < this.nbChiffres && n < REGLES_MULTIJOUEUR.reseau.chiffresMax && w.reste > 64; i++) {
      const o = i * 5;
      if ((this.chiffres[o] - cx) ** 2 + (this.chiffres[o + 2] - cz) ** 2 > r2) continue;
      w.coordonnee(this.chiffres[o]);
      w.u8(Math.min(255, Math.round(this.chiffres[o + 1] * 16)));
      w.coordonnee(this.chiffres[o + 2]);
      w.f32(this.chiffres[o + 3]);
      w.u8(this.chiffres[o + 4]);
      n++;
    }
    w.ecrire16(compte, n);
  }

  private bitsLocaux(): number {
    const entrees = this.elements.entrees;
    return (entrees.action('haut') ? BITS_ACTIONS.haut : 0) | (entrees.action('bas') ? BITS_ACTIONS.bas : 0)
      | (entrees.action('gauche') ? BITS_ACTIONS.gauche : 0) | (entrees.action('droite') ? BITS_ACTIONS.droite : 0);
  }
}

/** Une carte reçue d'un joueur ne vise que des armes et des bonus qui existent. */
export function ameliorationValide(a: Amelioration): boolean {
  if (!a || typeof a !== 'object') return false;
  switch (a.type) {
    case 'arme': return IDS_ARMES.includes(a.id);
    case 'eveil': return IDS_ARMES.includes(a.id) && (a.variante === 0 || a.variante === 1);
    case 'passif': return IDS_PASSIFS.includes(a.id);
    case 'soin': return true;
    default: return false;
  }
}
