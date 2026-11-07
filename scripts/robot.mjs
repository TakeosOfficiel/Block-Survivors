/**
 * Robot d'équilibrage (src/robot/robot.ts) : joue des parties complètes sur N graines
 * dans Chromium sans écran, puis écrit les statistiques de référence (moyenne, écart
 * type, min, max) que la version Rust devra retrouver, avec une tolérance.
 *
 *   npm run robot
 *   GRAINES=10 PERSONNAGES=guerrier,archer ARENES=prairie npm run robot
 *
 * Variables : GRAINES (nombre, 5 par défaut), PERSONNAGES et ARENES (listes, tous les
 * personnages et la prairie par défaut), DUREE (s de jeu max, 900 + prolongation),
 * INVINCIBLE=1 (joueur invincible : puissance des armes sur toute la partie),
 * PARALLELE (parties en même temps), SORTIE (dossier, docs/robot), PLAYWRIGHT (chemin
 * du module s'il n'est pas installé dans le projet).
 */
import { mkdirSync, writeFileSync } from 'node:fs';
import { createServer } from 'vite';

const module = await import(process.env.PLAYWRIGHT ?? 'playwright');
const { chromium } = module.chromium ? module : module.default;

const GRAINES = Number(process.env.GRAINES ?? 5);
const PERSONNAGES = (process.env.PERSONNAGES ?? 'guerrier,archer,mage,invocateur').split(',');
const ARENES = (process.env.ARENES ?? 'prairie').split(',');
const DUREE = Number(process.env.DUREE ?? 1200);
const PARALLELE = Number(process.env.PARALLELE ?? 4);
const SORTIE = process.env.SORTIE ?? 'docs/robot';
const INVINCIBLE = process.env.INVINCIBLE === '1';

const serveur = await createServer({ server: { port: 5199, strictPort: true }, logLevel: 'error' });
await serveur.listen();
const navigateur = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--autoplay-policy=no-user-gesture-required'] });

/** Une partie : contexte neuf (profil vierge : ni talents ni Collection), bilan lu sur la page. */
async function jouer(personnage, arene, graine) {
  const contexte = await navigateur.newContext({ viewport: { width: 1280, height: 720 } });
  await contexte.addInitScript(() => {
    localStorage.setItem('block-survivors.options', JSON.stringify({ version: 2, shaders: 'aucun', volume: 0, secousses: false, vue: 'dessus' }));
  });
  const page = await contexte.newPage();
  page.on('pageerror', (e) => console.log(`[${personnage} ${graine}] ERREUR : ${e.message}`));
  await page.goto(`http://localhost:5199/?jouer=1&robot=1&graine=${graine}&personnage=${personnage}&arene=${arene}&duree=${DUREE}${INVINCIBLE ? '&invincible=1' : ''}`);
  await page.waitForFunction(() => window.bilanRobot, null, { timeout: 0, polling: 1000 });
  const bilan = await page.evaluate(() => window.bilanRobot);
  await contexte.close();
  console.log(`${personnage} / ${arene} / graine ${graine} : ${bilan.issue} à ${bilan.temps} s, niveau ${bilan.niveau}, ${bilan.tues} tués (${bilan.calcul} s de calcul)`);
  return bilan;
}

const taches = [];
for (const arene of ARENES) for (const personnage of PERSONNAGES) for (let g = 1; g <= GRAINES; g++) taches.push([personnage, arene, g]);
const bilans = [];
await Promise.all(Array.from({ length: PARALLELE }, async () => {
  while (taches.length > 0) bilans.push(await jouer(...taches.shift()));
}));
await navigateur.close();
await serveur.close();

// --- Statistiques ----------------------------------------------------------

const arrondi = (v) => Math.round(v * 10) / 10;
function resume(valeurs) {
  const n = valeurs.length;
  const moyenne = valeurs.reduce((a, b) => a + b, 0) / n;
  const ecart = Math.sqrt(valeurs.reduce((a, b) => a + (b - moyenne) ** 2, 0) / n);
  return { moyenne: arrondi(moyenne), ecart: arrondi(ecart), min: Math.min(...valeurs), max: Math.max(...valeurs) };
}

const groupes = {};
for (const b of bilans) (groupes[`${b.personnage}/${b.arene}`] ??= []).push(b);
const reference = {};
for (const [cle, liste] of Object.entries(groupes).sort()) {
  const armes = {};
  for (const b of liste) for (const [id, a] of Object.entries(b.armes)) (armes[id] ??= []).push(a.dps);
  const minutes = Math.max(...liste.map((b) => b.puissanceParMinute.length));
  reference[cle] = {
    parties: liste.length,
    victoires: liste.filter((b) => b.issue === 'victoire').length,
    survie: resume(liste.map((b) => b.temps)),
    niveau: resume(liste.map((b) => b.niveau)),
    tues: resume(liste.map((b) => b.tues)),
    degatsSubis: resume(liste.map((b) => b.degatsSubis)),
    puissanceParMinute: Array.from({ length: minutes }, (_, m) => resume(liste.filter((b) => b.puissanceParMinute.length > m).map((b) => b.puissanceParMinute[m])).moyenne),
    dpsParArme: Object.fromEntries(Object.entries(armes).sort().map(([id, v]) => [id, { ...resume(v), parties: v.length }])),
  };
}

mkdirSync(SORTIE, { recursive: true });
writeFileSync(`${SORTIE}/parties.json`, `${JSON.stringify(bilans.sort((a, b) => `${a.personnage}${a.graine}`.localeCompare(`${b.personnage}${b.graine}`)), null, 1)}\n`);
writeFileSync(`${SORTIE}/reference.json`, `${JSON.stringify({ graines: GRAINES, duree: DUREE, invincible: INVINCIBLE, reference }, null, 1)}\n`);
for (const [cle, r] of Object.entries(reference)) {
  console.log(`\n${cle} : ${r.victoires}/${r.parties} victoires, survie ${r.survie.moyenne} s (±${r.survie.ecart}), niveau ${r.niveau.moyenne}, ${r.tues.moyenne} tués`);
  console.log(`  puissance (dégâts/s) par minute : ${r.puissanceParMinute.join(' ')}`);
}
