/** HUD : compteur de FPS et messages centraux. */
import { CONFIG } from '../config';
import { element } from './dom';

export class Hud {
  private readonly elementFps = element('hud-fps');
  private readonly elementMessage = element('hud-message');

  private images = 0;
  private duree = 0;
  private minuterieMessage = 0;

  /** Dernière mesure (lue aussi par le debug). */
  fps = 0;
  msParImage = 0;

  afficherMessage(texte: string): void {
    this.elementMessage.textContent = texte;
    this.elementMessage.classList.add('visible');
    this.minuterieMessage = CONFIG.hud.dureeMessage;
  }

  /** dtBrut (non borné) mesure les FPS réels ; dtReel (borné) fait avancer les minuteries. */
  majVisuel(dtBrut: number, dtReel: number): void {
    this.mesurerFps(dtBrut);

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
