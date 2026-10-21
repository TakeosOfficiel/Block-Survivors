/**
 * Écran de chargement : le logo et la barre d'expérience du jeu ; pendant le
 * chargement d'une partie, la photo de l'arène (voir assets/photosArenes.ts)
 * encadrée dans un panneau du jeu, avec son nom et une astuce. Une seule barre pour tout le chargement : planifier() annonce le
 * nombre d'étapes, chacune en remplit une part égale, et la barre ne recule
 * jamais ; seul le libellé change.
 */
import { photoArene } from '../assets/photosArenes';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ASTUCES } from '../donnees/astuces';
import type { DefinitionArene } from '../donnees/types';
import { element } from './dom';

export class EcranChargement {
  private readonly racine = element('chargement');
  private readonly fenetre = element('chargement-fenetre');
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

  /** Sprites de la barre d'expérience, une fois les textures de l'interface chargées. */
  habiller(textures: BibliothequeTextures): void {
    const sprites = CONFIG.hud.barreXp;
    const fond = textures.url(sprites.fond);
    const progression = textures.url(sprites.progression);
    if (fond) element('chargement-cadre').style.backgroundImage = `url("${fond}")`;
    if (progression) this.barre.style.backgroundImage = `url("${progression}")`;
  }

  /** L'arène de la partie qui se charge : sa photo encadrée, son nom et une astuce. */
  decor(arene: DefinitionArene): void {
    this.arene.textContent = arene.nom;
    this.astuce.textContent = `Astuce : ${ASTUCES[Math.floor(Math.random() * ASTUCES.length)]}`;
    this.fenetre.classList.remove('cache');
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
