//! Côté invité (src/reseau/client.ts) : applique les instantanés de l'hôte à sa propre
//! `Partie` (qui sert telle quelle à l'affichage) et les adoucit d'un pas à l'autre.
//! L'invité est le joueur 0 de sa partie ; il prédit sa marche et la corrige d'après la
//! position que l'hôte renvoie pour sa dernière entrée prise en compte. Les autres
//! joueurs, les ennemis et les orbes glissent vers leur dernière position reçue.

use bs_regles::Partie;
use bs_regles::etat::Issue;
use bs_regles::evenements::Evenement;
use bs_regles::ramassage::{Coffre, Objet, Orbe};

use super::instantane::*;
use super::octets::Lecteur;

/// Entrées dont on garde la position prédite.
const HISTORIQUE: usize = 128;
/// Part de l'écart corrigée à chaque instantané ; au-delà de `TELEPORTATION` blocs, d'un coup.
const CORRECTION: f32 = 0.35;
const TELEPORTATION: f32 = 3.0;
/// Rapidité du glissement vers la position reçue (1/s).
const LISSAGE: f32 = 14.0;
/// Places dans un salon (rangs réseau).
const RANGS: usize = 8;

pub struct Recepteur {
    /// Rang réseau → joueur de la partie locale (0 : moi).
    pub joueurs: [Option<usize>; RANGS],
    dernier_numero: u32,
    historique: [(u32, f32, f32); HISTORIQUE],
    /// Position reçue des joueurs (index : joueur local) et des ennemis (index du pool).
    cibles_joueurs: Vec<(f32, f32, f32)>,
    cibles: Vec<(f32, f32, f32)>,
    anciens: Vec<(u32, f32, f32, f32)>,
    /// Un instantané a été reçu depuis le début.
    pub recu: bool,
}

impl Recepteur {
    /// `joueurs` : (rang réseau, joueur local) de chacun.
    pub fn nouveau(joueurs: &[(u8, usize)]) -> Self {
        let mut table = [None; RANGS];
        for &(rang, k) in joueurs {
            if let Some(t) = table.get_mut(rang as usize) {
                *t = Some(k);
            }
        }
        Self {
            joueurs: table,
            dernier_numero: 0,
            historique: [(u32::MAX, 0.0, 0.0); HISTORIQUE],
            cibles_joueurs: vec![(0.0, 0.0, 0.0); RANGS],
            cibles: Vec::with_capacity(bs_regles::ennemis::CAPACITE),
            anciens: Vec::with_capacity(bs_regles::ennemis::CAPACITE),
            recu: false,
        }
    }

    /// Note la position prédite après l'entrée `numero`.
    pub fn noter_entree(&mut self, numero: u32, x: f32, z: f32) {
        self.historique[numero as usize % HISTORIQUE] = (numero, x, z);
    }

    /// Glisse les autres joueurs et les ennemis vers leur position reçue (chaque pas).
    pub fn interpoler(&mut self, p: &mut Partie, dt: f32) {
        let k = 1.0 - libm::expf(-LISSAGE * dt);
        for j in 1..p.nombre_joueurs() {
            let (cx, cz, ca) = self.cibles_joueurs[j];
            let e = p.etat_joueur_mut(j);
            let (dx, dz) = ((cx - e.x) * k, (cz - e.z) * k);
            e.vx = dx / dt.max(1e-4);
            e.vz = dz / dt.max(1e-4);
            e.x += dx;
            e.z += dz;
            e.angle += bs_commun::maths::ecart_angle(e.angle, ca) * k;
        }
        for (i, e) in p.ennemis.pool[..p.ennemis.nombre].iter_mut().enumerate() {
            let Some(&(cx, cz, ca)) = self.cibles.get(i) else {
                break;
            };
            e.x += (cx - e.x) * k;
            e.z += (cz - e.z) * k;
            e.angle += bs_commun::maths::ecart_angle(e.angle, ca) * k;
        }
    }

