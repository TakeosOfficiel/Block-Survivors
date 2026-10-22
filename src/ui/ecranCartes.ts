/**
 * Écran de montée de niveau (ou d'ouverture d'un coffre) : 3 cartes
 * d'amélioration en infobulles du jeu, nom coloré selon la famille, choisies
 * à la souris ou avec les touches 1, 2, 3
 * (rangée du haut ou pavé numérique). « Relancer » (ou R) tire d'autres
 * cartes, un nombre limité de fois par partie.
 */
import type { BibliothequeTextures } from '../assets/textures';
import type { Carte } from '../progression/ameliorations';
import { caseImage } from './bilan';
import { element } from './dom';

const TOUCHES: Readonly<Record<string, number>> = {
  Digit1: 0, Digit2: 1, Digit3: 2,
  Numpad1: 0, Numpad2: 1, Numpad3: 2,
};

/** Nouveau tirage de cartes : null quand il ne reste plus de relance. */
export interface Relance {
  restantes(): number;
  relancer(): readonly Carte[] | null;
}

/** Délai avant qu'un clic ou une touche soit accepté (évite un choix involontaire en pleine action). */
const DELAI_ACTIVATION_MS = 350;

export class EcranCartes {
  private readonly racine = element('cartes');
  private readonly titre = element('cartes-titre');
  private readonly liste = element('cartes-liste');
  private readonly boutonRelancer = element<HTMLButtonElement>('cartes-relancer');
  private relance: Relance | null = null;
  private readonly textures: BibliothequeTextures;
  private cartes: readonly Carte[] = [];
  private surChoix: ((carte: Carte) => void) | null = null;
  private ouvertureMs = 0;

  constructor(textures: BibliothequeTextures) {
    this.textures = textures;
    window.addEventListener('keydown', this.surTouche);
    this.boutonRelancer.addEventListener('click', () => this.relancer());
  }

  get ouvert(): boolean {
    return this.surChoix !== null;
  }

  /** `titre` : « Niveau 5 ! », « Coffre ! »… */
  ouvrir(titre: string, cartes: readonly Carte[], surChoix: (carte: Carte) => void, relance: Relance): void {
    this.surChoix = surChoix;
    this.relance = relance;
    this.ouvertureMs = performance.now();
    this.titre.textContent = titre;
    this.afficherCartes(cartes);
    this.racine.classList.remove('cache');
  }

  private afficherCartes(cartes: readonly Carte[]): void {
    this.cartes = cartes;
    this.liste.replaceChildren(...cartes.map((carte, i) => this.creerCarte(carte, i)));
    const restantes = this.relance?.restantes() ?? 0;
    this.boutonRelancer.disabled = restantes === 0;
    this.boutonRelancer.textContent = restantes === 0
      ? 'Plus de relance'
      : `Relancer les cartes (R) · ${restantes} restante${restantes > 1 ? 's' : ''}`;
  }

  private relancer(): void {
    if (!this.ouvert || performance.now() - this.ouvertureMs < DELAI_ACTIVATION_MS) return;
    const cartes = this.relance?.relancer();
    if (!cartes) return;
    this.ouvertureMs = performance.now();
    this.afficherCartes(cartes);
  }

  private fermer(): void {
    this.surChoix = null;
    this.racine.classList.add('cache');
  }

  private choisir(index: number): void {
    const carte = this.cartes[index];
    const rappel = this.surChoix;
    if (!carte || !rappel || performance.now() - this.ouvertureMs < DELAI_ACTIVATION_MS) return;
    this.fermer();
    rappel(carte);
  }

  private readonly surTouche = (e: KeyboardEvent): void => {
    if (!this.ouvert) return;
    if (e.code === 'KeyR' && !e.repeat) {
      e.preventDefault();
      this.relancer();
      return;
    }
    const index = TOUCHES[e.code];
    if (index === undefined) return;
    e.preventDefault();
    this.choisir(index);
  };

  private creerCarte(carte: Carte, index: number): HTMLElement {
    const bouton = document.createElement('button');
    bouton.className = 'carte infobulle';
    bouton.addEventListener('click', () => this.choisir(index));

    const touche = document.createElement('span');
    touche.className = 'carte-touche';
    touche.textContent = `[${index + 1}]`;

    const nom = document.createElement('div');
    nom.className = `carte-nom rarete-${carte.rarete}`;
    nom.textContent = carte.nom;

    const etiquette = document.createElement('div');
    etiquette.className = 'carte-etiquette';
    etiquette.textContent = carte.etiquette;

    const description = document.createElement('div');
    description.className = 'carte-description';
    description.textContent = carte.description;

    const lignes = document.createElement('ul');
    lignes.className = 'carte-stats';
    for (const ligne of carte.lignes) {
      const item = document.createElement('li');
      item.textContent = ligne;
      lignes.append(item);
    }

    bouton.append(touche, caseImage(this.textures.url(carte.icone), carte.nom, 'grande'), nom, etiquette, description, lignes);
    return bouton;
  }
}
