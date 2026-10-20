/** Barre d'expérience et niveau, comme en jeu (sprites vanilla 182x5). */
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import type { EtatJoueur } from '../etat';
import { element } from './dom';

export class BarreXp {
  private readonly remplissage = element('hud-xp-remplissage');
  private readonly niveau = element('hud-niveau');
  private ratioAffiche = -1;
  private niveauAffiche = -1;

  constructor(textures: BibliothequeTextures) {
    const sprites = CONFIG.hud.barreXp;
    const fond = textures.url(sprites.fond);
    const progression = textures.url(sprites.progression);
    // Sans sprites, les couleurs CSS de secours restent visibles
    if (fond) element('hud-xp').style.backgroundImage = `url("${fond}")`;
    if (progression) this.remplissage.style.backgroundImage = `url("${progression}")`;
  }

  afficher(joueur: EtatJoueur): void {
    const ratio = Math.min(1, joueur.xp / joueur.xpNiveau);
    // Arrondi au pixel de la barre (182 px) : pas d'écriture DOM inutile
    const pixels = Math.floor(ratio * 182);
    if (pixels !== this.ratioAffiche) {
      this.ratioAffiche = pixels;
      this.remplissage.style.width = `${(pixels / 182) * 100}%`;
    }
    if (joueur.niveau !== this.niveauAffiche) {
      this.niveauAffiche = joueur.niveau;
      this.niveau.textContent = String(joueur.niveau);
    }
  }
}
