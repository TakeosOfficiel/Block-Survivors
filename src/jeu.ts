/**
 * Une partie : assemble les systèmes et fixe l'ordre de mise à jour.
 *
 * Convention : maj(dt) = simulation en temps de jeu (ne touche qu'à l'état),
 * majVisuel(...) = recopie de l'état dans la scène et le DOM.
 */
import type { BibliothequeTextures } from './assets/textures';
import { CONFIG } from './config';
import { Entrees } from './controles/entrees';
import type { IdEnnemi } from './donnees/ennemis';
import type { DefinitionArene } from './donnees/types';
import { Ombres } from './effets/ombres';
import { Particules } from './effets/particules';
import { DirecteurApparitions } from './entites/directeur';
import { Ennemis } from './entites/ennemis';
import { Joueur } from './entites/joueur';
import { Projectiles } from './entites/projectiles';
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
  private readonly ombres: Ombres;
  private readonly particules: Particules;
  private readonly joueur: Joueur;
  private readonly projectiles: Projectiles;
  private readonly ennemis: Ennemis;
  private readonly camera: CameraJeu;
  private readonly directeur: DirecteurApparitions;
  private readonly debug: PanneauDebug;
  private readonly sourcesEntites: readonly SourceEntites[];
  private readonly boucle: Boucle;
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
    this.joueur = new Joueur(this.etat, this.entrees, scene, textures);
    this.projectiles = new Projectiles(this.etat, this.joueur, scene, textures);
    this.ennemis = new Ennemis(
      this.etat,
      { cible: this.joueur, tireur: this.projectiles },
      this.particules,
      scene,
      textures,
    );
    this.camera = new CameraJeu(rendu.camera, this.entrees, this.etat);
    this.directeur = new DirecteurApparitions(this.etat, this.ennemis, this.camera);
    this.sourcesEntites = [this.joueur, this.ennemis, this.projectiles];

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
    this.ennemis.maj(dt);
    this.projectiles.maj(dt);
    this.particules.maj(dt);
  }

  afficher(dtJeu: number, dtReel: number, dtBrut: number): void {
    this.ombres.commencer();
    this.joueur.majVisuel(this.ombres);
    this.ennemis.majVisuel(this.ombres);
    this.ombres.terminer();
    this.projectiles.majVisuel();
    this.particules.majVisuel();
    this.camera.majVisuel(dtJeu, dtReel);
    this.rendu.dessiner();

    this.hud.majVisuel(dtBrut, dtReel, this.etat.joueur);
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
