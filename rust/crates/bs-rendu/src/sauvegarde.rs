//! Profil du joueur pendant la partie : profils et progression lus au lancement (dossier
//! du système, ou `BS_DOSSIER`), récompense de maîtrise enregistrée quand l'écran de fin
//! s'ouvre (parties jouées seulement : la démonstration ne compte pas). Une partie gagnée
//! puis continuée compte une seconde fois à sa fin, en mode sans fin, pour la suite seule.
//!
//! `BS_IMPORTER=<fichier.bssave>` importe un profil exporté (par exemple par la version
//! TypeScript) : il devient le profil actif. `BS_EXPORTER=<fichier.bssave>` exporte le
//! profil actif (lisible par la version TypeScript). Le menu Profils fait de même.
//!
//! `BS_SKIN=<fichier.png>` importe une skin dans le profil (ramenée au format du jeu,
//! voir `skins`) et la choisit ; `BS_PSEUDO=<pseudo>` change le pseudo.
//!
//! La Collection (armures, objets découverts) donne ses bonus au départ de chaque partie
//! et retient aussitôt les pièces gagnées sur les boss et les objets pris.

use bevy::prelude::*;
use bs_donnees::arenes::Monde;
use bs_donnees::equipement::Piece;
use bs_donnees::progression::BonusTalents;
use bs_donnees::{Apparence, Donnees};
use bs_regles::evenements::Evenement;
use bs_regles::partie::Partie;
use bs_sauvegarde::collection::MONDES;
use bs_sauvegarde::profil_joueur::{CHEMIN_SKIN_IMPORTE, ChoixSkin, nettoyer_pseudo};
use bs_sauvegarde::{
    Amis, Bilan, Collection, ProfilJoueur, Profils, Progression, Recompense, Stockage,
};

use crate::PartieEnCours;

/// Ce que le profil apporte à chaque partie : talents (et bonus d'armure portée), pièces
/// d'armure déjà possédées (jamais retirées sur un boss), apparence du personnage joué
/// (`None` : celle de la classe).
#[derive(Debug, Clone, Default)]
pub struct Depart {
    pub talents: BonusTalents,
    pub armures: Vec<(Monde, Piece)>,
    pub apparence: Option<&'static Apparence>,
}

impl Depart {
    pub fn appliquer(&self, partie: &mut Partie) {
        partie.appliquer_talents(&self.talents);
        let monde = partie.etat.arene.monde;
        let pieces: Vec<Piece> = self
            .armures
            .iter()
            .filter(|(m, _)| *m == monde)
            .map(|(_, p)| *p)
            .collect();
        partie.posseder(&pieces);
    }
}

#[derive(Resource)]
pub struct Sauvegarde {
    pub progression: Progression,
    pub collection: Collection,
    pub profil: ProfilJoueur,
    /// Profils du dossier (profil actif, clés de ses données).
    pub profils: Profils,
    /// Code ami et amis (réseau : étape 6).
    #[allow(dead_code)] // lu par la page Multijoueur (étapes 5 et 6)
    pub amis: Amis,
    /// Récompense de la partie qui vient de finir (écran de fin).
    pub recompense: Option<Recompense>,
    /// XP totale et boss vaincus au passage en mode sans fin.
    depart_sans_fin: (f32, u32),
}

impl Sauvegarde {
    pub fn charger(d: &'static Donnees) -> Self {
        let mut profils = Profils::charger(Stockage::du_systeme());
        if let Some(chemin) = std::env::var_os("BS_IMPORTER") {
            match std::fs::read_to_string(&chemin) {
                Ok(contenu) => match profils.importer(&contenu) {
                    Ok(id) => info!("profil {id} importé de {}", chemin.display()),
                    Err(e) => warn!("{} : {}", chemin.display(), e.message()),
                },
                Err(e) => warn!("{} : {e}", chemin.display()),
            }
        }
        info!(
            "profil {} ({})",
            profils.actif(),
            profils.stockage().dossier().display()
        );
        if let Some(chemin) = std::env::var_os("BS_EXPORTER") {
            let fichier = profils.exporter(bs_commun::VERSION, &date_iso());
            match std::fs::write(&chemin, fichier) {
                Ok(()) => info!(
                    "profil {} exporté dans {}",
                    profils.actif(),
                    chemin.display()
                ),
                Err(e) => warn!("{} : {e}", chemin.display()),
            }
        }
        let mut s = Self::du_profil(d, profils);
        changer_profil(&mut s.profil);
        s
    }

