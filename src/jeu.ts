/**
 * Une partie : assemble les systèmes et fixe l'ordre de mise à jour.
 *
 * Convention : maj(dt) = simulation en temps de jeu (ne touche qu'à l'état),
 * majVisuel(...) = recopie de l'état dans la scène et le DOM.
 */
import type { LecteurSons } from './audio/sons';
import type { BibliothequeTextures } from './assets/textures';
import { Arsenal } from './combat/arsenal';
import { Balayages } from './combat/balayages';
import type { ContexteCombat } from './combat/contexte';
import { Foudre } from './combat/foudre';
import { Invocations } from './combat/invocations';
import { Orbites } from './combat/orbites';
import { ProjectilesJoueur } from './combat/projectilesJoueur';
import { CONFIG } from './config';
import { Entrees } from './controles/entrees';
import type { IdEnnemi } from './donnees/ennemis';
import { IDS_OBJETS, OBJETS } from './donnees/objets';
import { GERBES } from './donnees/particules';
import { RECOMPENSES } from './donnees/talents';
import { PROGRESSION } from './donnees/progression';
import type { IdSon } from './donnees/sons';
import { Ombres } from './effets/ombres';
import { ChiffresDegats } from './effets/chiffresDegats';
import { Etiquette } from './effets/etiquette';
import { Particules } from './effets/particules';
import { RenduProjectiles } from './effets/renduProjectiles';
import type { Retours } from './effets/retours';
import { Butin } from './entites/butin';
import { DirecteurApparitions } from './entites/directeur';
import { DirecteurBoss } from './entites/directeurBoss';
import { Ennemis } from './entites/ennemis';
import { Joueur } from './entites/joueur';
import { Objets } from './entites/objets';
import { Orbes } from './entites/orbes';
import { ProjectilesEnnemis } from './entites/projectilesEnnemis';
import type { SourceEntites } from './entites/types';
import { creerEtatPartie, type EtatPartie } from './etat';
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
import { formaterTemps } from './utils/format';
import { remplirBilan } from './ui/bilan';
import { CompteurDegats } from './ui/compteurDegats';
import { PanneauDebug } from './ui/debug';
import { EcranCartes } from './ui/ecranCartes';
import { EcranFin } from './ui/ecranFin';
import { EcranPause } from './ui/ecranPause';
import { Hud } from './ui/hud';
import type { PanneauOptions } from './ui/panneauOptions';

/** Services créés avant la partie (dès le menu) et partagés avec elle. */
export interface ServicesJeu {
  readonly sons: LecteurSons;
  readonly options: Options;
  readonly panneauOptions: PanneauOptions;
  readonly sauvegarde: Sauvegarde;
  readonly profil: Profil;
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
  private readonly vegetation: Vegetation;
  private readonly entrees = new Entrees();
  private readonly hud: Hud;
  private readonly camera: CameraJeu;
  private readonly boucle: Boucle;

  // Effets
  private readonly ombres: Ombres;
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
  private readonly directeur: DirecteurApparitions;
  private readonly directeurBoss: DirecteurBoss;

  // Combat du joueur
  private readonly arsenal: Arsenal;
  private readonly balayages: Balayages;
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
  private readonly compteurDegats: CompteurDegats;
  private readonly panneauOptions: PanneauOptions;
  private readonly sourcesEntites: readonly SourceEntites[];
  private niveauAffiche = 1;
  private viesTraitees = 0;
  /** s réelles écoulées depuis la fin de la partie (−1 tant qu'elle continue). */
  private minuterieFin = -1;
  /** 15:00 passé mais un boss vit encore : la victoire attend sa mort. */
  private prolongation = false;

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

    // Monde : blocs affichés, carte de collision, navigation des ennemis
    construireArene(arene, monde, textures, scene);
    rendu.appliquerAmbiance(arene);
    this.vegetation = new Vegetation(scene);
    this.terrain = { carte: monde.carte, flux: new ChampDeFlux(monde.carte) };
    const carte = monde.carte;
    this.ombres = new Ombres(textures.obtenir(CONFIG.ombre.texture), scene);
    this.particules = new Particules(scene, rendu.camera, textures, IDS_OBJETS.map((id) => OBJETS[id].icone));
    this.renduProjectiles = new RenduProjectiles(textures, scene);
    this.chiffres = new ChiffresDegats(scene, rendu.camera);

