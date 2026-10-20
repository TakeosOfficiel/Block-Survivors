/** HUD : cœurs, armes possédées, compteur de FPS et messages centraux. */
import type { BibliothequeTextures } from '../assets/textures';
import type { Arsenal } from '../combat/arsenal';
import { CONFIG } from '../config';
import type { EtatJoueur } from '../etat';
import { BarreArmes } from './barreArmes';
import { BarreCoeurs } from './coeurs';
import { element } from './dom';

export class Hud {
  private readonly elementFps = element('hud-fps');
  private readonly elementMessage = element('hud-message');
  private readonly coeurs: BarreCoeurs;
  private readonly armes: BarreArmes;

  private images = 0;
  private duree = 0;
  private minuterieMessage = 0;

  /** Dernière mesure (lue aussi par le debug). */
  fps = 0;
  msParImage = 0;

  constructor(textures: BibliothequeTextures, pvMax: number) {
    this.coeurs = new BarreCoeurs(textures, pvMax);
    this.armes = new BarreArmes(textures);
  }

  /** Affiche un message central pendant `duree` secondes (Infinity = jusqu'au prochain message). */
  afficherMessage(texte: string, duree: number = CONFIG.hud.dureeMessage): void {
    this.elementMessage.textContent = texte;
    this.elementMessage.classList.add('visible');
    this.minuterieMessage = duree;
  }

  /** dtBrut (non borné) mesure les FPS réels ; dtReel (borné) fait avancer les minuteries. */
  majVisuel(dtBrut: number, dtReel: number, joueur: EtatJoueur, arsenal: Arsenal): void {
    this.mesurerFps(dtBrut);
    this.coeurs.afficher(joueur.pv);
    this.armes.afficher(arsenal);

    if (this.minuterieMessage > 0) {
      this.minuterieMessage -= dtReel;
      if (this.minuterieMessage <= 0) this.elementMessage.classList.remove('visible');
    }
  }

  private mesurerFps(dtBrut: number): void {
    this.images++;
    this.duree += dtBrut;
    if (this.duree < CONFIG.hud.periodeFps) return;
    this.fps = Math.round(this.images / this.duree);
    this.msParImage = (this.duree * 1000) / this.images;
    this.images = 0;
    this.duree = 0;
    this.elementFps.textContent = `FPS : ${this.fps}`;
  }
}
