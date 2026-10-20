/**
 * Une partie : assemble les systèmes et fixe l'ordre de mise à jour.
 *
 * Convention : maj(dt) = simulation en temps de jeu (ne touche qu'à `etat`),
 * majVisuel(...) = recopie de l'état dans la scène et le DOM.
 */
import type { BibliothequeTextures } from './assets/textures';
import { Entrees } from './controles/entrees';
import type { DefinitionArene } from './donnees/types';
import { Joueur } from './entites/joueur';
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
  private readonly hud = new Hud();
  private readonly joueur: Joueur;
  private readonly camera: CameraJeu;
  private readonly debug: PanneauDebug;
  private readonly sourcesEntites: readonly SourceEntites[];
  private readonly boucle: Boucle;

  constructor(rendu: Rendu, textures: BibliothequeTextures, arene: DefinitionArene) {
    this.rendu = rendu;
    this.textures = textures;
    this.etat = creerEtatPartie(arene);

    construireArene(arene, textures, rendu.scene);
    this.joueur = new Joueur(this.etat, this.entrees, rendu.scene, textures);
    this.camera = new CameraJeu(rendu.camera, this.entrees, this.etat);
    this.sourcesEntites = [this.joueur];

    this.debug = new PanneauDebug({
      rendu,
      etat: this.etat,
      hud: this.hud,
      camera: this.camera,
      textures,
      compterEntites: () => this.compterEntites(),
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
    this.etat.tempsJeu += dt;
    this.joueur.maj(dt);
  }

  afficher(dtJeu: number, dtReel: number, dtBrut: number): void {
    this.joueur.majVisuel(dtJeu);
    this.camera.majVisuel(dtJeu, dtReel);
    this.rendu.dessiner();
    this.hud.majVisuel(dtBrut, dtReel);
    this.debug.majVisuel(dtReel);
  }

  vitesseTemps(): number {
    return this.debug.vitesseTemps;
  }

  // -----------------------------------------------------------------------

  private compterEntites(): number {
    let total = 0;
    for (let i = 0; i < this.sourcesEntites.length; i++) total += this.sourcesEntites[i].actifs;
    return total;
  }
}
