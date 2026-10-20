/** Icônes des armes possédées, avec leur rang (en bas à gauche de l'écran). */
import type { BibliothequeTextures } from '../assets/textures';
import type { Arsenal } from '../combat/arsenal';
import { rangMax } from '../combat/armeEquipee';
import { element } from './dom';

export class BarreArmes {
  private readonly conteneur = element('hud-armes');
  private readonly textures: BibliothequeTextures;
  private version = -1;

  constructor(textures: BibliothequeTextures) {
    this.textures = textures;
  }

  /** Reconstruit les icônes seulement quand l'arsenal a changé. */
  afficher(arsenal: Arsenal): void {
    if (arsenal.version === this.version) return;
    this.version = arsenal.version;
    this.conteneur.replaceChildren(...arsenal.armes.map((arme) => {
      const case_ = document.createElement('div');
      case_.className = 'arme';
      case_.title = `${arme.definition.nom} — ${arme.definition.description}`;

      const url = this.textures.url(arme.definition.icone);
      if (url) {
        const icone = document.createElement('img');
        icone.src = url;
        icone.alt = arme.definition.nom;
        case_.append(icone);
      } else {
        case_.textContent = arme.definition.nom.charAt(0);
      }

      const rang = document.createElement('span');
      rang.className = 'rang';
      const max = rangMax(arme.definition);
      rang.textContent = arme.rang >= max ? 'MAX' : String(arme.rang);
      case_.append(rang);
      return case_;
    }));
  }
}
