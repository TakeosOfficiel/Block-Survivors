/**
 * Exporte un vrai fichier .bssave depuis la version TypeScript (profil créé au premier
 * lancement), comme le bouton « Exporter » : sauvegardes de test de la version Rust
 * (rust/crates/bs-sauvegarde/tests/donnees/).
 *
 *   node scripts/exporter-bssave.mjs sortie.bssave [pseudo]
 */

import { createServer } from 'vite';
const module = await import(process.env.PLAYWRIGHT ?? 'playwright');
const { chromium } = module.chromium ? module : module.default;
const [sortie, pseudo = 'Testeur'] = process.argv.slice(2);
const serveur = await createServer({ server: { port: 5310, strictPort: true }, logLevel: 'error' });
await serveur.listen();
const nav = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
const page = await nav.newPage({ acceptDownloads: true });
page.on('pageerror', (e) => console.log('ERREUR', e.message));
await page.goto('http://localhost:5310/');
await page.waitForTimeout(4000);
// Un profil qui a vécu, par les fonctions du jeu : pseudo, deux parties terminées
await page.evaluate(async (nom) => {
  const { chargerProfil, enregistrerProfil } = await import('/src/profil.ts');
  const { Sauvegarde } = await import('/src/progression/sauvegarde.ts');
  enregistrerProfil({ ...chargerProfil(), pseudo: nom });
  const s = Sauvegarde.charger();
  s.terminerPartie({ personnage: 'guerrier', arene: 'prairie', xp: 1800, temps: 412, boss: 1, enEquipe: false, sansFin: false });
  s.terminerPartie({ personnage: 'mage', arene: 'prairie', xp: 950, temps: 240, boss: 0, enEquipe: false, sansFin: false });
}, pseudo);
const avant = await page.evaluate(() => Object.fromEntries(Object.entries(localStorage).filter(([k]) => k.startsWith('block-survivors'))));
console.log('stockage :', Object.keys(avant));
const telechargement = page.waitForEvent('download');
await page.evaluate(async () => { const t = await import('/src/progression/transfert.ts'); await t.exporterProfil(); });
const d = await telechargement;
await d.saveAs(sortie);
console.log('exporté :', d.suggestedFilename());
await nav.close(); await serveur.close();
