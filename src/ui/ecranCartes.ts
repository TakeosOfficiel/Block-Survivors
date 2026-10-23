/**
 * Écran de montée de niveau (ou d'ouverture d'un coffre) : 3 cartes
 * d'amélioration debout, au liseré d'or et au losange émeraude (comme
 * l'écran de chargement). La rareté se lit d'un coup d'œil : bandeau de sa
 * couleur en haut, liseré intérieur, halo et fond teinté ; choisies
 * à la souris ou avec les touches 1, 2, 3
 * (rangée du haut ou pavé numérique). « Relancer » (ou R) tire d'autres
 * cartes, un nombre limité de fois par partie.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { NOMS_RARETE, type Carte } from '../progression/ameliorations';
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
  private readonly texteRelancer = element('cartes-relancer-texte');
  private readonly resteRelancer = element('cartes-relancer-reste');
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
    this.texteRelancer.textContent = restantes === 0 ? 'Plus de relance' : '⟳ Relancer';
    this.resteRelancer.textContent = restantes === 0 ? '' : `· ${restantes} restante${restantes > 1 ? 's' : ''}`;
  }

  private relancer(): void {
    if (!this.ouvert || performance.now() - this.ouvertureMs < DELAI_ACTIVATION_MS) return;
    const cartes = this.relance?.relancer();
    if (!cartes) return;
    this.ouvertureMs = performance.now();
    this.afficherCartes(cartes);
  }

  /** Choisit une carte au hasard sans attendre (multijoueur : temps de choix écoulé). */
  choisirAuHasard(): void {
    if (!this.ouvert || this.cartes.length === 0) return;
    this.ouvertureMs = -Infinity;
    this.choisir(Math.floor(Math.random() * this.cartes.length));
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
    bouton.className = `carte orne famille-${carte.rarete}`;
    bouton.addEventListener('click', () => this.choisir(index));

    const joyau = document.createElement('i');
    joyau.className = 'joyau h';

    // Rareté : bandeau de sa couleur en haut de la carte (liseré et halo de la même couleur)
    const rarete = document.createElement('div');
    rarete.className = 'carte-rarete';
    rarete.textContent = NOMS_RARETE[carte.rarete];

    const touche = document.createElement('kbd');
    touche.className = 'carte-touche';
    touche.textContent = String(index + 1);

    const icone = document.createElement('div');
    icone.className = 'carte-icone';
    const url = this.textures.url(carte.icone);
    if (url) {
      const image = document.createElement('img');
      image.src = url;
      image.alt = '';
      icone.append(image);
    }

    const nom = document.createElement('div');
    nom.className = 'carte-nom';
    nom.textContent = carte.nom;

    const etiquette = document.createElement('span');
    etiquette.className = 'carte-etiquette';
    etiquette.textContent = carte.etiquette;

    const description = document.createElement('div');
    description.className = 'carte-description';
    description.textContent = carte.description;

    // « Dégâts : 10 → 15 » : la nouvelle valeur ressort
    const lignes = document.createElement('ul');
    lignes.className = 'carte-stats';
    for (const ligne of carte.lignes) {
      const item = document.createElement('li');
      const fleche = ligne.lastIndexOf('→ ');
      if (fleche < 0) {
        item.textContent = ligne;
      } else {
        const valeur = document.createElement('b');
        valeur.textContent = ligne.slice(fleche + 2);
        item.append(ligne.slice(0, fleche + 2), valeur);
      }
      lignes.append(item);
    }

    bouton.append(joyau, rarete, touche, icone, nom);
    if (carte.etiquette !== NOMS_RARETE[carte.rarete]) bouton.append(etiquette);
    bouton.append(description);
    if (carte.lignes.length > 0) bouton.append(lignes);
    return bouton;
  }
}
