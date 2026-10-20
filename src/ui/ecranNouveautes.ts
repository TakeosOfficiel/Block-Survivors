/**
 * Nouveautés (depuis le menu) : le journal des versions, lu dans CHANGELOG.md
 * au moment de la construction. S'ouvre tout seul la première fois qu'une
 * nouvelle version est lancée.
 */
import journal from '../../CHANGELOG.md?raw';
import { ecrireStockage, lireStockage } from '../options';
import { VERSION } from '../version';
import { element } from './dom';

const CLE = 'block-survivors.version-vue';
/** Versions affichées (les plus récentes). */
const NOMBRE_VERSIONS = 4;

interface Version {
  readonly titre: string;
  readonly sections: { readonly titre: string; readonly lignes: string[] }[];
}

export class EcranNouveautes {
  private readonly racine = element('nouveautes');

  constructor() {
    const liste = element('nouveautes-liste');
    liste.replaceChildren(...lireJournal(journal).slice(0, NOMBRE_VERSIONS).map((version, index) => {
      const bloc = document.createElement('section');
      bloc.className = 'nouveautes-version';
      const titre = document.createElement('h3');
      titre.className = index === 0 ? 'nouveautes-titre actuelle' : 'nouveautes-titre';
      titre.textContent = version.titre;
      bloc.append(titre);
      for (const section of version.sections) {
        const sous = document.createElement('div');
        sous.className = 'nouveautes-section';
        sous.textContent = section.titre;
        const ul = document.createElement('ul');
        for (const ligne of section.lignes) {
          const li = document.createElement('li');
          li.textContent = ligne;
          ul.append(li);
        }
        bloc.append(sous, ul);
      }
      return bloc;
    }));
    element<HTMLButtonElement>('nouveautes-fermer').addEventListener('click', () => this.fermer());
    window.addEventListener('keydown', (e) => {
      if (this.ouvert && e.code === 'Escape') this.fermer();
    });
  }

  get ouvert(): boolean {
    return !this.racine.classList.contains('cache');
  }

  ouvrir(): void {
    this.racine.classList.remove('cache');
    element('nouveautes-liste').scrollTop = 0;
  }

  /** Ouvre l'écran si cette version n'a encore jamais été lancée ici. */
  ouvrirSiNouvelleVersion(): void {
    if (lireStockage<string>(CLE) === VERSION) return;
    ecrireStockage(CLE, VERSION);
    this.ouvrir();
  }

  private fermer(): void {
    this.racine.classList.add('cache');
  }
}

/** Découpe le journal : « ## [0.6.0] — date », « ### Section », « - ligne ». */
function lireJournal(texte: string): Version[] {
  const versions: { titre: string; sections: { titre: string; lignes: string[] }[] }[] = [];
  for (const brute of texte.split('\n')) {
    const ligne = brute.trim();
    if (ligne.startsWith('## ')) {
      versions.push({ titre: `Version ${ligne.slice(3).replace(/[[\]]/g, '')}`, sections: [] });
    } else if (ligne.startsWith('### ') && versions.length > 0) {
      versions[versions.length - 1].sections.push({ titre: ligne.slice(4), lignes: [] });
    } else if (ligne.startsWith('- ')) {
      const sections = versions[versions.length - 1]?.sections;
      sections?.[sections.length - 1]?.lignes.push(ligne.slice(2));
    }
  }
  return versions;
}
