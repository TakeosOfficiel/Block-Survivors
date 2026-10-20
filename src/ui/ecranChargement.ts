/** Écran de chargement façon Minecraft (fond de terre assombri, étape, barre de progression). */
import { element } from './dom';

export class EcranChargement {
  private readonly racine = element('chargement');
  private readonly texte = element('chargement-texte');
  private readonly barre = element('chargement-barre');
  private libelle = 'Chargement…';

  /** `urlFond` : texture tuilée en fond ; la couleur CSS sert de secours si elle est indisponible. */
  constructor(urlFond: string) {
    this.racine.style.backgroundImage = `url("${urlFond}")`;
  }

  /** Nouvelle étape : libellé affiché, barre remise à zéro. */
  etape(libelle: string): void {
    this.libelle = libelle;
    this.texte.textContent = libelle;
    this.barre.style.width = '0%';
  }

  progression(faites: number, total: number): void {
    const ratio = total > 0 ? faites / total : 1;
    this.texte.textContent = `${this.libelle} ${faites} / ${total}`;
    this.barre.style.width = `${Math.round(ratio * 100)}%`;
  }

  erreur(message: string): void {
    this.texte.textContent = message;
    this.racine.classList.add('erreur');
  }

  afficher(): void {
    this.racine.classList.remove('cache');
  }

  masquer(): void {
    this.racine.classList.add('cache');
  }
}
