/**
 * Entrées : état brut du clavier et de la molette.
 * Les autres systèmes lisent ces valeurs ; aucune logique de jeu ici.
 */
import { CONFIG, type ActionDeplacement } from '../config';

export class Entrees {
  private readonly touches = new Map<string, boolean>();  // event.code → enfoncée
  private readonly codesControles = new Set<string>(Object.values(CONFIG.controles).flat());
  private molette = 0;  // défilement cumulé (pixels) pas encore consommé

  constructor() {
    window.addEventListener('keydown', this.surToucheEnfoncee);
    window.addEventListener('keyup', this.surToucheRelachee);
    // Fenêtre qui perd le focus : on relâche tout pour éviter un joueur qui court seul
    window.addEventListener('blur', this.toutRelacher);
    window.addEventListener('wheel', this.surMolette, { passive: false });
  }

  /** Vrai si au moins une des touches liées à l'action est enfoncée. */
  action(nom: ActionDeplacement): boolean {
    const codes = CONFIG.controles[nom];
    for (let i = 0; i < codes.length; i++) {
      if (this.touches.get(codes[i])) return true;
    }
    return false;
  }

  /** Défilement accumulé depuis le dernier appel, puis remise à zéro. */
  consommerMolette(): number {
    const valeur = this.molette;
    this.molette = 0;
    return valeur;
  }

  private readonly surToucheEnfoncee = (e: KeyboardEvent): void => {
    if (!this.codesControles.has(e.code)) return;
    this.touches.set(e.code, true);
    e.preventDefault();
  };

  private readonly surToucheRelachee = (e: KeyboardEvent): void => {
    this.touches.set(e.code, false);
  };

  private readonly toutRelacher = (): void => {
    this.touches.clear();
  };

  /** Ctrl + molette reste le zoom du navigateur. */
  private readonly surMolette = (e: WheelEvent): void => {
    if (e.ctrlKey) return;
    e.preventDefault();
    this.molette += e.deltaMode === WheelEvent.DOM_DELTA_LINE ? e.deltaY * 16
                  : e.deltaMode === WheelEvent.DOM_DELTA_PAGE ? e.deltaY * 400
                  : e.deltaY;
  };
}
