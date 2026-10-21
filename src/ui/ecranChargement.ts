/**
 * Écran de chargement : la vraie map de la partie qui apparaît chunk par
 * chunk (voir carteChargement.ts), le nom de l'arène, l'étape en cours et une
 * barre de progression. Une seule barre pour tout le
 * chargement : planifier() annonce le nombre d'étapes, chacune en remplit une
 * part égale, et la barre ne recule jamais ; seul le libellé change.
 */
import type { VueDessus } from '../monde/vueDessus';
import { CarteChargement } from './carteChargement';
import { element } from './dom';

export class EcranChargement {
  private readonly racine = element('chargement');
  private readonly texte = element('chargement-texte');
  private readonly barre = element('chargement-barre');
  private readonly arene = element('chargement-arene');
  private readonly carte = new CarteChargement();
  private libelle = 'Chargement…';
  private etapes = 1;
  private index = -1;
  /** Part déjà remplie (0 à 1). */
  private rempli = 0;

  /** Nouveau chargement de `etapes` étapes : la barre repart de zéro. */
  planifier(etapes: number): void {
    this.etapes = Math.max(1, etapes);
    this.index = -1;
    this.rempli = 0;
    this.barre.style.width = '0%';
  }

  /** Étape suivante : nouveau libellé, la barre continue depuis où elle en est. */
  etape(libelle: string): void {
    this.libelle = libelle;
    this.index = Math.min(this.index + 1, this.etapes - 1);
    this.remplir(this.index / this.etapes);
  }

  progression(faites: number, total: number): void {
    const ratio = total > 0 ? faites / total : 1;
    this.remplir((Math.max(0, this.index) + ratio) / this.etapes);
  }

  /** Arène de la partie (son nom) et taille de sa map : la grille de ses chunks s'affiche, encore vide. */
  annoncerCarte(nom: string, largeur: number, profondeur: number): void {
    this.arene.textContent = nom;
    this.carte.annoncer(largeur, profondeur);
  }

  /** La map vue de dessus est prête : ses chunks s'allument avec la suite du chargement. */
  montrerCarte(vue: VueDessus, tete: CanvasImageSource | null): void {
    this.carte.montrer(vue, this.rempli, tete);
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
    this.arene.textContent = '';
    this.carte.cacher();
  }

  private remplir(part: number): void {
    this.rempli = Math.max(this.rempli, Math.min(1, part));
    this.carte.progression(this.rempli);
    const pourcent = Math.round(this.rempli * 100);
    this.texte.textContent = `${this.libelle} ${pourcent} %`;
    this.barre.style.width = `${pourcent}%`;
  }
}
