/**
 * Une partie : assemble les systèmes et fixe l'ordre de mise à jour.
 *
 * Convention : maj(dt) = simulation en temps de jeu (ne touche qu'à l'état),
 * majVisuel(...) = recopie de l'état dans la scène et le DOM.
 */
import * as THREE from 'three';
import type { LecteurSons } from './audio/sons';
import type { BibliothequeTextures } from './assets/textures';
import { cumulerBonusArmes } from './combat/armeEquipee';
import { mecaniquesClasse } from './combat/talents/classes';
import { Talents } from './combat/talents/talents';
import { Arsenal } from './combat/arsenal';
import { Balayages } from './combat/balayages';
import { Chutes } from './combat/chutes';
import { Fissures } from './combat/fissures';
import { Heurts } from './combat/heurts';
import { Meteores } from './combat/meteores';
import { Novas } from './combat/novas';
import { OrbesArcaniques } from './combat/orbesArcaniques';
import { Pieges } from './combat/pieges';
import { Pluies } from './combat/pluies';
import { Ricochets } from './combat/ricochets';
import { Tornades } from './combat/tornades';
import { Tourbillons } from './combat/tourbillons';
import type { ContexteCombat } from './combat/contexte';
import { Foudre } from './combat/foudre';
import { Invocations } from './combat/invocations';
import { Orbites } from './combat/orbites';
import { ProjectilesJoueur } from './combat/projectilesJoueur';
import { CONFIG } from './config';
import { Entrees } from './controles/entrees';
import { CALENDRIER_BOSS, CALENDRIER_MINIBOSS } from './donnees/boss';
import type { IdEnnemi } from './donnees/ennemis';
import { IDS_OBJETS, OBJETS } from './donnees/objets';
import { GERBES } from './donnees/particules';
import { PROGRESSION } from './donnees/progression';
import type { IdSon } from './donnees/sons';
import { Ombres } from './effets/ombres';
import { BarresVie } from './effets/barresVie';
import { ChiffresDegats, type StyleChiffre } from './effets/chiffresDegats';
import { Etiquette } from './effets/etiquette';
import { Particules } from './effets/particules';
import { RenduProjectiles } from './effets/renduProjectiles';
import type { Retours } from './effets/retours';
import { Butin } from './entites/butin';
import { Coffres } from './entites/coffres';
import { DirecteurApparitions } from './entites/directeur';
import { DirecteurBoss } from './entites/directeurBoss';
import { Ennemis } from './entites/ennemis';
import { Joueur } from './entites/joueur';
import { Objets } from './entites/objets';
import { Orbes } from './entites/orbes';
import { ProjectilesEnnemis } from './entites/projectilesEnnemis';
import type { CibleJoueur, SourceEntites } from './entites/types';
import { creerEtatPartie, equipeVaincue, minutesSansFin, type EtatPartie } from './etat';
import { lancerPartie, retournerAuMenu, type ChoixPartie } from './lancement';
import { construireArene } from './monde/arene';
import { Vegetation } from './monde/vegetation';
import type { MondeGenere } from './monde/generation';
import { ChampDeFlux, type Terrain } from './monde/navigation';
import { Boucle, type CallbacksBoucle } from './moteur/boucle';
import { CameraJeu } from './moteur/camera';
import type { Rendu } from './moteur/rendu';
import { Ameliorations, type Carte } from './progression/ameliorations';
import { gagnerXp } from './progression/niveaux';
import type { Options } from './options';
import { apparenceJoueur, type Profil } from './profil';
import { cumulerModificateurs } from './progression/modificateurs';
import { Passifs } from './progression/passifs';
import type { Recompense, Sauvegarde } from './progression/sauvegarde';
import { remplirBilan, remplirPastilles } from './ui/bilan';
import { CompteurDegats } from './ui/compteurDegats';
import { PanneauDebug, type EtatDebug } from './ui/debug';
import { EcranCartes } from './ui/ecranCartes';
import { EcranFin } from './ui/ecranFin';
import { EcranPause } from './ui/ecranPause';
import { Hud } from './ui/hud';
import { vueSuivante, type PanneauOptions } from './ui/panneauOptions';
import { element } from './ui/dom';
import { brancherPartie, type CrochetsReseau } from './reseau/partie';
import type { SessionReseau } from './reseau/session';

/** Services créés avant la partie (dès le menu) et partagés avec elle. */
export interface ServicesJeu {
  readonly sons: LecteurSons;
  readonly options: Options;
  readonly panneauOptions: PanneauOptions;
  readonly sauvegarde: Sauvegarde;
  readonly profil: Profil;
  /** Partie à plusieurs (src/reseau) ; absente en solo. */
  readonly reseau?: SessionReseau;
}

