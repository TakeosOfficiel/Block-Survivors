//! La partie à plusieurs par-dessus une `Session` (src/reseau/hote.ts et client.ts), sans
//! Bevy : le jeu crée sa `Partie`, puis passe par `HoteCoop` ou `InviteCoop` à chaque pas.
//! - Salon : chacun se présente (`Bonjour`), l'hôte tient la liste et la renvoie à tous.
//! - Lancement : l'hôte fige la liste (`Lancer`), chacun prépare sa partie et répond
//!   `Pret` ; quand tous le sont (ou au bout de `ATTENTE_PRETS`), l'hôte envoie `Depart`.
//! - Partie : l'hôte simule tout le monde (entrées des invités) et envoie 15 instantanés
//!   par seconde ; les invités prédisent leur marche et affichent l'instantané. Les
//!   cartes, coffres, bilans et la fin passent en messages fiables (JSON).
//!
//! L'ordre des joueurs est celui de la liste figée : chez l'hôte, le joueur `k` de la
//! partie est le `k`-ième de la liste (0 : lui) ; chez un invité, il est le joueur 0 et
//! les autres suivent dans l'ordre de la liste.

mod hote;
mod invite;

pub use hote::HoteCoop;
pub use invite::InviteCoop;

use bs_donnees::Donnees;
use bs_protocole::coop::{InfoJoueur, MessageFiable, Presentation};
use bs_regles::Partie;

/// Places d'un salon (rangs réseau), l'hôte compris.
pub const PLACES: usize = bs_protocole::PLACES_SALON as usize;
/// Attente des `Pret` avant de partir sans les retardataires (s).
pub const ATTENTE_PRETS: f32 = 30.0;
/// Un invité silencieux depuis plus longtemps ne bouge plus (s).
pub const SILENCE_MAX: f32 = 1.0;
/// Un invité coupé depuis plus longtemps quitte la partie pour de bon (s).
pub const ABSENCE_MAX: f32 = 60.0;
/// Écart entre deux bilans envoyés à chaque invité (s).
pub const ECART_STATS: f32 = 5.0;

/// Où en est la coop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhaseCoop {
    /// Connexion au relais, ou salon pas encore ouvert/rejoint.
    Connexion,
    /// Dans le salon : liste des joueurs, en attente du lancement.
    Salon,
    /// Lancée : chacun prépare sa partie (`arene` : son identifiant).
    Chargement {
        arene: String,
    },
    Partie,
    /// Fini (refusé, exclu, salon fermé, connexion perdue) : raison lisible.
    Finie(String),
}

/// Message fiable (JSON) ? Les binaires commencent par leur type (1 ou 2).
fn fiable(octets: &[u8]) -> Option<MessageFiable> {
    (octets.first() == Some(&b'{'))
        .then(|| MessageFiable::lire(octets))
        .flatten()
}

/// Index de l'arène générée `id`.
pub fn index_arene(d: &Donnees, id: &str) -> Option<usize> {
    d.arenes.arenes.iter().position(|a| a.id == id)
}

/// Index du personnage `id` (0 si inconnu).
pub fn index_personnage(d: &Donnees, id: &str) -> usize {
    d.personnages.personnages.get_index_of(id).unwrap_or(0)
}

/// Ajoute à `p` un combattant pour ce joueur, avec ses talents ; son rang `k`.
fn ajouter_joueur(p: &mut Partie, presentation: &Presentation) -> usize {
    let d = p.d;
    let k = p.ajouter_combattant(index_personnage(d, &presentation.personnage));
    let bonus = presentation.bonus(d);
    p.avec_combattant(k, |p| p.appliquer_talents(&bonus));
    k
}

/// Le joueur de la place `place` dans la liste figée.
fn par_place(joueurs: &[InfoJoueur], place: u8) -> Option<usize> {
    joueurs.iter().position(|j| j.index == place)
}
