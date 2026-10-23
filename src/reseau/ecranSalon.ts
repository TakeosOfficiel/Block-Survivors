/**
 * Écran du multijoueur (bouton « Multijoueur » du menu) : choix de la classe,
 * puis Héberger ou Rejoindre. Chacun joue avec sa classe, son pseudo, son skin
 * et la progression de sa classe (maîtrise et talents, comme en solo) ; la
 * maîtrise progresse aussi à plusieurs, mais les records d'une partie à
 * plusieurs restent à part de ceux du solo. La mise en relation se fait par
 * deux codes à copier-coller : l'invitation de l'hôte, puis la réponse du
 * joueur. L'hôte lance la partie quand tout le monde est là.
 */
import { ARENES } from '../donnees/arenes';
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import { IDS_PERSONNAGES, PERSONNAGES, type IdPersonnage } from '../donnees/personnages';
import type { DefinitionArene } from '../donnees/types';
import type { ChoixPartie } from '../lancement';
import type { Collection } from '../progression/collection';
import type { Sauvegarde } from '../progression/sauvegarde';
import type { Profil } from '../profil';
import { element } from '../ui/dom';
import { SessionReseau, type Invitation, type ProfilReseau } from './session';

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
#salon textarea { width: 100%; height: 76px; resize: none; font-family: monospace; font-size: 11px; word-break: break-all; }
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
  private invitation: Invitation | null = null;
  private surLancer: ((l: LancementMultijoueur) => void) | null = null;
  private ouvert = false;
  private lance = false;

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
      if ((this.ouvert || this.lance) && e.code === 'Enter') e.stopImmediatePropagation();
    }, true);
  }

  /** Résolue quand l'hôte lance la partie (jamais si le joueur reste en solo). */
  attendreLancement(): Promise<LancementMultijoueur> {
    return new Promise((resoudre) => {
      this.surLancer = resoudre;
    });
  }

  private ouvrir(): void {
    this.ouvert = true;
    this.racine.classList.remove('cache');
    this.afficherAccueil();
  }

  private fermer(): void {
    this.ouvert = false;
    this.session?.fermer();
    this.session = null;
    this.invitation?.annuler();
    this.invitation = null;
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
    const heberger = bouton('Héberger', () => this.heberger());
    heberger.id = 'salon-heberger';
    const rejoindre = bouton('Rejoindre', () => this.afficherRejoindre());
    rejoindre.id = 'salon-rejoindre';
    actions.append(heberger, rejoindre, bouton('Retour', () => this.fermer()));
    c.append(actions);
    c.append(texte('Sans serveur : l\'hôte envoie un code d\'invitation (Discord, SMS…), le joueur renvoie un code de réponse. '
      + 'Ça passe derrière la plupart des box ; sinon, essayez en réseau local ou avec Tailscale ou ZeroTier.', 'ecran-aide'));
  }

  /** Ce que ce joueur apporte au salon : son pseudo, son skin, sa classe et ses talents. */
  private profilReseau(): ProfilReseau {
    const p = this.profil;
    return {
      pseudo: p.pseudo, personnage: this.personnage, skin: p.skin, brasFins: p.brasFins, skinImporte: p.skin === 'importe' ? p.skinImporte : null,
      bonus: this.solo.bonus(this.personnage),
      armure: { bonus: this.collection.bonus(), portees: this.collection.portees() },
    };
  }

  // --- Hôte ----------------------------------------------------------------

  private heberger(): void {
    const session = SessionReseau.heberger(this.profilReseau(), this.arene.id);
    this.session = session;
    session.surChangement = () => this.afficherHote();
    this.afficherHote();
  }

  private afficherHote(statut = '', erreur = false): void {
    const session = this.session;
    if (!session) return;
    const c = this.corps;
    c.replaceChildren();
    c.append(texte(`Vous hébergez : ${this.arene.nom}`, 'titre-panneau'), this.listeJoueurs(session));

    const invitation = this.invitation;
    if (invitation) {
      c.append(texte('1. Envoyez ce code d\'invitation au joueur :', 'ecran-aide'));
      c.append(zoneCode(invitation.code, 'salon-code-invitation'));
      c.append(texte('2. Collez ici le code de réponse qu\'il vous renvoie :', 'ecran-aide'));
      const reponse = zoneSaisie('salon-code-reponse');
      c.append(reponse);
      const valider = bouton('Valider la réponse', () => {
        valider.disabled = true;
        this.afficherStatut('Connexion…');
        invitation.accepter(reponse.value).then(() => {
          this.invitation = null;
          this.afficherHote('Joueur connecté !');
        }, (erreur: Error) => {
          valider.disabled = false;
          this.afficherStatut(erreur.message, true);
        });
      });
      valider.id = 'salon-valider';
      c.append(rangee(valider, bouton('Annuler l\'invitation', () => {
        invitation.annuler();
        this.invitation = null;
        this.afficherHote();
      })));
    }

    const actions = rangee();
    const inviter = bouton('Inviter un joueur', () => {
      inviter.disabled = true;
      this.afficherStatut('Préparation du code (quelques secondes)…');
      session.inviter().then((i) => {
        this.invitation = i;
        this.afficherHote();
      }, (erreur: Error) => this.afficherStatut(erreur.message, true));
    });
    inviter.id = 'salon-inviter';
    inviter.disabled = invitation !== null || session.nombreJoueurs >= REGLES_MULTIJOUEUR.joueursMax;
    const lancer = bouton('Lancer la partie', () => {
      session.lancer();
      this.terminer(session);
    });
    lancer.id = 'salon-lancer';
    lancer.classList.add('page-fermer');
    actions.append(inviter, lancer, bouton('Quitter le salon', () => this.fermer()));
    c.append(actions, statutVide());
    if (statut) this.afficherStatut(statut, erreur);
  }

  // --- Joueur --------------------------------------------------------------

  private afficherRejoindre(): void {
    const c = this.corps;
    c.replaceChildren();
    c.append(texte('1. Collez le code d\'invitation de l\'hôte :', 'ecran-aide'));
    const offre = zoneSaisie('salon-code-offre');
    c.append(offre);
    const generer = bouton('Générer ma réponse', () => {
      generer.disabled = true;
      this.afficherStatut('Préparation de la réponse (quelques secondes)…');
      SessionReseau.rejoindre(this.profilReseau(), offre.value).then(({ session, code }) => {
        this.session = session;
        session.surChangement = () => this.afficherInvite(code);
        session.surLancement = () => this.terminer(session);
        session.surPerte = (raison) => this.afficherStatut(raison, true);
        this.afficherInvite(code);
      }, (erreur: Error) => {
        generer.disabled = false;
        this.afficherStatut(erreur.message, true);
      });
    });
    generer.id = 'salon-generer';
    c.append(rangee(generer, bouton('Retour', () => this.afficherAccueil())), statutVide());
  }

  private afficherInvite(code: string): void {
    const session = this.session;
    if (!session) return;
    const c = this.corps;
    c.replaceChildren();
    if (session.joueurs.length === 0) {
      c.append(texte('2. Renvoyez ce code de réponse à l\'hôte :', 'ecran-aide'));
      c.append(zoneCode(code, 'salon-code-retour'));
      c.append(texte('En attente de l\'hôte…', 'ecran-aide'));
    } else {
      const arene = ARENES.find((a) => a.id === session.arene);
      c.append(texte(`Connecté ! Arène : ${arene?.nom ?? session.arene}`, 'titre-panneau'), this.listeJoueurs(session));
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

function zoneSaisie(id: string): HTMLTextAreaElement {
  const zone = document.createElement('textarea');
  zone.className = 'champ-mc';
  zone.id = id;
  zone.spellcheck = false;
  zone.placeholder = 'BS1.…';
  return zone;
}

/** Code à copier : sélectionné au clic, avec un bouton « Copier ». */
function zoneCode(code: string, id: string): HTMLElement {
  const zone = zoneSaisie(id);
  zone.readOnly = true;
  zone.value = code;
  zone.addEventListener('focus', () => zone.select());
  const copier = bouton('Copier', () => {
    void navigator.clipboard?.writeText(code).then(() => { copier.textContent = 'Copié !'; }, () => zone.select());
  });
  const div = document.createElement('div');
  div.append(zone, rangee(copier));
  return div;
}