export class Jeu implements CallbacksBoucle, Retours {
  private readonly rendu: Rendu;
  private readonly textures: BibliothequeTextures;
  private readonly choix: ChoixPartie;
  private readonly sons: LecteurSons;
  private readonly sauvegarde: Sauvegarde;
  /** Émeraudes et déblocages gagnés (fin de partie). */
  private recompense: Recompense | null = null;
  private readonly etat: EtatPartie;
  private readonly terrain: Terrain;
  /** Taille du décor à photographier (blocs) : l'arène, ou toute la construction d'une map Minecraft. */
  private readonly tailleDecor: number;
  private readonly vegetation: Vegetation;
  private readonly entrees = new Entrees();
  private readonly aideSouris = element('hud-aide-souris');
  private readonly hud: Hud;
  private readonly camera: CameraJeu;
  private readonly boucle: Boucle;

  // Effets
  private readonly ombres: Ombres;
  private readonly barresMiniBoss: BarresVie;
  private readonly particules: Particules;
  private readonly chiffres: ChiffresDegats;
  private readonly etiquette: Etiquette;
  private readonly renduProjectiles: RenduProjectiles;

  // Entités
  private readonly joueur: Joueur;
  private readonly ennemis: Ennemis;
  private readonly projectilesEnnemis: ProjectilesEnnemis;
  private readonly orbes: Orbes;
  private readonly objets: Objets;
  private readonly coffres: Coffres;
  private readonly directeur: DirecteurApparitions;
  private readonly directeurBoss: DirecteurBoss;
  private readonly directeurMiniBoss: DirecteurBoss;

  // Combat du joueur
  private readonly arsenal: Arsenal;
  private readonly mecaniques: Talents;
  private readonly balayages: Balayages;
  private readonly fissures: Fissures;
  private readonly chutes: Chutes;
  private readonly tourbillons: Tourbillons;
  private readonly heurts: Heurts;
  private readonly tornades: Tornades;
  private readonly novas: Novas;
  private readonly meteores: Meteores;
  private readonly orbesArcaniques: OrbesArcaniques;
  private readonly ricochets: Ricochets;
  private readonly pluies: Pluies;
  private readonly pieges: Pieges;
  private readonly projectilesJoueur: ProjectilesJoueur;
  private readonly orbites: Orbites;
  private readonly foudre: Foudre;
  private readonly invocations: Invocations;

  // Progression
  private readonly passifs: Passifs;
  private readonly ameliorations: Ameliorations;
  private readonly ecranCartes: EcranCartes;
  private readonly ecranPause: EcranPause;
  private readonly ecranFin: EcranFin;

  private readonly debug: PanneauDebug;
  /** Ennemis proches trouvés par le debug (tampon réutilisé). */
  private readonly prochesDebug = new Int32Array(256);
  private readonly compteurDegats: CompteurDegats;
  private readonly panneauOptions: PanneauOptions;
  private readonly sourcesEntites: readonly SourceEntites[];
  private niveauAffiche = 1;
  private viesTraitees = 0;
  /** s réelles écoulées depuis la fin de la partie (−1 tant qu'elle continue). */
  private minuterieFin = -1;
  /** 15:00 passé mais un boss vit encore : la victoire attend sa mort. */
  private prolongation = false;
  /** XP et boss déjà comptés par la victoire, quand la partie continue en mode sans fin. */
  private xpComptee = 0;
  private bossComptes = 0;
  /** Partie à plusieurs : points d'accroche du réseau (null en solo). */
  private readonly reseau: CrochetsReseau | null;

