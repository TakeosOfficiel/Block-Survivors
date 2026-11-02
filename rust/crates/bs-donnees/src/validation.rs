//! Validation des références entre tables : une arme qui cite un projectile
//! absent, une arène qui fait apparaître un ennemi inconnu… Chaque erreur dit
//! quel fichier et quelle entrée corriger.

use crate::Donnees;
use crate::arenes::TacheSol;
use crate::communs::Table;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErreurValidation {
    pub fichier: &'static str,
    pub entree: String,
    pub message: String,
}

impl std::fmt::Display for ErreurValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.ron, {} : {}",
            self.fichier, self.entree, self.message
        )
    }
}

struct Verif<'a> {
    d: &'a Donnees,
    erreurs: Vec<ErreurValidation>,
}

impl Verif<'_> {
    fn exige<T>(
        &mut self,
        fichier: &'static str,
        entree: &str,
        table: &Table<T>,
        quoi: &str,
        id: &str,
    ) {
        if !table.contains_key(id) {
            self.erreurs.push(ErreurValidation {
                fichier,
                entree: entree.to_owned(),
                message: format!("{quoi} inconnu(e) « {id} »"),
            });
        }
    }

    fn erreur(&mut self, fichier: &'static str, entree: &str, message: String) {
        self.erreurs.push(ErreurValidation {
            fichier,
            entree: entree.to_owned(),
            message,
        });
    }

    fn apparence(&mut self, fichier: &'static str, entree: &str, a: &crate::Apparence) {
        let d = self.d;
        self.exige(fichier, entree, &d.modeles, "modèle", &a.modele);
        if let Some(m) = d.modeles.get(&a.modele) {
            for partie in a.couleurs_secours.keys() {
                if !m.parties.contains_key(partie) {
                    self.erreur(
                        fichier,
                        entree,
                        format!("couleur de secours pour une partie absente « {partie} »"),
                    );
                }
            }
        }
    }
}

