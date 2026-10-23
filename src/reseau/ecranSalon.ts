/**
 * Page Multijoueur (bouton « Multijoueur » du menu), au style des autres pages :
 * à gauche la partie à plusieurs, à droite les amis.
 *
 * Chacun joue avec la classe choisie au menu, son pseudo, son skin et la
 * progression de sa classe (maîtrise et talents, comme en solo) ; l'arène est
 * celle de l'hôte (choisie à son menu). L'hôte ouvre un salon et donne son code
 * court, ou invite ses amis d'un clic ; un ami en ligne qui héberge se rejoint
 * d'un clic, un ami au menu peut être sollicité (« Proposer de jouer » : s'il
 * accepte, il héberge et l'invitation qui revient est acceptée d'office).
 * Invitations et demandes arrivent en notifications sur le menu.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { ARENES } from '../donnees/arenes';
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import { PERSONNAGES, type IdPersonnage } from '../donnees/personnages';
import type { ChoixPartie } from '../lancement';
import type { Collection } from '../progression/collection';
import type { Sauvegarde } from '../progression/sauvegarde';
import type { Profil } from '../profil';
import { element } from '../ui/dom';
import { portrait } from '../ui/portraits';
import { skinPourReseau } from './apparence';
import type { Ami, ServiceAmis } from './amis';
import type { InfoJoueur } from './protocole';
import { SessionReseau, type ProfilReseau } from './session';
import { LONGUEUR_CODE_SALON, normaliserCode } from './signalisation';

/** Ce que le salon renvoie au lancement. */
export interface LancementMultijoueur {
  readonly choix: ChoixPartie;
  readonly session: SessionReseau;
}

/** s avant qu'une notification sans réponse ne s'efface. */
const DUREE_NOTIFICATION = 60;

export class EcranSalon {
  /** Classe et arène choisies au menu (branché par main.ts une fois le menu créé). */
  choixMenu: () => ChoixPartie;

  private readonly racine = element('salon');
  private readonly contenu = element('salon-contenu');
  private readonly statut = element('salon-statut');
  private readonly listeAmis = element('amis-liste');
  private readonly notifications = element('amis-notifications');
  private readonly textures: BibliothequeTextures;
  private readonly solo: Sauvegarde;
  private readonly profil: Profil;
  private readonly collection: Collection;
  private readonly amis: ServiceAmis;
  private session: SessionReseau | null = null;
  /** Une connexion est en cours (héberger ou rejoindre) : les autres actions attendent. */
  private occupe = false;
  private surLancer: ((l: LancementMultijoueur) => void) | null = null;
  private ouvert = false;
  private lance = false;
  private skinReseau: Promise<string | null> = Promise.resolve(null);
  /** Amis à qui l'on a proposé de jouer : leur invitation sera acceptée d'office. */
  private readonly propositions = new Set<string>();
  /** Amis déjà invités dans le salon ouvert. */
  private readonly invites = new Set<string>();