  constructor(
    rendu: Rendu,
    textures: BibliothequeTextures,
    monde: MondeGenere,
    choix: ChoixPartie,
    services: ServicesJeu,
  ) {
    this.rendu = rendu;
    this.textures = textures;
    this.choix = choix;
    this.sons = services.sons;
    this.sauvegarde = services.sauvegarde;
    const arene = choix.arene;
    this.etat = creerEtatPartie(arene, choix.personnage);
    const scene = rendu.scene;

    // Bonus permanents du personnage et de son arbre de talents
    const talents = this.sauvegarde.bonus(choix.personnage);
    const bonus = { ...this.etat.personnage.bonus };
    cumulerModificateurs(bonus, talents.modif);
    this.passifs = new Passifs(bonus);
    this.passifs.calculer(this.etat.modificateurs);
    this.etat.joueur.pvMax += this.etat.modificateurs.pvMax;
    this.etat.joueur.pv = this.etat.joueur.pvMax;
    this.etat.relancesRestantes += talents.relances;
    this.etat.talents = { armes: cumulerBonusArmes(talents.bonusArmes), mecaniques: talents.mecaniques };

    // Monde : blocs affichés, carte de collision, navigation des ennemis
    construireArene(arene, monde, textures, scene);
    rendu.appliquerAmbiance(arene);
    this.vegetation = new Vegetation(scene);
    this.terrain = { carte: monde.carte, flux: new ChampDeFlux(monde.carte) };
    const boite = monde.decor ? new THREE.Box3().setFromObject(monde.decor) : null;
    this.tailleDecor = boite && !boite.isEmpty()
      ? Math.max(boite.max.x - boite.min.x, boite.max.z - boite.min.z) * CONFIG.chargement.photo.cadreDecor
      : Math.max(arene.largeur, arene.profondeur);
    const carte = monde.carte;
    this.ombres = new Ombres(textures.obtenir(CONFIG.ombre.texture), scene);
    this.barresMiniBoss = new BarresVie(scene, rendu.camera, 8);
    this.particules = new Particules(scene, rendu.camera, textures, IDS_OBJETS.map((id) => OBJETS[id].icone));
    this.renduProjectiles = new RenduProjectiles(textures, scene, rendu.camera);
    this.chiffres = new ChiffresDegats(scene, rendu.camera);

    // Entités
    const apparence = apparenceJoueur(this.etat.personnage, services.profil);
    this.joueur = new Joueur(this.etat, carte, this.entrees, this, scene, textures, apparence);
    this.etiquette = new Etiquette(scene, services.profil.pseudo);
    // Coups des ennemis : de plus en plus forts en mode sans fin
    const cible: CibleJoueur = {
      subirDegats: (degats, boss) => this.joueur.subirDegats(degats * (1 + minutesSansFin(this.etat) * PROGRESSION.sansFin.degatsParMinute), boss),
    };
    this.projectilesEnnemis = new ProjectilesEnnemis(this.etat, carte, cible);
    this.orbes = new Orbes(this.etat, this, scene, textures, rendu.camera);
    this.objets = new Objets(this.etat, this.particules, this, scene, textures);
    this.coffres = new Coffres(this.etat, this.terrain, this.particules, this, scene, textures);
    const butin = new Butin(this.orbes, this.objets, this.terrain);
    butin.brancherCoffres(this.coffres);
    this.ennemis = new Ennemis(
      this.etat,
      this.terrain,
      { cible, tireur: this.projectilesEnnemis },
      this.particules,
      butin,
      this,
      scene,
      textures,
    );
    this.camera = new CameraJeu(rendu.camera, this.entrees, this.etat);

    // Combat du joueur
    this.mecaniques = new Talents(this.etat);
    const combat: ContexteCombat = {
      etat: this.etat,
      carte,
      ennemis: this.ennemis,
      particules: this.particules,
      retours: this,
      talents: this.mecaniques,
      declencherGesteJoueur: (angle) => this.joueur.declencherGeste(angle),
    };
    this.balayages = new Balayages(combat, textures, scene);
    this.projectilesJoueur = new ProjectilesJoueur(combat);
    this.orbites = new Orbites(combat);
    this.foudre = new Foudre(combat, scene);
    this.invocations = new Invocations(combat, scene, textures, new BarresVie(scene, rendu.camera, CONFIG.armes.golems.capacite), this.projectilesJoueur);
    this.ennemis.definirAllies(this.invocations);
    this.projectilesEnnemis.definirEnnemis(this.ennemis);
    this.fissures = new Fissures(combat, textures, scene);
    this.chutes = new Chutes(combat, textures, scene);
    this.tourbillons = new Tourbillons(combat, textures, scene);
    this.heurts = new Heurts(combat, this.projectilesEnnemis);
    this.tornades = new Tornades(combat, textures, scene);
    this.novas = new Novas(combat, textures, scene);
    this.meteores = new Meteores(combat, textures, scene);
    this.orbesArcaniques = new OrbesArcaniques(combat, textures, scene);
    this.ricochets = new Ricochets(combat);
    this.pluies = new Pluies(combat, textures, scene);
    this.pieges = new Pieges(combat, textures, scene);
    this.arsenal = new Arsenal(combat, {
      balayages: this.balayages,
      projectiles: this.projectilesJoueur,
      orbites: this.orbites,
      foudre: this.foudre,
      invocations: this.invocations,
      fissures: this.fissures,
      chutes: this.chutes,
      tourbillons: this.tourbillons,
      heurts: this.heurts,
      tornades: this.tornades,
      novas: this.novas,
      meteores: this.meteores,
      orbesArcaniques: this.orbesArcaniques,
      ricochets: this.ricochets,
      pluies: this.pluies,
      pieges: this.pieges,
    });
    for (const id of this.etat.personnage.armesDepart) this.arsenal.ameliorer(id);
    for (const id of talents.armes) this.arsenal.ameliorer(id);
    this.mecaniques.brancher(combat, this.arsenal, { scene, textures, orbes: this.orbes }, mecaniquesClasse(choix.personnage));
    this.ennemis.definirTalents(this.mecaniques);
    this.joueur.talents = this.mecaniques;

    // Interface et directeurs d'apparition
    this.hud = new Hud(textures, {
      etat: this.etat,
      arsenal: this.arsenal,
      passifs: this.passifs,
      ennemis: this.ennemis,
      tempsAvantBoss: () => this.directeurBoss.tempsAvantProchain,
      bossAVaincre: () => this.prolongation,
    });
    this.directeur = new DirecteurApparitions(this.etat, this.terrain, this.ennemis, this.camera);
    this.directeurBoss = new DirecteurBoss({
      calendrier: CALENDRIER_BOSS, liste: (a) => a.boss, annonce: (nom) => `Un boss approche : ${nom} !`, distance: 12, son: 'boss', secousse: 0.6,
    }, this.etat, this.terrain, this.ennemis, this.camera, this.hud, this);
    this.directeurMiniBoss = new DirecteurBoss({
      // Les mini-boss arrivent sans prévenir, comme les autres ennemis
      calendrier: CALENDRIER_MINIBOSS, liste: (a) => a.miniBoss, annonce: null, distance: 10, son: null, secousse: 0,
    }, this.etat, this.terrain, this.ennemis, this.camera, this.hud, this);

    // Progression et écrans
    this.ameliorations = new Ameliorations(this.etat, this.arsenal, this.passifs);
    this.ecranCartes = new EcranCartes(textures);
    this.compteurDegats = new CompteurDegats(this.etat, this.arsenal.armes, textures);
    this.panneauOptions = services.panneauOptions;
    // Tab : afficher ou masquer le compteur de dégâts (retenu dans les options)
    window.addEventListener('keydown', (e) => {
      if (e.code !== 'Tab') return;
      e.preventDefault();
      if (!e.repeat) this.panneauOptions.modifier((o) => { o.compteurDegats = !o.compteurDegats; });
    });
    // F5 : vue de dessus ↔ 3e personne ; la touche est un geste de l'utilisateur,
    // la souris peut être capturée tout de suite
    window.addEventListener('keydown', (e) => {
      if (e.code !== 'F5') return;
      e.preventDefault();
      if (e.repeat) return;
      this.panneauOptions.modifier(vueSuivante);
      if (this.camera.alaSouris && this.vueInteractive()) this.entrees.capturerSouris(this.toile);
      else this.entrees.libererSouris();
    });
    // Vue à la 3e personne : un clic dans le jeu capture la souris
    window.addEventListener('click', () => {
      if (this.camera.alaSouris && this.vueInteractive()) this.entrees.capturerSouris(this.toile);
    });
    const bilan = (conteneur: HTMLElement) => remplirBilan(conteneur, this.etat, this.arsenal.armes, textures);
    this.ecranPause = new EcranPause({
      peutPauser: () => !this.ecranCartes.ouvert && this.etat.issue === null,
      recommencer: () => lancerPartie(this.choix),
      menuPrincipal: () => retournerAuMenu(this.choix),
      remplirResume: (conteneur) => remplirPastilles(conteneur, this.etat, textures),
      remplirBilan: bilan,
    }, services.panneauOptions);
    this.ecranFin = new EcranFin({
      // À plusieurs, une nouvelle partie se refait depuis le salon
      rejouer: () => (this.reseau ? retournerAuMenu(this.choix) : lancerPartie(this.choix)),
      continuer: () => this.continuerSansFin(),
      menuPrincipal: () => retournerAuMenu(this.choix),
      remplirBilan: bilan,
    }, textures);

    this.sourcesEntites = [
      this.joueur, this.ennemis, this.projectilesEnnemis, this.projectilesJoueur, this.ricochets, this.invocations, this.orbes, this.objets, this.coffres,
    ];

    this.debug = new PanneauDebug({
      rendu,
      etat: this.etat,
      hud: this.hud,
      camera: this.camera,
      textures,
      compterEntites: () => this.compterEntites(),
      compterParticules: () => this.particules.actives,
      definirInvincibilite: (actif) => { this.joueur.invincible = actif; },
      apparaitreEnnemi: (id) => this.apparaitreAutourDuJoueur(id),
      ameliorerArme: (id) => { this.arsenal.ameliorer(id); },
      ajouterBonus: (id) => this.ameliorations.appliquer({ type: 'passif', id }),
      poserObjet: (id) => this.objets.poser(id, this.etat.joueur.x + 1.5, this.etat.joueur.z),
      infligerEtat: (etat) => this.infligerEtatAutour(etat),
      gagnerNiveau: () => gagnerXp(this.etat, this.etat.joueur.xpNiveau - this.etat.joueur.xp),
      apparaitreBoss: () => this.directeurBoss.forcer(),
      allerVersFin: () => { this.etat.tempsJeu = Math.max(this.etat.tempsJeu, PROGRESSION.dureePartie - 10); },
      changerPersonnage: (id) => lancerPartie({ ...this.choix, personnage: id }),
      changerArene: (arene) => lancerPartie({ ...this.choix, arene }),
      frapper: () => this.frapperAutourDuJoueur(),
      offrirMaitrise: () => {
        const m = this.sauvegarde.offrirNiveau(this.choix.personnage);
        this.hud.afficherMessage(`Maîtrise ${m.niveau}`);
      },
    });
    this.reseau = services.reseau ? brancherPartie(services.reseau, {
      etat: this.etat, terrain: this.terrain, scene, camera: rendu.camera, cameraJeu: this.camera, textures, entrees: this.entrees,
      joueur: this.joueur, cible, mecaniques: this.mecaniques, ennemis: this.ennemis, projectilesEnnemis: this.projectilesEnnemis,
      orbes: this.orbes, objets: this.objets, particules: this.particules, chiffres: this.chiffres, retours: this,
      coffres: this.coffres, conclure: () => this.conclure(), continuerSansFin: () => this.continuerSansFin(),
    }) : null;
    this.rendu.preparerOmbres();
    this.boucle = new Boucle(this);
  }