    // Entités
    const apparence = apparenceJoueur(this.etat.personnage, services.profil);
    this.joueur = new Joueur(this.etat, carte, this.entrees, this, scene, textures, apparence);
    this.etiquette = new Etiquette(scene, services.profil.pseudo);
    this.projectilesEnnemis = new ProjectilesEnnemis(this.etat, carte, this.joueur);
    this.orbes = new Orbes(this.etat, this, scene, textures);
    this.objets = new Objets(this.etat, this.particules, this, scene, textures);
    this.ennemis = new Ennemis(
      this.etat,
      this.terrain,
      { cible: this.joueur, tireur: this.projectilesEnnemis },
      this.particules,
      new Butin(this.orbes, this.objets, this.terrain),
      this,
      scene,
      textures,
    );
    this.camera = new CameraJeu(rendu.camera, this.entrees, this.etat);

    // Combat du joueur
    const combat: ContexteCombat = {
      etat: this.etat,
      carte,
      ennemis: this.ennemis,
      particules: this.particules,
      retours: this,
      declencherGesteJoueur: (angle) => this.joueur.declencherGeste(angle),
    };
    this.balayages = new Balayages(combat, textures, scene);
    this.projectilesJoueur = new ProjectilesJoueur(combat);
    this.orbites = new Orbites(combat);
    this.foudre = new Foudre(combat, scene);
    this.invocations = new Invocations(combat, scene, textures);
    this.arsenal = new Arsenal(combat, {
      balayages: this.balayages,
      projectiles: this.projectilesJoueur,
      orbites: this.orbites,
      foudre: this.foudre,
      invocations: this.invocations,
    });
    for (const id of this.etat.personnage.armesDepart) this.arsenal.ameliorer(id);
    for (const id of talents.armes) this.arsenal.ameliorer(id);

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
    this.directeurBoss = new DirecteurBoss(this.etat, this.terrain, this.ennemis, this.camera, this.hud, this);

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
    const bilan = (conteneur: HTMLElement) => remplirBilan(conteneur, this.etat, this.arsenal.armes, textures);
    this.ecranPause = new EcranPause({
      peutPauser: () => !this.ecranCartes.ouvert && this.etat.issue === null,
      recommencer: () => lancerPartie(this.choix),
      menuPrincipal: () => retournerAuMenu(this.choix),
      remplirBilan: bilan,
    }, services.panneauOptions);
    this.ecranFin = new EcranFin({
      rejouer: () => lancerPartie(this.choix),
      menuPrincipal: () => retournerAuMenu(this.choix),
      remplirBilan: bilan,
    }, textures.url(RECOMPENSES.icone));

