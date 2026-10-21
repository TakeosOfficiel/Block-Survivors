/**
 * Écran de chargement façon WoW : une photo de l'arène en plein écran (voir
 * assets/photosArenes.ts), son nom, une astuce, et la barre de progression en
 * bas. Une seule barre pour tout le chargement : planifier() annonce le
 * nombre d'étapes, chacune en remplit une part égale, et la barre ne recule
 * jamais ; seul le libellé change.
 */
import { photoArene } from '../assets/photosArenes';
import { ASTUCES } from '../donnees/astuces';
import type { DefinitionArene } from '../donnees/types';
import { element } from './dom';

export class EcranChargement {
  private readonly racine = element('chargement');
  private readonly image = element('chargement-image');
  private readonly arene = element('chargement-arene');
  private readonly astuce = element('chargement-astuce');
  private readonly texte = element('chargement-texte');
  private readonly barre = element('chargement-barre');
  private libelle = 'Chargement…';
  private etapes = 1;
  private index = -1;
  /** Part déjà remplie (0 à 1). */
  private rempli = 0;
  /** Arène dont la photo est demandée (une demande plus récente l'emporte). */
  private areneVoulue = '';

  /**
   * L'arène qui se charge : sa photo en fond, et son nom (`avecNom`, faux au
   * lancement du jeu, où l'on montre seulement la dernière arène jouée).
   */
  decor(arene: DefinitionArene, avecNom: boolean): void {
    this.arene.textContent = avecNom ? arene.nom : '';
    this.astuce.textContent = avecNom ? `Astuce : ${ASTUCES[Math.floor(Math.random() * ASTUCES.length)]}` : '';
    if (arene.id === this.areneVoulue) return;
    this.areneVoulue = arene.id;
    this.image.classList.remove('visible');
    void photoArene(arene.id).then((adresse) => {
      if (!adresse || this.areneVoulue !== arene.id) return;
      const image = new Image();
      image.onload = () => {
        if (this.areneVoulue !== arene.id) return;
        this.image.style.backgroundImage = `url("${adresse}")`;
        this.image.classList.add('visible');
      };
      image.src = adresse;
    });
  }

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

  private remplir(part: number): void {
    this.rempli = Math.max(this.rempli, Math.min(1, part));
    const pourcent = Math.round(this.rempli * 100);
    this.texte.textContent = `${this.libelle} ${pourcent} %`;
    this.barre.style.width = `${pourcent}%`;
  }
}