  /** Prépare tous les modèles sur la carte graphique (pendant l'écran de chargement). */
  prechauffer(surProgression?: (faites: number, total: number) => void): Promise<void> {
    this.camera.majVisuel(0, 0);
    return this.rendu.prechauffer(surProgression);
  }

  /**
   * Belle vue de l'arène pour l'écran de chargement (comme les images de zone
   * de WoW) : de biais et de haut, toute l'arène dans le cadre, le brouillard
   * repoussé le temps de la photo. À prendre une fois tout préparé, avant le
   * début de la partie.
   */
  photographier(): HTMLCanvasElement | null {
    const P = CONFIG.chargement.photo;
    const taille = this.tailleDecor;
    const camera = new THREE.PerspectiveCamera(P.champ, 16 / 9, 0.5, 600);
    const distance = taille * P.recul;
    const plongee = P.plongeeDeg * Math.PI / 180, azimut = P.azimutDeg * Math.PI / 180;
    camera.position.set(
      Math.sin(azimut) * Math.cos(plongee) * distance,
      Math.sin(plongee) * distance,
      Math.cos(azimut) * Math.cos(plongee) * distance,
    );
    camera.lookAt(0, 0, 0);
    const brouillard = this.rendu.scene.fog as THREE.Fog | null;
    const avant = brouillard ? [brouillard.near, brouillard.far] : null;
    if (brouillard) {
      brouillard.near = distance * 0.8;
      brouillard.far = distance * 2.4;
    }
    const photo = this.rendu.photographier(camera, P.largeur);
    if (brouillard && avant) [brouillard.near, brouillard.far] = avant;
    return photo;
  }

