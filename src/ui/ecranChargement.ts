/**
 * Écran de chargement : la scène « La pause des mobs » derrière le logo (fond
 * de terre si elle est indisponible), une phrase des mobs et une barre de
 * progression. Une seule barre pour tout le chargement : planifier() annonce
 * le nombre d'étapes, chacune en remplit une part égale, et la barre ne
 * recule jamais ; la phrase suit l'avancement (une par cinquième de la barre).
 */
import { PHRASES_FEU_DE_CAMP } from '../donnees/feuDeCamp';
import { element } from './dom';
import type { SceneChargement } from './sceneChargement';

export class EcranChargement {
  private readonly racine = element('chargement');
  private readonly texte = element('chargement-texte');
  private readonly barre = element('chargement-barre');
  private scene: SceneChargement | null = null;
  private etapes = 1;
  private index = -1;
  /** Part déjà remplie (0 à 1). */
  private rempli = 0;

  /** `urlFond` : texture tuilée en fond ; la couleur CSS sert de secours si elle est indisponible. */
  constructor(urlFond: string) {
    this.racine.style.backgroundImage = `url("${urlFond}")`;
  }

  /** Nouveau chargement de `etapes` étapes : la barre repart de zéro. */
  planifier(etapes: number): void {
    this.etapes = Math.max(1, etapes);
    this.index = -1;
    this.rempli = 0;
    this.barre.style.width = '0%';
  }

  /** Branche la scène 3D (elle arrive après le premier affichage, une fois ses textures chargées). */
  brancher(scene: SceneChargement): void {
    this.scene = scene;
    scene.definirProgression(this.rempli);
    if (!this.racine.classList.contains('cache')) scene.demarrer();
    this.racine.classList.add('avec-scene');
  }

  /** Étape suivante : la barre continue depuis où elle en est (le vrai libellé reste lisible dans data-etape). */
  etape(libelle: string): void {
    this.racine.dataset.etape = libelle;
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
    this.scene?.demarrer();
  }

  masquer(): void {
    this.racine.classList.add('cache');
    this.scene?.arreter();
  }

  /** Fin d'un chargement de partie : la petite scène de fin à 100 %, puis l'écran s'efface. */
  async terminer(): Promise<void> {
    this.remplir(1);
    await this.scene?.finale();
    this.masquer();
  }

  private remplir(part: number): void {
    this.rempli = Math.max(this.rempli, Math.min(1, part));
    this.scene?.definirProgression(this.rempli);
    const pourcent = Math.round(this.rempli * 100);
    const phrase = PHRASES_FEU_DE_CAMP[Math.min(PHRASES_FEU_DE_CAMP.length - 1, Math.floor(this.rempli * PHRASES_FEU_DE_CAMP.length))];
    this.texte.textContent = `${phrase} ${pourcent} %`;
    this.barre.style.width = `${pourcent}%`;
  }
}
