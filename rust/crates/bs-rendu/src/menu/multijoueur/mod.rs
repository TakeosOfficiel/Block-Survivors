//! Page Multijoueur du menu (src/ui/ecranMultijoueur.ts) : créer un salon (public ou
//! privé), en rejoindre un par son code ou dans la liste des salons publics, puis, dans le
//! salon, voir qui est là ; l'hôte lance la partie sur l'arène choisie (générée), l'invité
//! la suit. Hors partie, `suivre` relève la session à chaque image et refait la page quand
//! le salon change ; une coop finie laisse sa raison dans `MessageCoop`. À droite, les
//! amis (`amis`) : code ami, ajout, présence, invitations et demandes.

mod amis;
mod vue;
mod vue_amis;

pub use vue::{VueMulti, construire};

use bevy::prelude::*;
use bs_reseau::{EvenementSession, PhaseCoop, SalonPublic, Session, code_saisi};

use super::{Menu, Page};
use crate::Phase;
use crate::amis::ReseauAmis;
use crate::coop::{Coop, MessageCoop, Role};
use crate::lancement::{self, ChoixPartie, DonneesJeu, PartiePreparee, SourceMenu};
use crate::sauvegarde::Sauvegarde;

/// Bouton de la page.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionMulti {
    /// Choisit la visibilité du salon à créer (public : vrai).
    Visibilite(bool),
    /// Crée le salon, avec la visibilité choisie.
    Creer,
    /// Déplie ou replie les succès de coop.
    Succes,
    /// Ouvre ou ferme le volet des amis.
    Amis,
    /// Commence la saisie d'un code.
    Code,
    /// Rejoint le salon du code saisi.
    Rejoindre,
    /// Rejoint le salon public de ce rang dans la liste.
    Public(usize),
    Actualiser,
    Lancer,
    Quitter,
    /// Commence la saisie d'un code ami.
    CodeAmi,
    /// Ajoute l'ami du code saisi.
    AjouterAmi,
    /// Rejoint le salon de l'ami de ce rang.
    RejoindreAmi(usize),
    /// Invite l'ami de ce rang (ouvre un salon privé s'il le faut).
    Inviter(usize),
    /// Demande à l'ami de ce rang de nous inviter.
    Demander(usize),
    Retirer(usize),
    /// Accepte le message reçu de ce rang (rejoindre, inviter, ajouter).
    Accepter(usize),
    /// Oublie le message reçu de ce rang.
    Ignorer(usize),
}

/// Liste des salons publics, demandée au relais par une session à part.
#[derive(Resource, Default)]
pub struct Annuaire {
    session: Option<Session>,
    salons: Vec<SalonPublic>,
    /// La saisie en cours est un code ami (sinon un code de salon).
    champ_ami: bool,
    /// Le salon à créer sera public.
    public: bool,
    /// Les succès de coop sont dépliés.
    succes_ouverts: bool,
    /// Le volet des amis est ouvert.
    amis_ouverts: bool,
}

impl Annuaire {
    pub fn actualiser(&mut self) {
        self.session = Some(Session::ouvrir(&bs_reseau::adresse_relais()));
    }

    /// Relève la session ; vrai si la liste a changé (ou la recherche a pris fin).
    fn maj(&mut self) -> bool {
        let Some(s) = &mut self.session else {
            return false;
        };
        let mut fini = false;
        while let Some(ev) = s.maj() {
            match ev {
                EvenementSession::Connectee => s.lister(),
                EvenementSession::Liste(l) => {
                    self.salons = l;
                    fini = true;
                }
                EvenementSession::Perdue(_) | EvenementSession::Refus(_) => fini = true,
                _ => {}
            }
        }
        if fini && let Some(mut s) = self.session.take() {
            s.quitter();
        }
        fini
    }
}
/// Ce qu'il faut pour agir sur la page.
pub struct Contexte<'a, 'w, 's> {
    pub commandes: &'a mut Commands<'w, 's>,
    pub dj: &'a DonneesJeu,
    pub sauvegarde: &'a mut Sauvegarde,
    pub coop: Option<&'a mut Coop>,
    pub amis: Option<&'a mut ReseauAmis>,
    pub message: &'a mut MessageCoop,
    pub annuaire: &'a mut Annuaire,
    pub phase: &'a mut NextState<Phase>,
}