  constructor(textures: BibliothequeTextures, solo: Sauvegarde, profil: Profil, collection: Collection, amis: ServiceAmis, choix: ChoixPartie) {
    this.textures = textures;
    this.solo = solo;
    this.profil = profil;
    this.collection = collection;
    this.amis = amis;
    this.choixMenu = () => choix;

    element<HTMLButtonElement>('menu-multijoueur').addEventListener('click', () => this.ouvrir());
    element<HTMLButtonElement>('salon-fermer').addEventListener('click', () => this.fermer());
    element('amis-code').textContent = amis.code;
    const copier = element<HTMLButtonElement>('amis-copier');
    copier.addEventListener('click', () => copierTexte(amis.code, copier));
    const saisie = element<HTMLInputElement>('amis-saisie');
    const ajouter = () => {
      const erreur = amis.ajouter(saisie.value);
      afficherStatut(element('amis-statut'), erreur ?? 'Ami ajouté : vous le verrez ici dès qu\'il sera en ligne.', erreur !== null);
      if (!erreur) saisie.value = '';
    };
    element<HTMLButtonElement>('amis-ajouter').addEventListener('click', ajouter);

    amis.surChangement = () => this.afficherAmis();
    amis.surInvitation = (ami, salon) => this.recevoirInvitation(ami, salon);
    amis.surDemande = (ami) => this.notifier(`<strong>${echapper(ami.pseudo)}</strong> voudrait jouer avec vous.`, 'Héberger', () => this.hebergerPour(ami));
    amis.surAjout = (code, pseudo) => this.notifier(`<strong>${echapper(pseudo)}</strong> vous a ajouté en ami.`, 'Ajouter aussi', () => {
      amis.ajouter(code, pseudo);
    });

    // Entrée lancerait la partie solo du menu derrière : bloquée tant que la page est ouverte ou la partie lancée
    window.addEventListener('keydown', (e) => {
      if (!(this.ouvert || this.lance) || (e.code !== 'Enter' && e.code !== 'NumpadEnter')) return;
      e.stopImmediatePropagation();
      const actif = document.activeElement?.id;
      if (actif === 'salon-code-saisie') document.getElementById('salon-entrer')?.click();
      else if (actif === 'amis-saisie') ajouter();
    }, true);
  }