  demarrer(): void {
    const manquantes = this.textures.bilan.manquantes.length;
    if (manquantes > 0) {
      this.hud.afficherMessage(`${manquantes} texture(s) introuvable(s) : couleurs de secours`);
    }
    this.boucle.demarrer();
  }

  /** Options modifiables en cours de partie. */
  appliquerOptions(options: Options): void {
    this.reseau?.appliquerOptions(options);
    this.hud.afficherFps = options.afficherFps;
    this.camera.secoussesActives = options.secousses;
    if (options.vue !== this.camera.vue) this.camera.definirMode(options.vue);
    this.chiffres.actif = options.chiffresDegats;
    this.compteurDegats.definir(options.compteurDegats);
    this.vegetation.definirShaders(options.shaders !== 'aucun');
    this.rendu.definirShaders(options.shaders);
  }

  // --- Retours -----------------------------------------------------------

  son(id: IdSon, hauteur?: number): void {
    this.sons.jouer(id, hauteur);
  }

  secousse(force: number): void {
    this.camera.secouer(force);
  }

  degats(x: number, y: number, z: number, valeur: number, style: StyleChiffre): void {
    this.chiffres.ajouter(x, y, z, valeur, style);
    this.reseau?.surDegats(x, y, z, valeur, style);
  }

