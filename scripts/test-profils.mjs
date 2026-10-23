/**
 * Test des profils sur le serveur de développement : migration de la
 * sauvegarde d'avant les profils, création, changement, export puis import
 * (nouveau profil), suppression.
 *
 *   npm run dev   (dans un autre terminal)
 *   node scripts/test-profils.mjs
 *
 * Variables : URL, DOSSIER (fichier exporté), PLAYWRIGHT (chemin du module
 * playwright s'il n'est pas installé dans le projet).
 */
const { chromium } = await import(process.env.PLAYWRIGHT ?? 'playwright');

const URL = process.env.URL ?? 'http://localhost:5173/';
const DOSSIER = process.env.DOSSIER ?? '.';
let echecs = 0;
const verifier = (condition, message) => {
  console.log(`${condition ? 'OK  ' : 'ÉCHEC'} ${message}`);
  if (!condition) echecs++;
};

const navigateur = await chromium.launch({ args: ['--enable-unsafe-swiftshader', '--use-angle=swiftshader'] });
const contexte = await navigateur.newContext({ viewport: { width: 1100, height: 700 }, acceptDownloads: true });
const page = await contexte.newPage();
page.on('pageerror', (e) => console.log(`ERREUR : ${e.message}`));
page.on('dialog', (d) => d.accept());
const menu = () => page.waitForSelector('#menu:not(.cache)', { timeout: 120000 });
const pseudo = () => page.inputValue('#menu-pseudo');
const stockage = () => page.evaluate(() => Object.fromEntries(Object.keys(localStorage).filter((c) => c.startsWith('block-survivors.')).map((c) => [c, localStorage.getItem(c)])));

// Sauvegarde d'avant les profils
await page.goto(URL, { waitUntil: 'domcontentloaded' });
await page.evaluate(() => {
  localStorage.clear();
  localStorage.setItem('block-survivors.profil', JSON.stringify({ pseudo: 'Ancien', skin: 'steve', brasFins: false, skinImporte: null }));
  localStorage.setItem('block-survivors.progression', JSON.stringify({ version: 3, maitrise: { mage: 5000 }, records: { prairie: 321 } }));
  localStorage.setItem('block-survivors.collection', JSON.stringify({ pieces: {}, portees: {}, decouverts: [] }));
  localStorage.setItem('block-survivors.options', JSON.stringify({ version: 2, volume: 0.3 }));
});
await page.reload({ waitUntil: 'domcontentloaded' });
await menu();
let s = await stockage();
verifier(await pseudo() === 'Ancien', 'migration : le pseudo d\'avant est celui du profil 1');
verifier(!('block-survivors.progression' in s) && 'block-survivors.p.1.progression' in s, 'migration : la progression est rangée dans le profil 1');
verifier('block-survivors.options' in s, 'migration : les options restent communes');

// Nouveau profil
await page.click('#menu-profils');
await page.fill('#profils-pseudo', 'Second');
await page.click('#profils-creer');
await menu();
s = await stockage();
verifier(await pseudo() === 'Second', 'création : le jeu redémarre sur le nouveau profil');
verifier(JSON.parse(s['block-survivors.profils']).ids.length === 2, 'création : 2 profils');

// Retour au premier
await page.click('#menu-profils');
const lignes = await page.$$eval('.profils-ligne', (l) => l.map((e) => e.textContent));
verifier(lignes.some((t) => t.includes('Ancien') && t.includes('Mage')), 'liste : le premier profil montre sa maîtrise de Mage');
await page.click('.profils-ligne:not(.actif) >> text=Jouer avec');
await menu();
verifier(await pseudo() === 'Ancien', 'changement : retour au premier profil');

// Export puis import : un nouveau profil identique
await page.click('#menu-profils');
const [telechargement] = await Promise.all([page.waitForEvent('download'), page.click('#profils-exporter')]);
const fichier = `${DOSSIER}/profil-test.bssave`;
await telechargement.saveAs(fichier);
await page.setInputFiles('#profils-fichier', fichier);
await menu();
s = await stockage();
const registre = JSON.parse(s['block-survivors.profils']);
verifier(registre.ids.length === 3 && registre.actif === '3', 'import : un 3e profil, devenu l\'actif');
verifier(await pseudo() === 'Ancien' && JSON.parse(s['block-survivors.p.3.progression']).records.prairie === 321, 'import : pseudo et records repris');

// Suppression du profil actif : retour au premier
await page.click('#menu-profils');
await page.click('.profils-ligne.actif >> text=Supprimer');
await menu();
s = await stockage();
verifier(JSON.parse(s['block-survivors.profils']).ids.length === 2 && !('block-survivors.p.3.progression' in s), 'suppression : profil et données retirés');

await navigateur.close();
console.log(echecs === 0 ? 'Tous les tests des profils passent.' : `${echecs} échec(s).`);
process.exit(echecs === 0 ? 0 : 1);