  /** Résolue quand la partie à plusieurs est lancée (jamais si le joueur reste en solo). */
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
    void this.amis.demarrer().then(() => this.afficherAmis());
    this.afficherMoi();
    this.afficher();
    this.afficherAmis();
  }

  private fermer(): void {
    this.quitterSalon();
    this.ouvert = false;
    this.racine.classList.add('cache');
  }

  private quitterSalon(): void {
    this.session?.fermer();
    this.session = null;
    this.invites.clear();
    this.amis.definirEtat('menu');
  }

  // --- Partie à plusieurs ----------------------------------------------------

  /** Qui l'on est : classe et arène du menu. */
  private afficherMoi(): void {
    const { personnage, arene } = this.choixMenu();
    const texte = document.createElement('div');
    texte.append(fort(`Vous : ${this.profil.pseudo}`), ligne(`${PERSONNAGES[personnage].nom} · ${arene.nom} (choisies dans le menu)`));
    element('salon-moi').replaceChildren(this.image(personnage), texte);
  }

  /** Le contenu selon où l'on en est : accueil, salon hébergé, salon rejoint. */
  private afficher(): void {
    const s = this.session;
    if (!s) this.afficherAccueil();
    else if (s.role === 'hote') this.afficherHote(s);
    else this.afficherInvite(s);
  }

  private afficherAccueil(): void {
    const heberger = document.createElement('button');
    heberger.id = 'salon-heberger';
    heberger.className = 'salon-tuile';
    heberger.append(fort('Héberger un salon'), ligne('Vos amis entrent avec le code du salon, ou d\'un clic depuis leur liste d\'amis.'));
    heberger.addEventListener('click', () => void this.heberger());

    const rejoindre = document.createElement('div');
    rejoindre.className = 'salon-tuile';
    const saisie = document.createElement('input');
    saisie.id = 'salon-code-saisie';
    saisie.className = 'champ-mc salon-saisie';
    saisie.maxLength = 7;
    saisie.placeholder = 'ABC-DEF';
    saisie.spellcheck = false;
    saisie.autocomplete = 'off';
    const entrer = bouton('Rejoindre', () => {
      const code = normaliserCode(saisie.value, LONGUEUR_CODE_SALON);
      if (!code) {
        this.afficherStatut('Le code d\'un salon a 6 lettres ou chiffres, comme K7F-3QX', true);
        return;
      }
      saisie.value = code;
      void this.rejoindre(code);
    });
    entrer.id = 'salon-entrer';
    const rangee = document.createElement('div');
    rangee.className = 'salon-ligne';
    rangee.append(saisie, entrer);
    rejoindre.append(fort('Rejoindre un salon'), ligne('Tapez le code donné par l\'hôte.'), rangee);

    this.contenu.replaceChildren(heberger, rejoindre);
  }

  private afficherHote(s: SessionReseau): void {
    const bloc = document.createElement('div');
    bloc.className = 'salon-code-bloc';
    const code = fort(s.code);
    code.id = 'salon-code';
    code.className = 'salon-code';
    const copier = bouton('Copier', () => copierTexte(s.code, copier));
    const titre = document.createElement('div');
    titre.append(ligne('Code du salon'), code);
    bloc.append(titre, copier);

    const lancer = bouton('Lancer la partie', () => {
      s.lancer();
      this.terminer(s);
    });
    lancer.id = 'salon-lancer';
    lancer.classList.add('page-fermer');
    const actions = document.createElement('div');
    actions.className = 'salon-actions';
    actions.append(lancer, bouton('Fermer le salon', () => {
      this.quitterSalon();
      this.afficher();
      this.afficherAmis();
    }));
    this.contenu.replaceChildren(bloc, this.places(s), actions);
  }

  private afficherInvite(s: SessionReseau): void {
    const hote = s.joueurs.find((j) => j.index === 0);
    const arene = ARENES.find((a) => a.id === s.arene);
    const titre = document.createElement('div');
    titre.className = 'salon-tuile';
    titre.append(fort(hote ? `Salon de ${hote.pseudo}` : 'Entrée dans le salon…'), ligne(arene ? `Arène : ${arene.nom}` : ''));
    const attente = ligne('En attente du lancement par l\'hôte…');
    attente.className = 'salon-attente';
    const actions = document.createElement('div');
    actions.className = 'salon-actions';
    actions.append(bouton('Quitter le salon', () => {
      this.quitterSalon();
      this.afficher();
    }));
    this.contenu.replaceChildren(titre, this.places(s), attente, actions);
  }

  /** Les places du salon : les joueurs présents, puis les places libres. */
  private places(s: SessionReseau): HTMLElement {
    const grille = document.createElement('div');
    grille.id = 'salon-joueurs';
    grille.className = 'salon-places';
    for (const j of s.joueurs) grille.append(this.place(j, s));
    for (let i = s.joueurs.length; i < REGLES_MULTIJOUEUR.joueursMax; i++) {
      const libre = document.createElement('div');
      libre.className = 'salon-place libre';
      libre.textContent = 'Place libre';
      grille.append(libre);
    }
    return grille;
  }

  private place(j: InfoJoueur, s: SessionReseau): HTMLElement {
    const div = document.createElement('div');
    div.className = 'salon-place';
    const role = j.index === 0 ? ' · hôte' : '';
    const moi = j.index === s.index ? ' · vous' : '';
    const texte = document.createElement('div');
    texte.append(fort(j.pseudo), ligne(`${PERSONNAGES[j.personnage].nom}${role}${moi}`));
    div.append(this.image(j.personnage), texte);
    return div;
  }

  private async heberger(): Promise<SessionReseau | null> {
    if (this.occupe || this.session) return this.session;
    this.occupe = true;
    this.afficherStatut('Ouverture du salon…');
    try {
      const s = await SessionReseau.heberger(await this.profilReseau(), this.choixMenu().arene.id);
      if (!this.ouvert) {
        s.fermer();
        return null;
      }
      this.session = s;
      s.surChangement = () => this.surChangementSalon(s);
      this.surChangementSalon(s);
      this.afficherStatut('');
      return s;
    } catch (erreur) {
      this.afficherStatut((erreur as Error).message, true);
      return null;
    } finally {
      this.occupe = false;
    }
  }

  private async rejoindre(code: string): Promise<void> {
    if (this.occupe) return;
    if (this.session) this.quitterSalon();
    this.occupe = true;
    this.afficher();
    this.afficherStatut('Recherche du salon…');
    try {
      const s = await SessionReseau.rejoindre(await this.profilReseau(), code);
      if (!this.ouvert) {
        s.fermer();
        return;
      }
      this.session = s;
      s.surChangement = () => this.surChangementSalon(s);
      s.surLancement = () => this.terminer(s);
      s.surPerte = (raison) => {
        if (this.session !== s) return;
        this.quitterSalon();
        this.afficher();
        this.afficherStatut(raison, true);
      };
      this.surChangementSalon(s);
      this.afficherStatut('');
    } catch (erreur) {
      this.afficherStatut((erreur as Error).message, true);
    } finally {
      this.occupe = false;
    }
  }

  private surChangementSalon(s: SessionReseau): void {
    if (this.session !== s) return;
    const place = s.role === 'hote' && s.joueurs.length < REGLES_MULTIJOUEUR.joueursMax;
    this.amis.definirEtat('salon', place ? s.code : null, Math.max(1, s.joueurs.length));
    this.afficher();
    this.afficherAmis();
  }

  /** Ce que ce joueur apporte au salon : son pseudo, son skin, la classe du menu et ses talents. */
  private async profilReseau(): Promise<ProfilReseau> {
    const p = this.profil;
    const personnage = this.choixMenu().personnage;
    const skinImporte = await this.skinReseau;
    return {
      pseudo: p.pseudo, personnage, skin: skinImporte ? 'importe' : p.skin === 'importe' ? 'classe' : p.skin, brasFins: p.brasFins, skinImporte,
      bonus: this.solo.bonus(personnage),
      armure: { bonus: this.collection.bonus(), portees: this.collection.portees() },
    };
  }

  private terminer(s: SessionReseau): void {
    const arene = ARENES.find((a) => a.id === s.arene) ?? this.choixMenu().arene;
    this.ouvert = false;
    this.lance = true;
    this.racine.classList.add('cache');
    this.notifications.replaceChildren();
    element('menu').classList.add('cache');
    this.amis.definirEtat('partie', null, s.joueurs.length);
    this.surLancer?.({ choix: { personnage: this.choixMenu().personnage, arene }, session: s });
  }

  private afficherStatut(message: string, erreur = false): void {
    afficherStatut(this.statut, message, erreur);
  }

  private image(personnage: IdPersonnage): HTMLElement {
    const adresse = portrait(this.textures, PERSONNAGES[personnage]);
    const img = document.createElement('img');
    img.alt = '';
    if (adresse) img.src = adresse;
    return img;
  }

  // --- Amis ------------------------------------------------------------------

  private afficherAmis(): void {
    if (!this.ouvert) return;
    const amis = this.amis;
    if (amis.amis.length === 0) {
      this.listeAmis.replaceChildren(ligne('Aucun ami pour l\'instant : échangez une fois vos codes amis, puis vous verrez ici quand vos amis sont en ligne, et vous les rejoindrez d\'un clic.'));
      this.listeAmis.firstElementChild?.classList.add('amis-vide');
      return;
    }
    // En ligne d'abord
    const tries = [...amis.amis].sort((a, b) => Number(amis.enLigne(b.code) !== null) - Number(amis.enLigne(a.code) !== null));
    this.listeAmis.replaceChildren(...tries.map((a) => this.ligneAmi(a)));
  }

  private ligneAmi(ami: Ami): HTMLElement {
    const presence = this.amis.enLigne(ami.code);
    const div = document.createElement('div');
    div.className = 'ami';
    div.dataset['code'] = ami.code;
    const pastille = document.createElement('span');
    pastille.className = `ami-pastille ${presence?.etat ?? ''}`;
    const texte = document.createElement('div');
    texte.className = 'ami-texte';
    const etat = !presence ? 'Hors ligne'
      : presence.etat === 'salon' ? `Dans un salon (${presence.joueurs}/${REGLES_MULTIJOUEUR.joueursMax})`
      : presence.etat === 'partie' ? 'En partie' : 'En ligne';
    texte.append(fort(ami.pseudo), ligne(etat));
    div.append(pastille, texte);

    const s = this.session;
    if (presence?.salon && !s) {
      div.append(bouton('Rejoindre', () => void this.rejoindre(presence.salon!)));
    } else if (presence && presence.etat !== 'partie' && s?.role === 'hote' && !s.joueurs.some((j) => j.pseudo === ami.pseudo)) {
      const invite = this.invites.has(ami.code);
      const b = bouton(invite ? 'Invité' : 'Inviter', () => {
        this.invites.add(ami.code);
        void this.amis.inviter(ami.code, s.code);
        this.afficherAmis();
      });
      b.disabled = invite;
      div.append(b);
    } else if (presence?.etat === 'menu' && !s) {
      const propose = this.propositions.has(ami.code);
      const b = bouton(propose ? 'Proposé' : 'Proposer de jouer', () => {
        this.propositions.add(ami.code);
        void this.amis.demanderAJouer(ami.code);
        this.afficherStatut(`Proposition envoyée à ${ami.pseudo} : s'il accepte, vous rejoindrez son salon tout seul.`);
        this.afficherAmis();
      });
      b.disabled = propose;
      div.append(b);
    }
    const retirer = bouton('✕', () => this.amis.retirer(ami.code));
    retirer.classList.add('ami-retirer');
    retirer.title = 'Retirer de mes amis';
    div.append(retirer);
    return div;
  }

  /** Invitation d'un ami : acceptée d'office si on lui avait proposé de jouer, sinon une notification. */
  private recevoirInvitation(ami: Ami, salon: string): void {
    if (this.lance) return;
    if (this.propositions.delete(ami.code) && !this.session) {
      if (!this.ouvert) this.ouvrir();
      void this.rejoindre(salon);
      return;
    }
    this.notifier(`<strong>${echapper(ami.pseudo)}</strong> vous invite dans son salon.`, 'Rejoindre', () => {
      if (!this.ouvert) this.ouvrir();
      void this.rejoindre(salon);
    });
  }

  /** Un ami voudrait jouer : on héberge et on l'invite (il entrera de lui-même). */
  private hebergerPour(ami: Ami): void {
    if (!this.ouvert) this.ouvrir();
    void (this.session?.role === 'hote' ? Promise.resolve(this.session) : this.heberger()).then((s) => {
      if (!s || s.role !== 'hote') return;
      this.invites.add(ami.code);
      void this.amis.inviter(ami.code, s.code);
      this.afficherAmis();
    });
  }

  /** Notification sur le menu (jamais en partie), avec une action et « Plus tard ». */
  private notifier(html: string, action: string, faire: () => void): void {
    if (this.lance) return;
    const notif = document.createElement('div');
    notif.className = 'amis-notif';
    const texte = document.createElement('div');
    texte.innerHTML = html;
    const fermer = () => notif.remove();
    const actions = document.createElement('div');
    actions.className = 'salon-actions';
    actions.append(bouton(action, () => {
      fermer();
      faire();
    }), bouton('Plus tard', fermer));
    notif.append(texte, actions);
    this.notifications.append(notif);
    setTimeout(fermer, DUREE_NOTIFICATION * 1000);
  }
}

function fort(contenu: string): HTMLElement {
  const b = document.createElement('strong');
  b.textContent = contenu;
  return b;
}

function ligne(contenu: string): HTMLElement {
  const span = document.createElement('span');
  span.textContent = contenu;
  return span;
}

function bouton(libelle: string, action: () => void): HTMLButtonElement {
  const b = document.createElement('button');
  b.className = 'bouton-mc';
  b.textContent = libelle;
  b.addEventListener('click', action);
  return b;
}

function afficherStatut(cible: HTMLElement, message: string, erreur: boolean): void {
  cible.textContent = message;
  cible.classList.toggle('erreur', erreur);
}

function copierTexte(texte: string, b: HTMLButtonElement): void {
  void navigator.clipboard?.writeText(texte).then(() => {
    b.textContent = 'Copié !';
    setTimeout(() => { b.textContent = 'Copier'; }, 1500);
  });
}

function echapper(texte: string): string {
  return texte.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
}
