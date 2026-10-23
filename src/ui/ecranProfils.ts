/**
 * Profils (bouton « Profils » du menu) : la liste des profils de cette
 * installation (pseudo, maîtrise de chaque classe), pour jouer avec un autre,
 * en créer un, en supprimer un, ou transférer le profil actif (export et
 * import, progression/transfert.ts). Changer de profil recharge le jeu.
 */
import { IDS_PERSONNAGES, PERSONNAGES } from '../donnees/personnages';
import { niveauMaitrise } from '../donnees/talents';
import { exporterProfil, importerProfil } from '../progression/transfert';
import { LONGUEUR_PSEUDO, nettoyerPseudo, pseudoAleatoire } from '../profil';
import {
  changerProfil, creerProfil, ecrireDonneeBrute, idsProfils, lireDonneeBrute, peutCreerProfil, profilActif, relancerSurLeMenu,
  supprimerProfil,
} from '../profils';
import { element } from './dom';

/** Ce que la liste montre d'un profil, lu dans ses données sans le charger. */
interface ResumeProfil {
  readonly id: string;
  readonly pseudo: string;
  /** « Guerrier 3 · Mage 1 » : maîtrise des classes jouées. */
  readonly maitrises: string;
}

export class EcranProfils {
  private readonly racine = element('profils');
  private readonly liste = element('profils-liste');
  private readonly pseudo = element<HTMLInputElement>('profils-pseudo');
  private readonly boutonCreer = element<HTMLButtonElement>('profils-creer');
  private readonly fichier = element<HTMLInputElement>('profils-fichier');
  private readonly texte = element('profils-texte');

  constructor() {
    element<HTMLButtonElement>('menu-profils').addEventListener('click', () => this.ouvrir());
    element<HTMLButtonElement>('profils-fermer').addEventListener('click', () => this.fermer());
    this.pseudo.maxLength = LONGUEUR_PSEUDO;
    this.pseudo.addEventListener('keydown', (e) => {
      // Les touches tapées ici ne lancent pas la partie du menu
      e.stopPropagation();
      if (e.code === 'Enter') this.creer();
    });
    this.boutonCreer.addEventListener('click', () => this.creer());
    element<HTMLButtonElement>('profils-exporter').addEventListener('click', async () => {
      await exporterProfil();
      this.texte.textContent = 'Profil exporté : dans l\'autre version du jeu, Profils → « Importer un profil ».';
    });
    element<HTMLButtonElement>('profils-importer').addEventListener('click', () => this.fichier.click());
    this.fichier.addEventListener('change', async () => {
      const choisi = this.fichier.files?.[0];
      this.fichier.value = '';
      if (!choisi) return;
      const erreur = await importerProfil(choisi);
      if (erreur) this.texte.textContent = erreur;
      else relancerSurLeMenu();
    });
    // Entrée lancerait la partie du menu derrière, Échap ferme
    window.addEventListener('keydown', (e) => {
      if (!this.ouvert) return;
      if (e.code === 'Enter') e.stopImmediatePropagation();
      if (e.code === 'Escape') this.fermer();
    }, true);
  }

  get ouvert(): boolean {
    return !this.racine.classList.contains('cache');
  }

  ouvrir(): void {
    this.pseudo.value = '';
    this.pseudo.placeholder = pseudoAleatoire();
    this.dessiner();
    this.racine.classList.remove('cache');
  }

  private fermer(): void {
    this.racine.classList.add('cache');
  }

  private dessiner(): void {
    const actif = profilActif();
    const resumes = idsProfils().map(resumer);
    this.liste.replaceChildren(...resumes.map((r) => {
      const ligne = document.createElement('div');
      ligne.className = 'panneau-mc profils-ligne';
      ligne.classList.toggle('actif', r.id === actif);
      const infos = document.createElement('div');
      infos.className = 'profils-infos';
      const nom = document.createElement('div');
      nom.className = 'option-nom';
      nom.textContent = r.id === actif ? `${r.pseudo} (profil actuel)` : r.pseudo;
      const detail = document.createElement('div');
      detail.className = 'ecran-aide';
      detail.textContent = r.maitrises || 'Aucune partie jouée';
      infos.append(nom, detail);
      ligne.append(infos);
      if (r.id !== actif) {
        ligne.append(bouton('Jouer avec', () => {
          changerProfil(r.id);
          relancerSurLeMenu();
        }));
      }
      if (resumes.length > 1) {
        ligne.append(bouton('Supprimer', () => {
          if (!window.confirm(`Supprimer le profil « ${r.pseudo} » et toute sa progression ? C'est définitif.`)) return;
          supprimerProfil(r.id);
          if (r.id === actif) relancerSurLeMenu();
          else this.dessiner();
        }));
      }
      return ligne;
    }));
    this.boutonCreer.disabled = !peutCreerProfil();
    this.pseudo.disabled = !peutCreerProfil();
  }

  /** Nouveau profil, vierge, avec ce pseudo ; le jeu redémarre dessus. */
  private creer(): void {
    if (!peutCreerProfil()) return;
    const pseudo = nettoyerPseudo(this.pseudo.value) || this.pseudo.placeholder;
    creerProfil((id) => ecrireDonneeBrute(id, 'profil', JSON.stringify({ pseudo, skin: 'classe', brasFins: false, skinImporte: null })));
    relancerSurLeMenu();
  }
}

function resumer(id: string): ResumeProfil {
  const profil = lireJson<{ pseudo?: unknown }>(lireDonneeBrute(id, 'profil'));
  const progression = lireJson<{ maitrise?: Record<string, unknown> }>(lireDonneeBrute(id, 'progression'));
  const maitrises = IDS_PERSONNAGES.flatMap((p) => {
    const xp = progression?.maitrise?.[p];
    return typeof xp === 'number' && xp > 0 ? [`${PERSONNAGES[p].nom} ${niveauMaitrise(xp)}`] : [];
  });
  return {
    id,
    pseudo: typeof profil?.pseudo === 'string' && profil.pseudo ? nettoyerPseudo(profil.pseudo) : 'Joueur',
    maitrises: maitrises.join(' · '),
  };
}

function lireJson<T>(brut: string | null): T | null {
  if (!brut) return null;
  try {
    return JSON.parse(brut) as T;
  } catch {
    return null;
  }
}

function bouton(libelle: string, action: () => void): HTMLButtonElement {
  const b = document.createElement('button');
  b.className = 'bouton-mc';
  b.textContent = libelle;
  b.addEventListener('click', action);
  return b;
}