  // --- CallbacksBoucle ---------------------------------------------------

  simuler(dt: number): void {
    // À plusieurs, celui qui choisit ses cartes reste sur place, armes au repos (la partie continue)
    const pauseLocale = this.reseau?.pauseLocale === true;
    // À plusieurs, un joueur qui a rejoint ne simule que sa marche et l'effet des armes : le reste vient de l'hôte
    if (this.reseau?.client) {
      this.reseau.simulerClient(dt);
      if (!pauseLocale) {
        this.arsenal.maj(dt);
        this.mecaniques.maj(dt);
      }
      this.majSystemesArmes(dt);
      this.reseau.majArmes(dt);
      this.particules.maj(dt);
      this.chiffres.maj(dt);
      return;
    }
    if (this.etat.issue === null) {
      this.etat.tempsJeu += dt;
      this.verifierFin();
    }
    if (!pauseLocale) this.joueur.maj(dt);
    this.reseau?.majJoueurs(dt);
    this.terrain.flux.maj(this.etat.joueur.x, this.etat.joueur.z);
    this.directeur.maj(dt * (this.reseau?.facteurApparitions ?? 1));
    this.directeurBoss.maj();
    this.directeurMiniBoss.maj();
    this.ennemis.maj(dt);           // reconstruit la grille utilisée par les armes
    if (!pauseLocale) {
      this.arsenal.maj(dt);
      this.mecaniques.maj(dt);
    }
    this.orbes.maj(dt);
    this.objets.maj(dt);
    this.coffres.maj(dt);
    this.majSystemesArmes(dt);
    this.reseau?.majArmes(dt);
    this.projectilesEnnemis.maj(dt);
    this.particules.maj(dt);
    this.chiffres.maj(dt);
    this.traiterResurrection();
    this.reseau?.finPas(dt);
  }

  /** Projectiles, invocations et effets des armes du joueur. */
  private majSystemesArmes(dt: number): void {
    this.projectilesJoueur.maj(dt);
    this.invocations.maj(dt);
    this.foudre.maj(dt);
    this.fissures.maj(dt);
    this.chutes.maj(dt);
    this.tourbillons.maj(dt);
    this.tornades.maj(dt);
    this.novas.maj(dt);
    this.meteores.maj(dt);
    this.orbesArcaniques.maj(dt);
    this.ricochets.maj(dt);
    this.pluies.maj(dt);
    this.pieges.maj(dt);
  }

  /** Après une résurrection (totem) : les ennemis proches sont repoussés, gerbe de particules. */
  private traiterResurrection(): void {
    const j = this.etat.joueur;
    if (j.viesUtilisees === this.viesTraitees) return;
    this.viesTraitees = j.viesUtilisees;
    const t = CONFIG.joueur.totem;
    this.ennemis.repousser(j.x, j.z, t.rayonRecul, t.recul);
    this.particules.emettre(j.x, 1.2, j.z, GERBES.totem);
  }

