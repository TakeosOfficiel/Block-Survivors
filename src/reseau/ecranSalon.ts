/**
 * Écran du multijoueur (bouton « Multijoueur » du menu) : choix de la classe,
 * puis Héberger ou Rejoindre. Chacun joue avec sa classe, son pseudo, son skin
 * et la progression de sa classe (maîtrise et talents, comme en solo) ; la
 * maîtrise progresse aussi à plusieurs, mais les records d'une partie à
 * plusieurs restent à part de ceux du solo. L'hôte ouvre un salon et donne
 * son code court ; ses amis le tapent dans Rejoindre et entrent d'eux-mêmes
 * (signalisation.ts). L'hôte lance la partie quand tout le monde est là.
 */
import { ARENES } from '../donnees/arenes';
import { IDS_PERSONNAGES, PERSONNAGES, type IdPersonnage } from '../donnees/personnages';
import type { DefinitionArene } from '../donnees/types';
import type { ChoixPartie } from '../lancement';
import type { Collection } from '../progression/collection';
import type { Sauvegarde } from '../progression/sauvegarde';
import type { Profil } from '../profil';
import { element } from '../ui/dom';
import { skinPourReseau } from './apparence';
import { SessionReseau, type ProfilReseau } from './session';
import { normaliserCode } from './signalisation';

/** Ce que le salon renvoie au lancement. */
export interface LancementMultijoueur {
  readonly choix: ChoixPartie;
  readonly session: SessionReseau;
}

const STYLE = `
#salon { z-index: 16; }
#salon .salon-corps { width: min(760px, 94vw); max-height: calc(100vh - 80px); overflow-y: auto; display: flex; flex-direction: column; gap: 10px; }
#salon .salon-rangee { display: flex; gap: 8px; flex-wrap: wrap; align-items: center; }
#salon .salon-choix.choisie { border-color: var(--page-or); color: var(--page-or-clair); }
#salon .salon-code { font-size: 44px; letter-spacing: 8px; text-align: center; color: var(--page-or-clair); padding: 10px 0; user-select: all; }
#salon .salon-saisie { width: 260px; font-size: 32px; letter-spacing: 6px; text-align: center; text-transform: uppercase; }
#salon select { min-width: 220px; }
#salon .salon-statut { min-height: 1.4em; color: #9aa0ad; }
#salon .salon-statut.erreur { color: #ff7070; }
#salon .salon-joueurs { display: flex; flex-direction: column; gap: 4px; }
#salon .bouton-mc { padding: 0 14px; }
.reseau-alerte { position: fixed; top: 64px; left: 50%; transform: translateX(-50%); z-index: 20; padding: 8px 16px; color: #ff8080; background: rgba(0, 0, 0, 0.75); border: 2px solid #ff8080; }
`;

export class EcranSalon {
  private readonly racine: HTMLDivElement;
  private readonly corps: HTMLDivElement;
  private readonly solo: Sauvegarde;
  private readonly profil: Profil;
  private readonly collection: Collection;
  private personnage: IdPersonnage;
  private arene: DefinitionArene;
  private session: SessionReseau | null = null;
  private surLancer: ((l: LancementMultijoueur) => void) | null = null;
  private ouvert = false;
  private lance = false;
  private skinReseau: Promise<string | null> = Promise.resolve(null);

  constructor(solo: Sauvegarde, profil: Profil, collection: Collection, choix: ChoixPartie) {
    this.collection = collection;
    this.solo = solo;
    this.profil = profil;
    this.personnage = solo.verrou('personnage', choix.personnage) ? IDS_PERSONNAGES[0] : choix.personnage;
    this.arene = solo.verrou('arene', choix.arene.id) ? ARENES[0] : choix.arene;

    const style = document.createElement('style');
    style.textContent = STYLE;
    document.head.append(style);
    this.racine = document.createElement('div');
    this.racine.id = 'salon';
    this.racine.className = 'ecran page cache';
    this.corps = document.createElement('div');
    this.corps.className = 'salon-corps';
    const titre = document.createElement('h2');
    titre.className = 'ecran-titre';
    titre.textContent = 'Multijoueur';
    this.racine.append(titre, this.corps);
    document.body.append(this.racine);

    element<HTMLButtonElement>('menu-multijoueur').addEventListener('click', () => this.ouvrir());
    // Entrée lancerait la partie solo du menu derrière : bloquée tant que le salon est ouvert ou lancé
    window.addEventListener('keydown', (e) => {
      if (!(this.ouvert || this.lance) || (e.code !== 'Enter' && e.code !== 'NumpadEnter')) return;
      e.stopImmediatePropagation();
      // Entrée dans le champ du code : on rejoint
      if (document.activeElement?.id === 'salon-code-saisie') document.getElementById('salon-entrer')?.click();
    }, true);
  }

