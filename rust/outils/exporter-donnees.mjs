/**
 * Exporte les données de la version TypeScript (src/donnees) en JSON, une table par
 * fichier, pour l'importeur Rust (bs-donnees, binaire « importer-donnees ») qui les
 * vérifie contre ses types et les écrit en RON dans rust/donnees/.
 *
 * Usage (depuis la racine du dépôt) : node rust/outils/exporter-donnees.mjs <dossier>
 *
 * Les modules TypeScript sont chargés par Vite (résolution des imports sans extension).
 * Infinity devient null ; les descriptions de l'équipement (des fonctions du niveau)
 * sont évaluées pour chaque niveau d'objet.
 */
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { createServer } from 'vite';

const sortie = process.argv[2];
if (!sortie) {
  console.error('usage : node rust/outils/exporter-donnees.mjs <dossier>');
  process.exit(1);
}
mkdirSync(sortie, { recursive: true });

const serveur = await createServer({ server: { middlewareMode: true }, appType: 'custom', logLevel: 'error' });
const charger = (nom) => serveur.ssrLoadModule(`/src/donnees/${nom}.ts`);
const m = {};
for (const nom of [
  'allies', 'apparitions', 'arenes', 'armes', 'armures', 'astuces', 'blocs', 'boss', 'coffres', 'ennemis',
  'equipement', 'etats', 'joueur', 'modeles', 'multijoueur', 'objets', 'particules', 'passifs', 'personnages',
  'plafonds', 'progression', 'projectiles', 'skins', 'sons', 'structures', 'talents',
]) m[nom] = await charger(nom);
await serveur.close();

function ecrire(nom, valeur) {
  const texte = JSON.stringify(valeur, (_, v) => (v === Infinity ? null : v), 2);
  writeFileSync(join(sortie, `${nom}.json`), texte + '\n');
}

const niveaux = m.equipement.REGLES_EQUIPEMENT.niveaux;
const equipement = Object.fromEntries(Object.entries(m.equipement.OBJETS_EQUIPEMENT).map(([id, o]) => [
  id, { ...o, description: niveaux.map((f) => o.description(f)) },
]));

ecrire('blocs', m.blocs.BLOCS);
ecrire('modeles', m.modeles.MODELES);
ecrire('ennemis', m.ennemis.ENNEMIS);
ecrire('projectiles', m.projectiles.PROJECTILES);
ecrire('armes', m.armes.ARMES);
ecrire('allies', { allies: m.allies.ALLIES, creatures_talents: m.allies.CREATURES_TALENTS });
ecrire('personnages', { personnages: m.personnages.PERSONNAGES, par_defaut: m.personnages.PERSONNAGE_PAR_DEFAUT });
// Arènes construites à la main ; les arènes importées (cartes/*.schem) se lisent au lancement
ecrire('arenes', { arenes: m.arenes.ARENES.filter((a) => !a.source), par_defaut: m.arenes.ARENE_PAR_DEFAUT });
ecrire('structures', m.structures.STRUCTURES);
ecrire('passifs', m.passifs.PASSIFS);
ecrire('objets', m.objets.OBJETS);
ecrire('coffres', { coffres: m.coffres.COFFRES, regles: m.coffres.REGLES_COFFRES });
ecrire('equipement', { objets: equipement, raretes: m.equipement.RARETES_OBJETS, regles: m.equipement.REGLES_EQUIPEMENT });
ecrire('armures', {
  armures: m.armures.ARMURES, noms_mondes: m.armures.NOMS_MONDES, noms_pieces: m.armures.NOMS_PIECES,
  butin_boss: m.armures.BUTIN_BOSS,
});
ecrire('talents', {
  paliers: m.talents.PALIERS, mecaniques_pretes: [...m.talents.MECANIQUES_PRETES], arbres: m.talents.ARBRES,
  maitrise: m.talents.MAITRISE, deblocages: m.talents.DEBLOCAGES,
});
ecrire('sons', m.sons.SONS);
ecrire('particules', { particules: m.particules.PARTICULES, gerbes: m.particules.GERBES });
ecrire('regles', {
  joueur: m.joueur.JOUEUR_BASE, apparitions: m.apparitions.APPARITIONS, boss: m.boss.CALENDRIER_BOSS,
  mini_boss: m.boss.CALENDRIER_MINIBOSS, progression: m.progression.PROGRESSION, plafonds: m.plafonds.PLAFONDS,
  profite_projectiles: m.plafonds.PROFITE_PROJECTILES, etats: m.etats.ETATS, resistance_boss: m.etats.RESISTANCE_BOSS,
  multijoueur: m.multijoueur.REGLES_MULTIJOUEUR,
});
ecrire('textes', { astuces: m.astuces.ASTUCES, skins: m.skins.NOMS_SKINS });
console.log(`données exportées dans ${sortie}`);