    /// Données du profil actif de `profils`.
    fn du_profil(d: &'static Donnees, profils: Profils) -> Self {
        let cle = profils.cle("progression");
        let progression = Progression::charger(d, profils.stockage().clone(), cle);
        let cle = profils.cle("collection");
        let collection = Collection::charger(profils.stockage().clone(), cle);
        let profil = ProfilJoueur::charger(profils.stockage().clone(), profils.cle("profil"));
        let amis = Amis::charger(profils.stockage().clone(), profils.cle("amis"));
        info!("code ami : {} ({} amis)", amis.code, amis.amis.len());
        Self {
            progression,
            collection,
            profil,
            profils,
            amis,
            recompense: None,
            depart_sans_fin: (0.0, 0),
        }
    }

    /// Relit tout depuis le profil devenu actif (changé, créé, importé, supprimé).
    pub fn recharger(&mut self, d: &'static Donnees) {
        *self = Self::du_profil(d, self.profils.clone());
        info!("profil {} chargé", self.profils.actif());
    }

    /// Exporte le profil actif en `.bssave` (dans Téléchargements s'il existe, sinon
    /// dans le dossier des sauvegardes) ; renvoie le fichier écrit.
    pub fn exporter(&self) -> Result<std::path::PathBuf, String> {
        let date = date_iso();
        let fichier = self.profils.exporter(bs_commun::VERSION, &date);
        let pseudo: String = self
            .profil
            .pseudo
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        let nom = format!("block-survivors-{pseudo}-{}.bssave", &date[..10]);
        let maison = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" });
        let telechargements = maison
            .map(|m| std::path::PathBuf::from(m).join("Downloads"))
            .filter(|t| t.is_dir() && std::env::var_os("BS_DOSSIER").is_none());
        let dossier = telechargements.unwrap_or_else(|| self.profils.stockage().dossier().into());
        let chemin = dossier.join(nom);
        std::fs::write(&chemin, fichier).map_err(|e| e.to_string())?;
        Ok(chemin)
    }

    /// Départ d'une partie du personnage d'index `personnage` : arbre de talents actif,
    /// bonus de l'armure portée, pièces possédées.
    pub fn depart(&self, d: &Donnees, personnage: usize) -> Depart {
        let mut talents: BonusTalents = d
            .personnages
            .personnages
            .get_index(personnage)
            .map(|(id, _)| self.progression.bonus(id))
            .unwrap_or_default();
        talents.modif.cumuler(&self.collection.bonus(d));
        let mut armures = Vec::new();
        for monde in MONDES {
            for piece in self.collection.pieces_du_monde(monde) {
                armures.push((monde, piece));
            }
        }
        // Skin du profil : même classe, autre modèle et autre skin
        let apparence = self.profil.apparence().map(|a| {
            let classe = &d.personnages.personnages[personnage].apparence;
            let a = Apparence {
                modele: a.modele.to_owned(),
                skin: a.skin,
                ..classe.clone()
            };
            &*Box::leak(Box::new(a))
        });
        Depart {
            talents,
            armures,
            apparence,
        }
    }

    /// Image de la skin importée, si le profil la porte.
    pub fn skin_importee(&self) -> Option<Image> {
        if self.profil.skin != ChoixSkin::Importe {
            return None;
        }
        let image = crate::skins::image_skin(self.profil.skin_importe.as_deref()?);
        if image.is_none() {
            warn!("skin importée illisible : skin de la classe");
        }
        image
    }