  /** Résolue quand l'hôte lance la partie (jamais si le joueur reste en solo). */
  attendreLancement(): Promise<LancementMultijoueur> {
    return new Promise((resoudre) => {
      this.surLancer = resoudre;
    });
  }

  private ouvrir(): void {
    // Skin importée (HD comprise) : une copie au format du jeu, assez légère pour partir sur le réseau
    const p = this.profil;
    this.skinReseau = p.skin === 'importe' && p.skinImporte ? skinPourReseau(p.skinImporte) : Promise.resolve(null);
    this.ouvert = true;
    this.racine.classList.remove('cache');
    this.afficherAccueil();
  }

  private fermer(): void {
    this.ouvert = false;
    this.session?.fermer();
    this.session = null;
    this.racine.classList.add('cache');
  }

  // --- Accueil : classe et arène -------------------------------------------

  private afficherAccueil(): void {
    const c = this.corps;
    c.replaceChildren();

    c.append(texte('Classe', 'titre-panneau'));
    const personnages = rangee();
    for (const id of IDS_PERSONNAGES) {
      const verrou = this.solo.verrou('personnage', id);
      const b = bouton(PERSONNAGES[id].nom, () => {
        this.personnage = id;
        this.afficherAccueil();
      });
      b.classList.add('salon-choix');
      b.classList.toggle('choisie', id === this.personnage);
      b.disabled = verrou !== null;
      b.dataset['id'] = id;
      personnages.append(b);
    }
    c.append(personnages);

    c.append(texte(`Maîtrise ${this.solo.maitrise(this.personnage).niveau} et talents de votre classe, comme en solo. `
      + 'L\'XP de la partie fait monter votre maîtrise ; les records à plusieurs restent à part de ceux du solo.', 'ecran-aide'));

    c.append(texte('Arène (choisie par l\'hôte)', 'titre-panneau'));
    const choixArene = document.createElement('select');
    choixArene.className = 'champ-mc';
    choixArene.id = 'salon-arene';
    for (const a of ARENES) {
      if (this.solo.verrou('arene', a.id)) continue;
      const option = document.createElement('option');
      option.value = a.id;
      option.textContent = a.nom;
      option.selected = a.id === this.arene.id;
      choixArene.append(option);
    }
    choixArene.addEventListener('change', () => {
      this.arene = ARENES.find((a) => a.id === choixArene.value) ?? this.arene;
    });
    c.append(choixArene);

    const actions = rangee();
    const heberger = bouton('Héberger', () => {
      heberger.disabled = true;
      void this.heberger().finally(() => { heberger.disabled = false; });
    });
    heberger.id = 'salon-heberger';
    const rejoindre = bouton('Rejoindre', () => this.afficherRejoindre());
    rejoindre.id = 'salon-rejoindre';
    actions.append(heberger, rejoindre, bouton('Retour', () => this.fermer()));
    c.append(actions);
    c.append(texte('L\'hôte ouvre un salon et donne son code à ses amis, qui le tapent dans Rejoindre. Pas de serveur de jeu : '
      + 'la partie passe directement d\'un joueur à l\'autre. Ça passe derrière la plupart des box ; sinon, essayez en réseau local '
      + 'ou avec Tailscale ou ZeroTier.', 'ecran-aide'));
    c.append(statutVide());
  }

  /** Ce que ce joueur apporte au salon : son pseudo, son skin, sa classe et ses talents. */
  private async profilReseau(): Promise<ProfilReseau> {
    const p = this.profil;
    const skinImporte = await this.skinReseau;
    return {
      pseudo: p.pseudo, personnage: this.personnage, skin: skinImporte ? 'importe' : p.skin === 'importe' ? 'classe' : p.skin, brasFins: p.brasFins, skinImporte,
      bonus: this.solo.bonus(this.personnage),
      armure: { bonus: this.collection.bonus(), portees: this.collection.portees() },
    };
  }

  // --- Hôte ----------------------------------------------------------------

  private async heberger(): Promise<void> {
    this.afficherStatut('Ouverture du salon…');
    try {
      const session = await SessionReseau.heberger(await this.profilReseau(), this.arene.id);
      if (!this.ouvert) {
        session.fermer();
        return;
      }
      this.session = session;
      session.surChangement = () => this.afficherHote();
      this.afficherHote();
    } catch (erreur) {
      this.afficherStatut((erreur as Error).message, true);
    }
  }