  afficher(dtJeu: number, dtReel: number, dtBrut: number): void {
    this.textures.majAnimations(performance.now() / 1000);
    this.reseau?.avantImage(dtReel);
    // Caméra d'abord : les objets tournés vers elle utilisent son orientation de cette image
    this.majVueSouris();
    this.camera.majVisuel(dtJeu, dtReel);
    this.joueur.lacetCommandes = this.camera.lacetCommandes;
    this.ombres.commencer();
    this.joueur.majVisuel(this.ombres);
    const j = this.etat.joueur;
    this.etiquette.placer(j.x, this.joueur.hauteur, j.z, !j.mort && !this.camera.alaSouris);
    this.ennemis.majVisuel(this.ombres, this.barresMiniBoss);
    this.invocations.majVisuel(this.ombres);
    this.mecaniques.majVisuel(this.ombres);
    this.chutes.majVisuel(this.ombres);
    this.meteores.majVisuel(this.ombres);
    this.reseau?.majVisuelOmbres(this.ombres);
    this.ombres.terminer();

    this.renduProjectiles.commencer();
    this.projectilesEnnemis.majVisuel(this.renduProjectiles);
    this.projectilesJoueur.majVisuel(this.renduProjectiles, this.etat.tempsJeu);
    this.ricochets.majVisuel(this.renduProjectiles);
    this.orbites.majVisuel(this.arsenal.armes, this.renduProjectiles, this.etat.tempsJeu);
    this.reseau?.majVisuelProjectiles(this.renduProjectiles);
    this.renduProjectiles.terminer();

    this.orbes.majVisuel(this.etat.tempsJeu);
    this.objets.majVisuel(this.etat.tempsJeu);
    this.coffres.majVisuel(this.etat.tempsJeu);
    this.balayages.majVisuel(dtJeu);
    this.foudre.majVisuel();
    this.fissures.majVisuel();
    this.tourbillons.majVisuel();
    this.tornades.majVisuel();
    this.novas.majVisuel();
    this.orbesArcaniques.majVisuel();
    this.pluies.majVisuel();
    this.pieges.majVisuel();
    this.reseau?.majVisuelEffets(dtJeu);
    this.particules.majVisuel();
    this.chiffres.majVisuel();
    this.rendu.dessiner();

    this.hud.majVisuel(dtBrut, dtReel);
    this.debug.majVisuel(dtReel);
    this.compteurDegats.majVisuel(dtReel);
    this.celebrerNiveau();
    this.afficherFin(dtReel);
    this.proposerCartes();
  }

  private get toile(): HTMLElement {
    return this.rendu.renderer.domElement;
  }

  /** Vrai quand aucun écran (cartes, pause, options, fin) ne demande le curseur. */
  private vueInteractive(): boolean {
    return !this.ecranCartes.ouvert && !this.ecranPause.active && !this.panneauOptions.ouvert && this.etat.issue === null;
  }

  /** Vue à la 3e personne : rend le curseur aux écrans, rappelle de cliquer sinon. */
  private majVueSouris(): void {
    const interactive = this.vueInteractive();
    if (!interactive) this.entrees.libererSouris();
    this.aideSouris.classList.toggle('cache', !(this.camera.alaSouris && interactive && !this.entrees.sourisCapturee));
  }

  /** Partie figée pendant le choix des cartes et la pause. */
  vitesseTemps(): number {
    // À plusieurs, la pause ne fige rien : seules les cartes (de n'importe qui) arrêtent la partie ; le debug reste à l'hôte
    if (this.reseau) return this.reseau.vitesseTemps(this.ecranCartes.ouvert) * (this.reseau.client ? 1 : this.debug.vitesseTemps);
    return this.ecranCartes.ouvert || this.ecranPause.active ? 0 : this.debug.vitesseTemps;
  }

  // --- Fin de partie -----------------------------------------------------

  /** Défaite à la mort du joueur ; victoire s'il tient jusqu'au bout du chrono. */
  private verifierFin(): void {
    const etat = this.etat;
    if (equipeVaincue(etat)) {
      etat.issue = 'defaite';
    } else if (!etat.sansFin && etat.tempsJeu >= PROGRESSION.dureePartie) {
      // Pas de victoire tant qu'un boss est en vie : il faut le vaincre
      if (this.ennemis.bossActif()) {
        if (!this.prolongation) {
          this.prolongation = true;
          this.hud.alerter('Vainquez le boss pour gagner !', 4);
        }
        return;
      }
      if (!this.prolongation) etat.tempsJeu = PROGRESSION.dureePartie;
      this.prolongation = false;
      etat.issue = 'victoire';
      this.ennemis.terrasserTous();
      this.son('victoire');
      this.secousse(0.5);
    }
    if (etat.issue !== null) this.conclure();
  }

  /** Fin de partie décidée : récompenses (sauvegarde), puis écran de fin après la chute ou la victoire. */
  private conclure(): void {
    const etat = this.etat;
    this.minuterieFin = 0;
    this.recompense = this.sauvegarde.terminerPartie({
      personnage: this.choix.personnage,
      arene: etat.arene.id,
      temps: etat.tempsJeu,
      xp: etat.stats.xpTotale - this.xpComptee,
      boss: etat.stats.bossVaincus - this.bossComptes,
      sansFin: etat.sansFin,
    });
    this.xpComptee = etat.stats.xpTotale;
    this.bossComptes = etat.stats.bossVaincus;
  }

