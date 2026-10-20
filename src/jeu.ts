/**
 * Une partie : assemble les systèmes et fixe l'ordre de mise à jour.
 *
 * Convention : maj(dt) = simulation en temps de jeu (ne touche qu'à l'état),
 * majVisuel(...) = recopie de l'état dans la scène et le DOM.
 */
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
import { JOUEUR_BASE } from './donnees/joueur';
import type { DefinitionArene } from './donnees/types';
import { Ombres } from './effets/ombres';
import { Particules } from './effets/particules';
import { RenduProjectiles } from './effets/renduProjectiles';
import { DirecteurApparitions } from './entites/directeur';
import { Ennemis } from './entites/ennemis';
import { Joueur } from './entites/joueur';
import { ProjectilesEnnemis } from './entites/projectilesEnnemis';
import type { SourceEntites } from './entites/types';
import { creerEtatPartie, type EtatPartie } from './etat';
import { construireArene } from './monde/arene';
import { Boucle, type CallbacksBoucle } from './moteur/boucle';
import { CameraJeu } from './moteur/camera';
import type { Rendu } from './moteur/rendu';
import { PanneauDebug } from './ui/debug';
import { Hud } from './ui/hud';

export class Jeu implements CallbacksBoucle {
  private readonly rendu: Rendu;
  private readonly textures: BibliothequeTextures;
  private readonly etat: EtatPartie;
  private readonly entrees = new Entrees();
  private readonly hud: Hud;
  private readonly camera: CameraJeu;
  private readonly boucle: Boucle;

  // Effets
  private readonly ombres: Ombres;
  private readonly particules: Particules;
  private readonly renduProjectiles: RenduProjectiles;

  // Entités
  private readonly joueur: Joueur;
  private readonly ennemis: Ennemis;
  private readonly projectilesEnnemis: ProjectilesEnnemis;
  private readonly directeur: DirecteurApparitions;

  // Combat du joueur
  private readonly arsenal: Arsenal;
  private readonly balayages: Balayages;
  private readonly projectilesJoueur: ProjectilesJoueur;
  private readonly orbites: Orbites;
  private readonly foudre: Foudre;
  private readonly invocations: Invocations;

  private readonly debug: PanneauDebug;
  private readonly sourcesEntites: readonly SourceEntites[];
  private mortAnnoncee = false;

  constructor(rendu: Rendu, textures: BibliothequeTextures, arene: DefinitionArene) {
    this.rendu = rendu;
    this.textures = textures;
    this.etat = creerEtatPartie(arene);
    const scene = rendu.scene;

    construireArene(arene, textures, scene);
    this.hud = new Hud(textures, this.etat.joueur.pvMax);
    this.ombres = new Ombres(textures.obtenir(CONFIG.ombre.texture), scene);
    this.particules = new Particules(scene);
    this.renduProjectiles = new RenduProjectiles(textures, scene);

    // Entités
    this.joueur = new Joueur(this.etat, this.entrees, scene, textures);
    this.projectilesEnnemis = new ProjectilesEnnemis(this.etat, this.joueur);
    this.ennemis = new Ennemis(
      this.etat,
      { cible: this.joueur, tireur: this.projectilesEnnemis },
      this.particules,
      scene,
      textures,
    );
    this.camera = new CameraJeu(rendu.camera, this.entrees, this.etat);
    this.directeur = new DirecteurApparitions(this.etat, this.ennemis, this.camera);

    // Combat du joueur
    const combat: ContexteCombat = {
      etat: this.etat,
      ennemis: this.ennemis,
      particules: this.particules,
      declencherGesteJoueur: () => this.joueur.declencherGeste(),
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
    for (const id of JOUEUR_BASE.armesDepart) this.arsenal.ameliorer(id);

    this.sourcesEntites = [this.joueur, this.ennemis, this.projectilesEnnemis, this.projectilesJoueur, this.invocations];

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
      frapper: () => this.frapperAutourDuJoueur(),
    });
    this.boucle = new Boucle(this);
  }

  demarrer(): void {
    const manquantes = this.textures.bilan.manquantes.length;
    if (manquantes > 0) {
      this.hud.afficherMessage(`${manquantes} texture(s) introuvable(s) : couleurs de secours`);
    }
    this.boucle.demarrer();
  }

  // --- CallbacksBoucle ---------------------------------------------------

  simuler(dt: number): void {
    if (!this.etat.joueur.mort) this.etat.tempsJeu += dt;
    this.joueur.maj(dt);
    this.directeur.maj(dt);
    this.ennemis.maj(dt);           // reconstruit la grille utilisée par les armes
    this.arsenal.maj(dt);
    this.projectilesJoueur.maj(dt);
    this.invocations.maj(dt);
    this.foudre.maj(dt);
    this.projectilesEnnemis.maj(dt);
    this.particules.maj(dt);
  }

  afficher(dtJeu: number, dtReel: number, dtBrut: number): void {
    this.ombres.commencer();
    this.joueur.majVisuel(this.ombres);
    this.ennemis.majVisuel(this.ombres);
    this.invocations.majVisuel(this.ombres);
    this.ombres.terminer();

    this.renduProjectiles.commencer();
    this.projectilesEnnemis.majVisuel(this.renduProjectiles);
    this.projectilesJoueur.majVisuel(this.renduProjectiles, this.etat.tempsJeu);
    this.orbites.majVisuel(this.arsenal.armes, this.renduProjectiles, this.etat.tempsJeu);
    this.renduProjectiles.terminer();

    this.balayages.majVisuel(dtJeu);
    this.foudre.majVisuel();
    this.particules.majVisuel();
    this.camera.majVisuel(dtJeu, dtReel);
    this.rendu.dessiner();

    this.hud.majVisuel(dtBrut, dtReel, this.etat.joueur, this.arsenal);
    this.debug.majVisuel(dtReel);
    if (this.etat.joueur.mort && !this.mortAnnoncee) {
      this.mortAnnoncee = true;
      this.hud.afficherMessage('Vous êtes mort !', Infinity);
    }
  }

  vitesseTemps(): number {
    return this.debug.vitesseTemps;
  }

  // --- Debug -------------------------------------------------------------

  private compterEntites(): number {
    let total = 0;
    for (let i = 0; i < this.sourcesEntites.length; i++) total += this.sourcesEntites[i].actifs;
    return total;
  }

  private apparaitreAutourDuJoueur(id: IdEnnemi): void {
    const angle = Math.random() * Math.PI * 2;
    const d = CONFIG.debug.distanceApparition;
    const j = this.etat.joueur;
    this.ennemis.apparaitre(id, j.x + Math.cos(angle) * d, j.z + Math.sin(angle) * d);
  }

  private frapperAutourDuJoueur(): void {
    const f = CONFIG.debug.frappe;
    const j = this.etat.joueur;
    this.ennemis.frapperZone(j.x, j.z, f.rayon, f.degats, f.recul);
  }
}