  private afficherHote(): void {
    const session = this.session;
    if (!session) return;
    const c = this.corps;
    c.replaceChildren();
    c.append(texte(`Vous hébergez : ${this.arene.nom}`, 'titre-panneau'));
    const code = texte(session.code, 'salon-code panneau-mc');
    code.id = 'salon-code';
    const copier = bouton('Copier le code', () => {
      void navigator.clipboard?.writeText(session.code).then(() => { copier.textContent = 'Copié !'; });
    });
    c.append(code, rangee(copier));
    c.append(texte('Donnez ce code à vos amis : ils cliquent sur Rejoindre et le tapent, puis apparaissent ici d\'eux-mêmes.', 'ecran-aide'));
    c.append(this.listeJoueurs(session));

    const lancer = bouton('Lancer la partie', () => {
      session.lancer();
      this.terminer(session);
    });
    lancer.id = 'salon-lancer';
    lancer.classList.add('page-fermer');
    c.append(rangee(lancer, bouton('Quitter le salon', () => this.fermer())), statutVide());
  }

  // --- Joueur --------------------------------------------------------------

  private afficherRejoindre(): void {
    const c = this.corps;
    c.replaceChildren();
    c.append(texte('Code du salon donné par l\'hôte :', 'titre-panneau'));
    const saisie = document.createElement('input');
    saisie.className = 'champ-mc salon-saisie';
    saisie.id = 'salon-code-saisie';
    saisie.maxLength = 7;
    saisie.placeholder = 'ABC-DEF';
    saisie.spellcheck = false;
    saisie.autocomplete = 'off';
    const entrer = bouton('Rejoindre', () => {
      const code = normaliserCode(saisie.value);
      if (!code) {
        this.afficherStatut('Le code fait 6 lettres ou chiffres, comme K7F-3QX', true);
        return;
      }
      saisie.value = code;
      entrer.disabled = true;
      this.afficherStatut('Recherche du salon…');
      this.profilReseau().then((profil) => SessionReseau.rejoindre(profil, code)).then((session) => {
        if (!this.ouvert) {
          session.fermer();
          return;
        }
        this.session = session;
        session.surChangement = () => this.afficherInvite();
        session.surLancement = () => this.terminer(session);
        session.surPerte = (raison) => this.afficherStatut(raison, true);
        this.afficherInvite();
      }, (erreur: Error) => {
        entrer.disabled = false;
        this.afficherStatut(erreur.message, true);
      });
    });
    entrer.id = 'salon-entrer';
    c.append(rangee(saisie, entrer, bouton('Retour', () => this.afficherAccueil())), statutVide());
    saisie.focus();
  }

  private afficherInvite(): void {
    const session = this.session;
    if (!session) return;
    const c = this.corps;
    c.replaceChildren();
    if (session.joueurs.length === 0) {
      c.append(texte('Relié à l\'hôte, entrée dans le salon…', 'titre-panneau'));
    } else {
      const arene = ARENES.find((a) => a.id === session.arene);
      c.append(texte(`Dans le salon ! Arène : ${arene?.nom ?? session.arene}`, 'titre-panneau'), this.listeJoueurs(session));
      c.append(texte('En attente du lancement par l\'hôte…', 'ecran-aide'));
    }
    c.append(rangee(bouton('Quitter le salon', () => this.fermer())), statutVide());
  }

  // --- Commun --------------------------------------------------------------

  private terminer(session: SessionReseau): void {
    const arene = ARENES.find((a) => a.id === session.arene) ?? this.arene;
    this.ouvert = false;
    this.lance = true;
    this.racine.classList.add('cache');
    element('menu').classList.add('cache');
    this.surLancer?.({ choix: { personnage: this.personnage, arene }, session });
  }

  private listeJoueurs(session: SessionReseau): HTMLElement {
    const liste = document.createElement('div');
    liste.className = 'salon-joueurs panneau-mc';
    liste.id = 'salon-joueurs';
    for (const j of session.joueurs) {
      const role = j.index === 0 ? ' (hôte)' : '';
      const moi = j.index === session.index ? ' — vous' : '';
      liste.append(texte(`${j.pseudo} : ${PERSONNAGES[j.personnage].nom}${role}${moi}`));
    }
    return liste;
  }

  private afficherStatut(message: string, erreur = false): void {
    const statut = this.corps.querySelector('.salon-statut');
    if (!statut) return;
    statut.textContent = message;
    statut.classList.toggle('erreur', erreur);
  }
}

function texte(contenu: string, classe = ''): HTMLDivElement {
  const div = document.createElement('div');
  div.textContent = contenu;
  if (classe) div.className = classe;
  return div;
}

function bouton(libelle: string, action: () => void): HTMLButtonElement {
  const b = document.createElement('button');
  b.className = 'bouton-mc';
  b.textContent = libelle;
  b.addEventListener('click', action);
  return b;
}

function rangee(...enfants: HTMLElement[]): HTMLDivElement {
  const div = document.createElement('div');
  div.className = 'salon-rangee';
  div.append(...enfants);
  return div;
}

function statutVide(): HTMLDivElement {
  return texte('', 'salon-statut');
}
