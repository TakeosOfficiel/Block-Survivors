/** Barre de vie du boss en haut de l'écran, comme en jeu (sprites vanilla 182x5). */
import type { BibliothequeTextures } from '../assets/textures';
import type { Ennemi } from '../entites/ennemi';
import { element } from './dom';

const CHEMIN_SPRITES = 'gui/sprites/boss_bar';

export class BarreBoss {
  private readonly racine = element('hud-boss');
  private readonly nom = element('hud-boss-nom');
  private readonly barre = element('hud-boss-barre');
  private readonly remplissage = element('hud-boss-remplissage');
  private readonly textures: BibliothequeTextures;
  private bossAffiche: Ennemi | null = null;
  private pixelsAffiches = -1;

  constructor(textures: BibliothequeTextures) {
    this.textures = textures;
  }

  /** Sprites requis pour une couleur de barre. */
  static sprites(couleur: string): string[] {
    return [`${CHEMIN_SPRITES}/${couleur}_background`, `${CHEMIN_SPRITES}/${couleur}_progress`];
  }

  afficher(boss: Ennemi | null): void {
    if (boss !== this.bossAffiche) {
      this.bossAffiche = boss;
      this.pixelsAffiches = -1;
      this.racine.classList.toggle('cache', boss === null);
      if (boss?.definition.boss) {
        const [fond, progression] = BarreBoss.sprites(boss.definition.boss.couleurBarre);
        this.nom.textContent = boss.definition.nom;
        this.barre.style.backgroundImage = urlCss(this.textures.url(fond));
        this.remplissage.style.backgroundImage = urlCss(this.textures.url(progression));
      }
    }
    if (!boss) return;
    const pixels = Math.ceil(Math.max(0, boss.pv / boss.pvMax) * 182);
    if (pixels !== this.pixelsAffiches) {
      this.pixelsAffiches = pixels;
      this.remplissage.style.width = `${(pixels / 182) * 100}%`;
    }
  }
}

function urlCss(url: string | null): string {
  return url ? `url("${url}")` : 'none';
}