    /// Enregistre la partie finie ; sa récompense est gardée pour l'écran de fin.
    pub fn terminer(&mut self, p: &Partie) {
        let e = &p.etat;
        let (personnage, _) =
            p.d.personnages
                .personnages
                .get_index(e.index_personnage)
                .expect("personnage de la partie");
        let s = &e.stats;
        let (xp, boss) = if e.sans_fin {
            let (xp, boss) = self.depart_sans_fin;
            (s.xp_totale - xp, s.boss_vaincus.saturating_sub(boss))
        } else {
            (s.xp_totale, s.boss_vaincus)
        };
        let bilan = Bilan {
            personnage,
            arene: &e.arene.id,
            temps: e.temps_jeu,
            xp: f64::from(xp),
            boss,
            sans_fin: e.sans_fin,
            en_equipe: false,
        };
        self.recompense = Some(self.progression.terminer_partie(&bilan));
    }

    /// La partie gagnée continue en mode sans fin : la suite comptera à part.
    pub fn continuer_sans_fin(&mut self, p: &Partie) {
        let s = &p.etat.stats;
        self.depart_sans_fin = (s.xp_totale, s.boss_vaincus);
        self.recompense = None;
    }
}

/// Après chaque pas : pièces gagnées sur un boss et objets découverts entrent dans la
/// Collection (enregistrée aussitôt) ; les pièces comptent dès la partie suivante.
pub fn noter_collection(mut jeu: ResMut<PartieEnCours>, mut sauvegarde: ResMut<Sauvegarde>) {
    let jeu = &mut *jeu;
    for ev in &jeu.partie.ev.liste {
        match *ev {
            Evenement::ButinBoss { monde, pieces } => {
                for piece in pieces.into_iter().flatten() {
                    sauvegarde.collection.ajouter(monde, piece);
                    jeu.depart.armures.push((monde, piece));
                }
            }
            Evenement::Decouverte(objet) => {
                if let Some((id, _)) = jeu.partie.d.equipement.objets.get_index(objet) {
                    sauvegarde.collection.decouvrir(id);
                }
            }
            _ => {}
        }
    }
}

/// `BS_SKIN` et `BS_PSEUDO` (essais ; la page Profil fait de même à la souris).
fn changer_profil(profil: &mut ProfilJoueur) {
    let mut change = false;
    if let Ok(pseudo) = std::env::var("BS_PSEUDO") {
        profil.pseudo = nettoyer_pseudo(&pseudo);
        change = true;
    }
    if let Some(chemin) = std::env::var_os("BS_SKIN") {
        let lu = std::fs::read(&chemin).map_err(|e| e.to_string());
        match lu.and_then(|o| crate::skins::normaliser(&o)) {
            Ok(s) => {
                info!(
                    "skin {} importée ({} px{})",
                    chemin.display(),
                    s.taille,
                    if s.reduite { ", réduite" } else { "" }
                );
                profil.skin_importe = Some(s.adresse);
                profil.skin = ChoixSkin::Importe;
                change = true;
            }
            Err(e) => warn!("{} : {e}", chemin.display()),
        }
    }
    if change {
        profil.enregistrer();
    }
}

/// Skin importée du profil, donnée aux textures sous son chemin (avant les demandes).
pub fn inserer_skin(
    sauvegarde: Res<Sauvegarde>,
    mut textures: ResMut<crate::textures::Textures>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Some(image) = sauvegarde.skin_importee() {
        textures.inserer(CHEMIN_SKIN_IMPORTE, images.add(image));
    }
}

/// Date et heure UTC au format ISO 8601 (« 2026-11-01T20:14:10Z »), comme dans les
/// fichiers exportés par le jeu TS.
fn date_iso() -> String {
    let s = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (jours, reste) = ((s / 86_400) as i64, s % 86_400);
    // Jours depuis 1970 → date civile (algorithme de H. Hinnant)
    let z = jours + 719_468;
    let ere = z.div_euclid(146_097);
    let j = z - ere * 146_097;
    let a = (j - j / 1460 + j / 36_524 - j / 146_096) / 365;
    let jour_an = j - (365 * a + a / 4 - a / 100);
    let m = (5 * jour_an + 2) / 153;
    let jour = jour_an - (153 * m + 2) / 5 + 1;
    let mois = if m < 10 { m + 3 } else { m - 9 };
    let annee = a + ere * 400 + i64::from(mois <= 2);
    let (h, mi, se) = (reste / 3600, reste / 60 % 60, reste % 60);
    format!("{annee:04}-{mois:02}-{jour:02}T{h:02}:{mi:02}:{se:02}Z")
}