    this.sourcesEntites = [
      this.joueur, this.ennemis, this.projectilesEnnemis, this.projectilesJoueur, this.invocations, this.orbes, this.objets,
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
      gagnerNiveau: () => gagnerXp(this.etat, this.etat.joueur.xpNiveau - this.etat.joueur.xp),
      apparaitreBoss: () => this.directeurBoss.forcer(),
      allerVersFin: () => { this.etat.tempsJeu = Math.max(this.etat.tempsJeu, PROGRESSION.dureePartie - 10); },
      changerPersonnage: (id) => lancerPartie({ ...this.choix, personnage: id }),
      changerArene: (arene) => lancerPartie({ ...this.choix, arene }),
      frapper: () => this.frapperAutourDuJoueur(),
      offrirEmeraudes: () => {
        this.sauvegarde.offrir(100);
        this.hud.afficherMessage(`${this.sauvegarde.emeraudes} émeraudes`);
      },
    });
    this.rendu.preparerOmbres();
    this.boucle = new Boucle(this);
  }

  /** Prépare tous les modèles sur la carte graphique (pendant l'écran de chargement). */
  prechauffer(surProgression?: (faites: number, total: number) => void): Promise<void> {
    this.camera.majVisuel(0, 0);
    return this.rendu.prechauffer(surProgression);
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
    this.hud.afficherFps = options.afficherFps;
    this.camera.secoussesActives = options.secousses;
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

  degats(x: number, y: number, z: number, valeur: number, critique: boolean): void {
    this.chiffres.ajouter(x, y, z, valeur, critique);
  }

  // --- CallbacksBoucle ---------------------------------------------------

  simuler(dt: number): void {
    if (this.etat.issue === null) {
      this.etat.tempsJeu += dt;
      this.verifierFin();
    }
    this.joueur.maj(dt);
    this.terrain.flux.maj(this.etat.joueur.x, this.etat.joueur.z);
    this.directeur.maj(dt);
    this.directeurBoss.maj();
    this.ennemis.maj(dt);           // reconstruit la grille utilisée par les armes
    this.arsenal.maj(dt);
    this.orbes.maj(dt);
    this.objets.maj();
    this.projectilesJoueur.maj(dt);
    this.invocations.maj(dt);
    this.foudre.maj(dt);
    this.projectilesEnnemis.maj(dt);
    this.particules.maj(dt);
    this.chiffres.maj(dt);
    this.traiterResurrection();
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
    this.ombres.commencer();
    this.joueur.majVisuel(this.ombres);
    const j = this.etat.joueur;
    this.etiquette.placer(j.x, this.joueur.hauteur, j.z, !j.mort);
    this.ennemis.majVisuel(this.ombres);
    this.invocations.majVisuel(this.ombres);
    this.ombres.terminer();

    this.renduProjectiles.commencer();
    this.projectilesEnnemis.majVisuel(this.renduProjectiles);
    this.projectilesJoueur.majVisuel(this.renduProjectiles, this.etat.tempsJeu);
    this.orbites.majVisuel(this.arsenal.armes, this.renduProjectiles, this.etat.tempsJeu);
    this.renduProjectiles.terminer();

    this.orbes.majVisuel(this.etat.tempsJeu);
    this.objets.majVisuel(this.etat.tempsJeu);
    this.balayages.majVisuel(dtJeu);
    this.foudre.majVisuel();
    this.particules.majVisuel();
    this.chiffres.majVisuel();
    this.camera.majVisuel(dtJeu, dtReel);
    this.rendu.dessiner();

    this.hud.majVisuel(dtBrut, dtReel);
    this.debug.majVisuel(dtReel);
    this.compteurDegats.majVisuel(dtReel);
    this.celebrerNiveau();
    this.afficherFin(dtReel);
    this.proposerCartes();
  }

  /** Partie figée pendant le choix des cartes et la pause. */
  vitesseTemps(): number {
    return this.ecranCartes.ouvert || this.ecranPause.active ? 0 : this.debug.vitesseTemps;
  }

  // --- Fin de partie -----------------------------------------------------

  /** Défaite à la mort du joueur ; victoire s'il tient jusqu'au bout du chrono. */
  private verifierFin(): void {
    const etat = this.etat;
    if (etat.joueur.mort) {
      etat.issue = 'defaite';
    } else if (etat.tempsJeu >= PROGRESSION.dureePartie) {
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
    if (etat.issue !== null) {
      this.minuterieFin = 0;
      this.recompense = this.sauvegarde.terminerPartie({
        arene: etat.arene.id,
        temps: etat.tempsJeu,
        tues: etat.ennemisTues,
        boss: etat.stats.bossVaincus,
        victoire: etat.issue === 'victoire',
      });
    }
  }

  /** Laisse la chute ou la victoire se jouer, puis affiche le bilan. */
  private afficherFin(dtReel: number): void {
    const issue = this.etat.issue;
    if (issue === null || this.minuterieFin < 0) return;
    this.minuterieFin += dtReel;
    if (this.minuterieFin < CONFIG.hud.delaiFin) return;
    this.minuterieFin = -1;
    this.ecranPause.definir(false);
    this.ecranFin.afficher(issue, this.etat.stats.xpTotale, formaterTemps(this.etat.tempsJeu), this.recompense);
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

  /** Ouvre les cartes d'un niveau en attente (un niveau à la fois). */
  private proposerCartes(): void {
    if (this.etat.niveauxEnAttente === 0 || this.ecranCartes.ouvert || this.etat.issue !== null) return;
    this.ecranPause.definir(false);
    const niveau = this.etat.joueur.niveau - this.etat.niveauxEnAttente + 1;
    this.ecranCartes.ouvrir(niveau, this.ameliorations.tirer(), (carte: Carte) => {
      this.ameliorations.appliquer(carte.amelioration);
      this.etat.niveauxEnAttente--;
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

  private frapperAutourDuJoueur(): void {
    const f = CONFIG.debug.frappe;
    const j = this.etat.joueur;
    this.ennemis.frapperZone(j.x, j.z, f.rayon, f.degats, f.recul);
  }
}
