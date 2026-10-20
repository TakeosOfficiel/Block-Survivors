/** Écran de chargement façon Minecraft (fond de terre assombri, barre de progression). */
import { element } from './dom';

export class EcranChargement {
  private readonly racine = element('chargement');
  private readonly texte = element('chargement-texte');
  private readonly barre = element('chargement-barre');

  /** `urlFond` : texture tuilée en fond ; la couleur CSS sert de secours si elle est indisponible. */
  constructor(urlFond: string) {
    this.racine.style.backgroundImage = `url("${urlFond}")`;
  }

  progression(faites: number, total: number): void {
    const ratio = total > 0 ? faites / total : 1;
    this.texte.textContent = `Chargement des textures… ${faites} / ${total}`;
    this.barre.style.width = `${Math.round(ratio * 100)}%`;
  }

  erreur(message: string): void {
    this.texte.textContent = message;
    this.racine.classList.add('erreur');
  }

  masquer(): void {
    this.racine.classList.add('cache');
  }
}