/// Ouvre ou rejoint un salon : la présentation part du choix courant.
fn entrer_salon(c: &mut Contexte, choix: ChoixPartie, code: Option<[u8; 6]>, public: bool) {
    let moi = crate::coop::presentation(c.dj.d, c.sauvegarde, choix.personnage);
    c.message.0 = None;
    let coop = match code {
        Some(code) => Coop::invite(moi, code),
        None => Coop::hote(moi, public),
    };
    c.commandes.insert_resource(coop);
}

/// Applique une action de la page.
fn agir(c: &mut Contexte, menu: &mut Menu, action: ActionMulti) {
    let choix = menu.choix;
    match action {
        ActionMulti::Visibilite(public) => c.annuaire.public = public,
        ActionMulti::Succes => c.annuaire.succes_ouverts ^= true,
        ActionMulti::Amis => c.annuaire.amis_ouverts ^= true,
        ActionMulti::Creer if c.coop.is_none() => {
            let public = c.annuaire.public;
            entrer_salon(c, choix, None, public);
        }
        ActionMulti::Code if c.coop.is_none() => {
            c.annuaire.champ_ami = false;
            menu.saisie = Some(String::new());
        }
        ActionMulti::CodeAmi => {
            c.annuaire.champ_ami = true;
            menu.saisie = Some(String::new());
        }
        ActionMulti::Rejoindre | ActionMulti::AjouterAmi => {
            c.annuaire.champ_ami = action == ActionMulti::AjouterAmi;
            let saisie = menu.saisie.take().unwrap_or_default();
            valider(c, menu, &saisie);
        }
        ActionMulti::Public(k) if c.coop.is_none() => {
            if let Some(s) = c.annuaire.salons.get(k) {
                let code = s.code;
                entrer_salon(c, choix, Some(code), false);
            }
        }
        ActionMulti::Actualiser => c.annuaire.actualiser(),
        ActionMulti::Lancer => lancer(c, choix),
        ActionMulti::Quitter => {
            if let Some(coop) = c.coop.as_deref_mut() {
                coop.quitter();
            }
            c.commandes.remove_resource::<Coop>();
            c.message.0 = None;
        }
        a => amis::agir(c, menu, a),
    }
}

/// Code validé par Entrée, puis bouton de la page ; vrai si le salon a été quitté.
pub fn traiter(
    c: &mut Contexte,
    menu: &mut Menu,
    code: Option<&str>,
    action: Option<ActionMulti>,
) -> bool {
    if let Some(code) = code {
        valider(c, menu, code);
    }
    if let Some(a) = action {
        agir(c, menu, a);
    }
    action == Some(ActionMulti::Quitter)
}

/// Code validé (bouton ou Entrée).
fn valider(c: &mut Contexte, menu: &Menu, saisie: &str) {
    if c.annuaire.champ_ami {
        amis::ajouter(c, saisie);
        return;
    }
    if c.coop.is_some() {
        return;
    }
    match code_saisi(saisie) {
        Some(code) => entrer_salon(c, menu.choix, Some(code), false),
        None => c.message.0 = Some("Code invalide : 6 lettres, comme ABC-DEF".into()),
    }
}

