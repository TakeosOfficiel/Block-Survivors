//! Profil du joueur pendant la partie : profils et progression lus au lancement (dossier
//! du système, ou `BS_DOSSIER`), récompense de maîtrise enregistrée quand l'écran de fin
//! s'ouvre (parties jouées seulement : la démonstration ne compte pas). Une partie gagnée
//! puis continuée compte une seconde fois à sa fin, en mode sans fin, pour la suite seule.
//!
//! `BS_IMPORTER=<fichier.bssave>` importe un profil exporté (par exemple par la version
//! TypeScript) : il devient le profil actif. `BS_EXPORTER=<fichier.bssave>` exporte le
//! profil actif (lisible par la version TypeScript). Les menus des profils viendront à
//! l'étape 5.

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_regles::partie::Partie;
use bs_sauvegarde::{Bilan, Profils, Progression, Recompense, Stockage};

#[derive(Resource)]
pub struct Sauvegarde {
    pub progression: Progression,
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
        let cle = profils.cle("progression");
        let progression = Progression::charger(d, profils.stockage().clone(), cle);
        Self {
            progression,
            recompense: None,
            depart_sans_fin: (0.0, 0),
        }
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