  /** Mode sans fin : la partie gagnée reprend, les ennemis se renforcent chaque minute jusqu'à la mort. */
  private continuerSansFin(): void {
    const etat = this.etat;
    if (etat.issue !== 'victoire') return;
    etat.issue = null;
    etat.sansFin = true;
    this.directeurBoss.reprendre();
    this.directeurMiniBoss.reprendre();
    this.ecranFin.masquer();
    this.hud.alerter('Mode sans fin : tenez le plus longtemps possible !', 4);
    this.reseau?.surContinuer();
  }

  /** Laisse la chute ou la victoire se jouer, puis affiche le bilan. */
  private afficherFin(dtReel: number): void {
    const issue = this.etat.issue;
    if (issue === null || this.minuterieFin < 0) return;
    this.minuterieFin += dtReel;
    if (this.minuterieFin < CONFIG.hud.delaiFin) return;
    this.minuterieFin = -1;
    this.ecranPause.definir(false);
    this.ecranFin.afficher(issue, this.etat, this.recompense);
  }

  // --- Progression -------------------------------------------------------

  /** Son et gerbe de particules à chaque niveau gagné. */
  private celebrerNiveau(): void {
    const j = this.etat.joueur;
    if (j.niveau === this.niveauAffiche) return;
    this.niveauAffiche = j.niveau;
    this.particules.emettre(j.x, 1, j.z, GERBES.niveau);
    this.son('niveau');
  }

  /** Ouvre les cartes d'un niveau en attente (un niveau à la fois), puis celles des coffres ouverts. */
  private proposerCartes(): void {
    const etat = this.etat;
    if (this.ecranCartes.ouvert || etat.issue !== null) return;
    if (etat.niveauxEnAttente === 0 && etat.choixCoffre.restants === 0) return;
    this.ecranPause.definir(false);
    const coffre = etat.niveauxEnAttente === 0;
    const titre = coffre ? etat.choixCoffre.titre : `Niveau ${etat.joueur.niveau - etat.niveauxEnAttente + 1} !`;
    this.ecranCartes.ouvrir(titre, this.ameliorations.tirer(), (carte: Carte) => {
      this.ameliorations.appliquer(carte.amelioration);
      this.reseau?.surCarte(carte.amelioration);
      if (coffre) etat.choixCoffre.restants--;
      else etat.niveauxEnAttente--;
    }, {
      restantes: () => this.etat.relancesRestantes,
      relancer: () => {
        if (this.etat.relancesRestantes <= 0) return null;
        this.etat.relancesRestantes--;
        return this.ameliorations.tirer();
      },
    });
  }

  // --- Debug -------------------------------------------------------------

  private compterEntites(): number {
    let total = 0;
    for (let i = 0; i < this.sourcesEntites.length; i++) total += this.sourcesEntites[i].actifs;
    return total;
  }

  /** Apparition sur une case libre autour du joueur. */
  private apparaitreAutourDuJoueur(id: IdEnnemi): void {
    const d = CONFIG.debug.distanceApparition;
    const j = this.etat.joueur;
    for (let essai = 0; essai < 16; essai++) {
      const angle = Math.random() * Math.PI * 2;
      const x = j.x + Math.cos(angle) * d;
      const z = j.z + Math.sin(angle) * d;
      if (this.terrain.carte.bloqueMarche(x, z)) continue;
      this.ennemis.apparaitre(id, x, z);
      return;
    }
  }

  /** Debug : inflige un état à tous les ennemis proches. */
  private infligerEtatAutour(etat: EtatDebug): void {
    const c = CONFIG.debug.etats;
    const j = this.etat.joueur;
    const n = this.ennemis.chercher(j.x, j.z, c.rayon, this.prochesDebug);
    for (let k = 0; k < n; k++) {
      const e = this.ennemis.obtenir(this.prochesDebug[k]);
      switch (etat) {
        case 'brulure': this.ennemis.bruler(e, c.brulure.cumuls, c.brulure.degatsParSeconde, c.brulure.duree); break;
        case 'poison': this.ennemis.empoisonner(e, c.poison.cumuls, c.poison.degatsParSeconde, c.poison.duree); break;
        case 'ralenti': this.ennemis.ralentir(e, c.ralenti.part, c.ralenti.duree); break;
        case 'enracine': this.ennemis.enraciner(e, c.enracine.duree); break;
        case 'etourdi': this.ennemis.etourdir(e, c.etourdi.duree); break;
        case 'marque': this.ennemis.marquer(e, c.marque.bonus, c.marque.duree); break;
      }
    }
  }

  private frapperAutourDuJoueur(): void {
    const f = CONFIG.debug.frappe;
    const j = this.etat.joueur;
    this.ennemis.frapperZone(j.x, j.z, f.rayon, f.degats, f.recul);
  }
}