    /// Applique un instantané ; faux s'il est périmé ou illisible.
    pub fn appliquer(&mut self, p: &mut Partie, octets: &[u8]) -> bool {
        let mut l = Lecteur::nouveau(octets);
        if l.u8() != TYPE_INSTANTANE {
            return false;
        }
        let numero = l.u32();
        if self.recu && numero <= self.dernier_numero {
            return false;
        }
        self.dernier_numero = numero;
        let premier = !std::mem::replace(&mut self.recu, true);
        let d = p.d;
        let e = &mut p.etat;
        e.temps_jeu = l.f64();
        e.ennemis_tues = l.u32();
        let niveau = u32::from(l.u16());
        let (xp, xp_niveau) = (l.f32(), l.f32());
        e.emeraudes = l.u32();
        e.issue = match l.u8() {
            1 => Some(Issue::Victoire),
            2 => Some(Issue::Defaite),
            _ => None,
        };
        let drapeaux = l.u8();
        e.sans_fin = drapeaux & 1 != 0;
        p.prolongation = drapeaux & 2 != 0;
        // Niveau commun : chaque niveau gagné, des cartes à choisir
        let j = &mut e.joueur;
        e.niveaux_en_attente += niveau.saturating_sub(j.niveau);
        e.stats.xp_totale += (niveau.saturating_sub(j.niveau)) as f32 * j.xp_niveau;
        j.niveau = niveau.max(1);
        j.xp = xp;
        j.xp_niveau = xp_niveau;

        // Équipe
        for _ in 0..l.u8() {
            let rang = l.u8() as usize;
            let derniere_entree = l.u32();
            let (x, z, angle) = (l.f32(), l.f32(), l.angle());
            let (pv, pv_max, bouclier) = (l.f32(), l.f32(), l.f32());
            let drapeaux = l.u8();
            let (temps_mort, invulnerabilite) = (l.f32(), l.f32());
            let commandes = commandes(l.u8(), l.f32());
            let _reanimation = l.part();
            let vies = u32::from(l.u8());
            let Some(k) = self.joueurs.get(rang).copied().flatten() else {
                continue;
            };
            if k >= p.nombre_joueurs() {
                continue;
            }
            if k > 0 {
                let m = &mut p.equipe.membres[k - 1];
                m.en_choix = drapeaux & JOUEUR_EN_CHOIX != 0;
                m.parti = drapeaux & JOUEUR_PARTI != 0;
                m.joueur.commandes = commandes;
                self.cibles_joueurs[k] = (x, z, angle);
            }
            let j = p.etat_joueur_mut(k);
            if k == 0 && !premier {
                self.corriger(j, derniere_entree, x, z);
            } else if premier || (j.x - x).abs() + (j.z - z).abs() > 2.0 * TELEPORTATION {
                j.x = x;
                j.z = z;
            }
            j.pv = pv;
            j.pv_max = pv_max;
            j.bouclier = bouclier;
            j.mort = drapeaux & JOUEUR_MORT != 0;
            j.protection = if drapeaux & JOUEUR_PROTEGE != 0 {
                0.1
            } else {
                0.0
            };
            j.temps_mort = temps_mort;
            j.invulnerabilite = invulnerabilite;
            j.vies_utilisees = vies;
        }

        // Ennemis : reprennent leur position d'avant (même identifiant), puis glissent
        self.anciens.clear();
        for en in p.ennemis.actifs() {
            self.anciens.push((en.id, en.x, en.z, en.angle));
        }
        self.anciens.sort_unstable_by_key(|a| a.0);
        self.cibles.clear();
        let n = (l.u16() as usize).min(p.ennemis.pool.len());
        for i in 0..n {
            let id = l.u32();
            let index_type = l.u16() as usize;
            let (x, z, angle) = (l.coordonnee(), l.coordonnee(), l.angle());
            let (pv, pv_max) = (l.f32(), l.f32());
            let drapeaux = l.u8();
            let mort = l.part();
            let vitesse = f32::from(l.u8()) / 16.0;
            let meche = l.part();
            let Some(def) = d.ennemis.get_index(index_type).map(|(_, v)| v) else {
                l.erreur = true;
                break;
            };
            let avant = self.anciens.binary_search_by_key(&id, |a| a.0).ok();
            let en = &mut p.ennemis.pool[i];
            en.id = id;
            en.index_type = index_type;
            en.def = def;
            (en.x, en.z, en.angle) = avant.map_or((x, z, angle), |a| {
                let a = self.anciens[a];
                (a.1, a.2, a.3)
            });
            en.vx = 0.0;
            en.vz = 0.0;
            en.pv = pv;
            en.pv_max = pv_max;
            en.mort = if drapeaux & ENNEMI_MORT != 0 {
                mort * bs_regles::ennemis::DUREE_MORT
            } else {
                -1.0
            };
            en.blessure = if drapeaux & ENNEMI_BLESSE != 0 {
                0.2
            } else {
                0.0
            };
            en.ralenti = if drapeaux & ENNEMI_RALENTI != 0 {
                0.3
            } else {
                0.0
            };
            en.vitesse_sol = vitesse;
            en.meche = meche;
            en.brulure = Default::default();
            en.poison = Default::default();
            en.saignement = Default::default();
            self.cibles.push((x, z, angle));
        }
        p.ennemis.nombre = self.cibles.len();

        // Butin
        let r = &mut p.ramassables;
        r.orbes.clear();
        for i in 0..l.u16() {
            let (x, z, valeur) = (l.coordonnee(), l.coordonnee(), l.f32());
            if r.orbes.len() < r.orbes.capacity() {
                let phase = f32::from(i) * 1.7;
                r.orbes.push(Orbe {
                    x,
                    z,
                    vitesse: 0.0,
                    aimantee: false,
                    valeur,
                    phase,
                });
            }
        }
        r.objets.clear();
        for i in 0..l.u16() {
            let index_type = l.u16() as usize;
            let (x, z, valeur) = (l.coordonnee(), l.coordonnee(), l.f32());
            if r.objets.len() < r.objets.capacity() && index_type < d.objets.len() {
                r.objets.push(Objet {
                    index_type,
                    x,
                    z,
                    phase: f32::from(i) * 2.3,
                    valeur,
                    aimante: false,
                    vitesse: 0.0,
                });
            }
        }
        r.coffres.pool.clear();
        for _ in 0..l.u16() {
            let index_type = l.u16() as usize;
            let (x, z, angle) = (l.coordonnee(), l.coordonnee(), l.angle());
            let ouvert = l.f32();
            if r.coffres.pool.len() < r.coffres.pool.capacity()
                && index_type < d.coffres.coffres.len()
            {
                r.coffres.pool.push(Coffre {
                    index_type,
                    x,
                    z,
                    angle,
                    ouvert: (ouvert >= 0.0).then_some(ouvert),
                });
            }
        }

        // Projectiles ennemis (ils avancent ici jusqu'au suivant)
        let pe = &mut p.projectiles_ennemis;
        pe.pool.clear();
        for _ in 0..l.u16() {
            let index_type = l.u16() as usize;
            let (x, z) = (l.coordonnee(), l.coordonnee());
            let (vx, vz, vie) = (l.f32(), l.f32(), l.f32());
            let renvoye = l.u8() != 0;
            let Some(def) = d.projectiles.get_index(index_type).map(|(_, v)| v) else {
                continue;
            };
            if pe.pool.len() < pe.pool.capacity() {
                pe.pool
                    .push(bs_regles::projectiles_ennemis::ProjectileEnnemi {
                        index_type,
                        def,
                        x,
                        z,
                        vx,
                        vz,
                        vie,
                        degats: 0.0,
                        boss: false,
                        renvoye,
                        arme: None,
                    });
            }
        }

        // Chiffres de dégâts
        for _ in 0..l.u16() {
            let (x, y, z, valeur) = (l.coordonnee(), l.f32(), l.coordonnee(), l.f32());
            let style = STYLES[(l.u8() as usize).min(STYLES.len() - 1)];
            p.ev.pousser(Evenement::Degats {
                x,
                y,
                z,
                valeur,
                style,
            });
        }
        p.ennemis.reconstruire_grille();
        !l.erreur
    }

    /// Corrige la marche prédite d'après la position de l'hôte pour l'entrée `numero`.
    /// Les positions prédites gardées sont décalées d'autant : sinon les instantanés
    /// suivants (avant que l'hôte ait vu les entrées corrigées) recorrigeraient le même
    /// écart et le joueur oscillerait.
    fn corriger(&mut self, j: &mut bs_regles::etat::EtatJoueur, numero: u32, x: f32, z: f32) {
        let (n, px, pz) = self.historique[numero as usize % HISTORIQUE];
        let (ex, ez) = if n == numero {
            (x - px, z - pz)
        } else {
            (x - j.x, z - j.z)
        };
        if ex.abs().max(ez.abs()) > TELEPORTATION {
            j.x = x;
            j.z = z;
            self.historique = [(u32::MAX, 0.0, 0.0); HISTORIQUE];
            return;
        }
        let (cx, cz) = (ex * CORRECTION, ez * CORRECTION);
        j.x += cx;
        j.z += cz;
        for h in &mut self.historique {
            h.1 += cx;
            h.2 += cz;
        }
    }
}
