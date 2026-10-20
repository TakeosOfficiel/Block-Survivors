/** Icônes des armes et des bonus passifs possédés, avec leur rang (en bas à gauche). */
import type { BibliothequeTextures } from '../assets/textures';
import { rangMax } from '../combat/armeEquipee';
import type { Arsenal } from '../combat/arsenal';
import { PASSIFS } from '../donnees/passifs';
import type { CheminTexture } from '../donnees/types';
import type { Passifs } from '../progression/passifs';
import { element } from './dom';

export class BarreCapacites {
  private readonly armes = element('hud-armes');
  private readonly passifs = element('hud-passifs');
  private readonly textures: BibliothequeTextures;
  private versionArmes = -1;
  private versionPassifs = -1;

  constructor(textures: BibliothequeTextures) {
    this.textures = textures;
  }

  /** Reconstruit les icônes seulement quand l'arsenal ou les passifs ont changé. */
  afficher(arsenal: Arsenal, passifs: Passifs): void {
    if (arsenal.version !== this.versionArmes) {
      this.versionArmes = arsenal.version;
      this.armes.replaceChildren(...arsenal.armes.map((a) =>
        this.creerCase(a.definition.nom, a.definition.description, a.definition.icone, a.rang, rangMax(a.definition))));
    }
    if (passifs.version !== this.versionPassifs) {
      this.versionPassifs = passifs.version;
      this.passifs.replaceChildren(...passifs.possedes.map((p) => {
        const def = PASSIFS[p.id];
        return this.creerCase(def.nom, def.description, def.icone, p.rang, def.niveaux.length);
      }));
    }
  }

  private creerCase(nom: string, description: string, icone: CheminTexture, rang: number, max: number): HTMLElement {
    const case_ = document.createElement('div');
    case_.className = 'capacite';
    case_.title = `${nom} — ${description}`;

    const url = this.textures.url(icone);
    if (url) {
      const image = document.createElement('img');
      image.src = url;
      image.alt = nom;
      case_.append(image);
    } else {
      case_.textContent = nom.charAt(0);
    }

    const etiquette = document.createElement('span');
    etiquette.className = 'rang';
    etiquette.textContent = rang >= max ? 'MAX' : String(rang);
    case_.append(etiquette);
    return case_;
  }
}