pub(crate) fn valider(d: &Donnees) -> Vec<ErreurValidation> {
    let mut v = Verif {
        d,
        erreurs: Vec::new(),
    };

    for (id, m) in &d.modeles {
        for partie in m.parties_citees() {
            if !m.parties.contains_key(partie) {
                v.erreur("modeles", id, format!("partie inconnue « {partie} »"));
            }
        }
    }
    for (id, e) in &d.ennemis {
        v.apparence("ennemis", id, &e.apparence);
        if let crate::entites::Comportement::Archer { projectile, .. } = &e.comportement {
            v.exige("ennemis", id, &d.projectiles, "projectile", projectile);
        }
        if let Some(b) = &e.butin {
            v.exige("ennemis", id, &d.objets, "objet", b);
        }
        if let Some(s) = &e.sons {
            v.exige("ennemis", id, &d.sons, "son", &s.blessure);
            v.exige("ennemis", id, &d.sons, "son", &s.mort);
        }
    }
    for (id, p) in &d.projectiles {
        if let Some(m) = &p.modele {
            v.exige("projectiles", id, &d.modeles, "modèle", m);
        }
        if let Some(t) = &p.trainee {
            v.exige("projectiles", id, &d.particules.gerbes, "gerbe", t);
        }
    }
    for (id, a) in &d.armes {
        if let Some(p) = a.mecanique.projectile() {
            v.exige("armes", id, &d.projectiles, "projectile", p);
        }
        match &a.mecanique {
            crate::armes::Mecanique::Invocation { allie, .. } => {
                v.exige("armes", id, &d.allies.allies, "allié", allie)
            }
            crate::armes::Mecanique::Explosif { gerbe, son, .. } => {
                if let Some(g) = gerbe {
                    v.exige("armes", id, &d.particules.gerbes, "gerbe", g);
                }
                if let Some(s) = son {
                    v.exige("armes", id, &d.sons, "son", s);
                }
            }
            _ => {}
        }
        if let Some(s) = &a.son {
            v.exige("armes", id, &d.sons, "son", s);
        }
        if !d.regles.profite_projectiles.contains_key(a.mecanique.nom()) {
            v.erreur(
                "regles",
                "profite_projectiles",
                format!("mécanique absente « {} »", a.mecanique.nom()),
            );
        }
    }
    for (id, a) in &d.allies.allies {
        v.apparence("allies", id, &a.apparence);
        for s in [&a.son_coup, &a.son_blessure, &a.son_mort] {
            v.exige("allies", id, &d.sons, "son", s);
        }
        if let Some(t) = &a.tir {
            v.exige("allies", id, &d.projectiles, "projectile", &t.projectile);
        }
    }
    for (id, a) in &d.allies.creatures_talents {
        v.apparence("allies", id, a);
    }
    for (id, p) in &d.personnages.personnages {
        v.apparence("personnages", id, &p.apparence);
        for a in p.armes_depart.iter().chain(&p.arsenal) {
            v.exige("personnages", id, &d.armes, "arme", a);
        }
        for c in &p.cartes_exclusives {
            v.exige("personnages", id, &d.passifs, "passif", c);
        }
    }
    let defaut = d.personnages.par_defaut.clone();
    v.exige(
        "personnages",
        "par_defaut",
        &d.personnages.personnages,
        "personnage",
        &defaut,
    );

    for a in &d.arenes.arenes {
        let e = a.id.as_str();
        for b in a.sol.iter().chain(&a.murs) {
            v.exige("arenes", e, &d.blocs, "bloc", &b.bloc);
        }
        for t in &a.taches {
            let (TacheSol::Rectangle { bloc, .. } | TacheSol::Taches { bloc, .. }) = t;
            v.exige("arenes", e, &d.blocs, "bloc", bloc);
        }
        for el in &a.decor {
            v.exige("arenes", e, &d.structures, "structure", el.structure());
        }
        if let Some(x) = &a.exterieur {
            v.exige("arenes", e, &d.blocs, "bloc", &x.bloc);
        }
        let choix = a
            .ennemis
            .iter()
            .chain(a.zones.iter().flat_map(|z| &z.ennemis));
        for en in a
            .boss
            .iter()
            .chain(&a.mini_boss)
            .chain(choix.map(|c| &c.ennemi))
        {
            v.exige("arenes", e, &d.ennemis, "ennemi", en);
        }
    }
    if !d.arenes.arenes.iter().any(|a| a.id == d.arenes.par_defaut) {
        v.erreur(
            "arenes",
            "par_defaut",
            format!("arène inconnue « {} »", d.arenes.par_defaut),
        );
    }
    for (id, s) in &d.structures {
        for b in s.legende.values() {
            v.exige("structures", id, &d.blocs, "bloc", b);
        }
        for ligne in s.couches.iter().flatten() {
            for c in ligne.chars().filter(|&c| c != '.') {
                if !s.legende.contains_key(&c) {
                    v.erreur("structures", id, format!("caractère sans légende « {c} »"));
                }
            }
        }
    }
    for (id, c) in &d.coffres.coffres {
        v.apparence("coffres", id, &c.apparence);
        v.exige("coffres", id, &d.sons, "son", &c.son);
    }
    for (id, o) in &d.equipement.objets {
        if o.description.len() != d.equipement.regles.niveaux.len() {
            v.erreur(
                "equipement",
                id,
                "une description par niveau d'objet".to_owned(),
            );
        }
    }
    for (id, paire) in &d.talents.arbres {
        v.exige("talents", id, &d.personnages.personnages, "personnage", id);
        for arbre in paire {
            for n in &arbre.noeuds {
                let entree = format!("{} / {}", arbre.id, n.id);
                for p in &n.prerequis {
                    if !arbre.noeuds.iter().any(|m| &m.id == p) {
                        v.erreur("talents", &entree, format!("prérequis inconnu « {p} »"));
                    }
                }
                let gains =
                    std::iter::once(&n.gains).chain(n.options.iter().flatten().map(|o| &o.gains));
                for g in gains {
                    let armes = g
                        .armes
                        .iter()
                        .map(|b| &b.arme)
                        .chain(&g.rangs_arme)
                        .chain(&g.arme_depart);
                    for a in armes {
                        v.exige("talents", &entree, &d.armes, "arme", a);
                    }
                }
            }
        }
    }
    for (k, s) in d.succes.iter().enumerate() {
        if d.succes[..k].iter().any(|t| t.id == s.id) {
            v.erreur("talents", &s.id, "succès en double".to_owned());
        }
        // Joueurs au plus dans une partie (bs-regles `equipe::regles::JOUEURS_MAX`)
        let max = 4;
        if s.condition.joueurs > max {
            v.erreur("talents", &s.id, format!("plus de {max} joueurs"));
        }
    }
    for (id, p) in &d.particules.particules {
        // « morceau » prend son image de l'icône donnée à l'émission
        if p.images.is_empty() && p.animation != crate::effets::AnimationParticule::Morceau {
            v.erreur("particules", id, "aucune image".to_owned());
        }
    }
    for (id, g) in &d.particules.gerbes {
        for s in g {
            v.exige(
                "particules",
                id,
                &d.particules.particules,
                "particule",
                &s.particule,
            );
        }
    }
    for (id, e) in &d.regles.etats {
        v.exige("regles", id, &d.particules.gerbes, "gerbe", &e.gerbe);
    }
    v.erreurs
}