/// Hôte : lance la partie sur l'arène choisie (générée seulement).
fn lancer(c: &mut Contexte, choix: ChoixPartie) {
    let Some(Role::Hote(h)) = c.coop.as_deref_mut().map(|c| &mut c.role) else {
        return;
    };
    let arene = &c.dj.arenes[choix.arene];
    if !matches!(arene.source, SourceMenu::Generee(_)) {
        c.message.0 = Some("En coop, choisissez une arène générée.".into());
        return;
    }
    let graine = lancement::graine_au_hasard();
    match PartiePreparee::nouvelle(c.dj, c.sauvegarde, choix, graine) {
        Ok(p) => {
            info!("partie en coop lancée (graine {graine})");
            h.lancer(&arene.def.id);
            p.inserer(c.commandes);
            c.phase.set(Phase::Chargement);
        }
        Err(e) => c.message.0 = Some(format!("Partie impossible : {e}")),
    }
}

/// Signature du salon : change quand la page est à refaire.
fn signature(coop: Option<&Coop>) -> u64 {
    let Some(coop) = coop else {
        return 0;
    };
    let phase = match coop.phase() {
        PhaseCoop::Connexion => 1,
        PhaseCoop::Salon => 2,
        PhaseCoop::Chargement { .. } => 3,
        PhaseCoop::Partie => 4,
        PhaseCoop::Finie(_) => 5,
    };
    coop.joueurs()
        .iter()
        .fold(phase, |s, j| s * 31 + u64::from(j.index) + 1)
}

/// À chaque image du menu : relève le salon et la liste, suit le lancement de l'hôte.
#[allow(clippy::too_many_arguments)]
pub fn suivre(
    mut commandes: Commands,
    coop: Option<ResMut<Coop>>,
    mut menu: ResMut<Menu>,
    mut message: ResMut<MessageCoop>,
    mut annuaire: ResMut<Annuaire>,
    (dj, sauvegarde): (Res<DonneesJeu>, Res<Sauvegarde>),
    mut phase: ResMut<NextState<Phase>>,
    (amis, mut vue): (Option<Res<ReseauAmis>>, Local<u64>),
) {
    if annuaire.maj() && menu.page == Page::Multijoueur {
        menu.a_refaire = true;
    }
    let mut coop = coop;
    let mut retirer = false;
    if let Some(c) = coop.as_deref_mut() {
        c.maj_salon();
        if let Some(r) = c.finie() {
            message.0 = Some(r.to_owned());
            retirer = true;
        } else if let Role::Invite(i) = &mut c.role
            && i.recepteur.is_none()
            && let Some(index) = i.arene(dj.d)
        {
            // L'hôte a lancé : ma partie sur son arène, avec mon personnage
            let arene = dj
                .arenes
                .iter()
                .position(|a| matches!(a.source, SourceMenu::Generee(k) if k == index));
            let choix = arene.map(|arene| ChoixPartie {
                arene,
                ..menu.choix
            });
            let graine = lancement::graine_au_hasard();
            match choix.map(|choix| PartiePreparee::nouvelle(&dj, &sauvegarde, choix, graine)) {
                Some(Ok(PartiePreparee::Generee(mut p))) => {
                    i.preparer(&mut p.0.partie);
                    PartiePreparee::Generee(p).inserer(&mut commandes);
                    phase.set(Phase::Chargement);
                }
                _ => {
                    message.0 = Some("Arène de l'hôte inconnue".into());
                    retirer = true;
                }
            }
        }
        if retirer {
            c.quitter();
            commandes.remove_resource::<Coop>();
        }
    }
    if retirer {
        coop = None;
    }
    // Amis : présences, messages (et leur arrivée)
    let version = amis.map_or(0, |a| a.version + 1);
    let s = signature(coop.as_deref())
        .wrapping_mul(1_000_003)
        .wrapping_add(version);
    if s != *vue {
        *vue = s;
        if menu.page == Page::Multijoueur {
            menu.a_refaire = true;
        }
    }
    if message.is_changed() && menu.page == Page::Multijoueur {
        menu.a_refaire = true;
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_lisibles_et_saisis() {
        let code = *b"ABCDEF";
        assert_eq!(bs_reseau::code_lisible(&code), "ABC-DEF");
        assert_eq!(code_saisi("abc-def"), Some(code));
        assert_eq!(code_saisi("abc"), None);
    }
}
