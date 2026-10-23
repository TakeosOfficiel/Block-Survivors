/** Bandeau d'information du multijoueur (joueur à terre, connexion perdue), en haut de l'écran. */
export class Bandeau {
  private readonly element: HTMLDivElement;
  private texte = '';

  constructor() {
    this.element = document.createElement('div');
    this.element.className = 'reseau-alerte cache';
    document.body.append(this.element);
  }

  /** Affiche le texte (vide : masque) ; le DOM ne change que si le texte change. */
  afficher(texte: string): void {
    if (texte === this.texte) return;
    this.texte = texte;
    this.element.textContent = texte;
    this.element.classList.toggle('cache', texte === '');
  }
}

/** Texte du bandeau pour le joueur local à terre (vide s'il est debout). */
export function texteATerre(mort: boolean, tempsMort: number, aTerre: number, reanimation: number): string {
  if (!mort) return '';
  if (tempsMort >= aTerre) return 'Vous êtes mort : vos alliés continuent sans vous';
  if (reanimation > 0) return `Réanimation : ${Math.floor(reanimation * 10) * 10} %`;
  return `À terre ! Un allié doit rester près de vous pour vous relever (${Math.ceil(aTerre - tempsMort)} s)`;
}
