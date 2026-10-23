/**
 * Test du multijoueur : un hôte et N joueurs (contextes Chromium séparés, donc
 * sauvegardes séparées) sur le serveur de développement. Échange des codes,
 * lancement, déplacement des joueurs, puis comparaison de ce que voient l'hôte
 * et chaque joueur (mêmes ennemis, mêmes positions).
 *
 *   npm run dev   (dans un autre terminal)
 *   node scripts/test-multijoueur.mjs
 *
 * Variables : JOUEURS (2 à 4), URL, L et H (taille des fenêtres), VITESSE (temps
 * accéléré chez l'hôte), INVINCIBLES=1, DUREE (ms), DOSSIER (captures),
 * COFFRE=1, MINUTEUR=1, SANS_FIN=1, REANIMATION=1 (vérifications en plus),
 * PLAYWRIGHT (chemin du module playwright s'il n'est pas installé dans le projet).
 */
const { chromium } = await import(process.env.PLAYWRIGHT ?? 'playwright');

const URL = process.env.URL ?? 'http://localhost:5173/';
const N = Number(process.env.JOUEURS ?? 2);
const DOSSIER = process.env.DOSSIER ?? '.';
const attendre = (ms) => new Promise((r) => setTimeout(r, ms));

const navigateur = await chromium.launch({
  args: ['--enable-unsafe-swiftshader', '--use-angle=swiftshader', '--disable-features=WebRtcHideLocalIpsWithMdns', '--autoplay-policy=no-user-gesture-required'],
});
const pages = [];
for (let i = 0; i < N; i++) {
  const contexte = await navigateur.newContext({ viewport: { width: Number(process.env.L ?? 1100), height: Number(process.env.H ?? 700) } });
  await contexte.addInitScript(() => {
    if (!localStorage.getItem('block-survivors.options')) localStorage.setItem('block-survivors.options', JSON.stringify({ version: 2, shaders: 'aucun', volume: 0 }));
    let images = 0;
    const compter = () => { images++; requestAnimationFrame(compter); };
    requestAnimationFrame(compter);
    // Cartes : la première, choisie d'office
    setInterval(() => {
      const cartes = document.querySelector('#cartes');
      if (!window.sansChoixAuto && cartes && !cartes.classList.contains('cache')) document.querySelector('#cartes-liste > *')?.click();
    }, 700);
    window.imagesParSeconde = () => new Promise((r) => { const d = images; setTimeout(() => r(images - d), 1000); });
  });
  const page = await contexte.newPage();
  page.on('console', (m) => { if (m.type() === 'error' || m.type() === 'warning') console.log(`[${i}] ${m.type()}: ${m.text().slice(0, 300)}`); });
  page.on('pageerror', (e) => console.log(`[${i}] ERREUR: ${e.message}\n${e.stack}`));
  await page.goto(URL);
  pages.push(page);
}
const [hote, ...invites] = pages;
for (const p of pages) await p.waitForSelector('#menu:not(.cache)', { timeout: 120000 });
console.log('menus chargés');

// Personnages différents pour les reconnaître
const persos = ['guerrier', 'archer', 'mage', 'invocateur'];
for (let i = 0; i < N; i++) {
  await pages[i].click('#menu-multijoueur');
  await pages[i].click(`#salon button[data-id="${persos[i]}"]`);
}
await hote.click('#salon-heberger');
for (const [k, invite] of invites.entries()) {
  await hote.click('#salon-inviter');
  await hote.waitForFunction(() => document.querySelector('#salon-code-invitation')?.value.startsWith('BS1.'), null, { timeout: 30000 });
  const offre = await hote.inputValue('#salon-code-invitation');
  console.log(`invitation ${k + 1} : ${offre.length} caractères`);
  await invite.click('#salon-rejoindre');
  await invite.fill('#salon-code-offre', offre);
  await invite.click('#salon-generer');
  await invite.waitForSelector('#salon-code-retour', { timeout: 30000 });
  const reponse = await invite.inputValue('#salon-code-retour');
  console.log(`réponse ${k + 1} : ${reponse.length} caractères`);
  await hote.fill('#salon-code-reponse', reponse);
  await hote.click('#salon-valider');
  await hote.waitForFunction((n) => document.querySelectorAll('#salon-joueurs > div').length === n, k + 2, { timeout: 30000 });
  await invite.waitForSelector('#salon-joueurs', { timeout: 10000 });
  console.log(`joueur ${k + 1} connecté`);
}
await hote.click('#salon-lancer');
for (const p of pages) await p.waitForFunction(() => window.blockSurvivors?.jeu && document.querySelector('#chargement')?.classList.contains('cache') !== false, null, { timeout: 180000 });
console.log('parties lancées, images/s :', await hote.evaluate(() => window.imagesParSeconde()), await invites[0]?.evaluate(() => window.imagesParSeconde()));
await attendre(4000);
if (process.env.VITESSE) await hote.evaluate((v) => { window.blockSurvivors.jeu.debug.vitesseTemps = v; }, Number(process.env.VITESSE));
if (process.env.INVINCIBLES) await hote.evaluate(() => {
  const jeu = window.blockSurvivors.jeu;
  jeu.joueur.invincible = true;
  for (const c of jeu.reseau.combattants) c.joueur.invincible = true;
});

// Les joueurs invités avancent (chacun dans une direction)
const touches = ['KeyD', 'KeyS', 'KeyA'];
for (const [k, invite] of invites.entries()) await invite.keyboard.down(touches[k]);
await attendre(2500);
for (const [k, invite] of invites.entries()) await invite.keyboard.up(touches[k]);
await attendre(1000);

// Coups portés par les armes des joueurs distants, chez l'hôte
await hote.evaluate(() => {
  const jeu = window.blockSurvivors.jeu;
  window.coupsDistants = 0;
  const blesser = jeu.ennemis.blesser.bind(jeu.ennemis);
  jeu.ennemis.blesser = (...args) => {
    if (jeu.etat.joueur !== jeu.etat.joueurs[0]) window.coupsDistants++;
    return blesser(...args);
  };
});
await attendre(Number(process.env.DUREE ?? 12000));

const etatHote = await hote.evaluate(() => {
  const jeu = window.blockSurvivors.jeu;
  const e = jeu.etat;
  const ids = [];
  for (let i = 0; i < jeu.ennemis.pool.nombre; i++) ids.push(jeu.ennemis.pool.elements[i].id);
  return {
    temps: e.tempsJeu.toFixed(1), tues: e.ennemisTues, niveau: e.joueur.niveau, coupsDistants: window.coupsDistants,
    joueurs: e.joueurs.map((j) => ({ x: j.x.toFixed(2), z: j.z.toFixed(2), pv: j.pv.toFixed(1), mort: j.mort, niveau: j.niveau })),
    arsenaux: [jeu.arsenal, ...jeu.reseau.combattants.map((c) => c.arsenal)].map((a) => a.armes.map((x) => `${x.id}:${x.rang}`).join(',')),
    ennemis: ids.length, ids,
  };
});
console.log('HÔTE', JSON.stringify({ ...etatHote, ids: undefined }));
for (const [k, invite] of invites.entries()) {
  const etat = await invite.evaluate(() => {
    const jeu = window.blockSurvivors.jeu;
    const e = jeu.etat;
    const ids = [];
    for (let i = 0; i < jeu.ennemis.pool.nombre; i++) ids.push(jeu.ennemis.pool.elements[i].id);
    let degats = 0;
    for (const d of e.stats.degatsParArme) degats += d;
    return {
      arsenal: jeu.arsenal.armes.map((x) => `${x.id}:${x.rang}`).join(','),
      autres: jeu.reseau.listeAutres.map((c) => c.arsenal.armes.map((x) => `${x.id}:${x.rang}`).join(',')),
      compteurDegats: Math.round(degats), temps: e.tempsJeu.toFixed(1), tues: e.ennemisTues, niveau: e.joueur.niveau, x: e.joueur.x.toFixed(2), z: e.joueur.z.toFixed(2), pv: e.joueur.pv.toFixed(1), ennemis: ids.length, ids };
  });
  const communs = etat.ids.filter((id) => etatHote.ids.includes(id)).length;
  console.log(`INVITÉ ${k + 1}`, JSON.stringify({ ...etat, ids: undefined, communsAvecHote: communs }));
}
for (let i = 0; i < N; i++) await pages[i].screenshot({ path: `${DOSSIER}/joueur${i}.png` });

// Coffre ouvert chez l'hôte : le joueur 1 reçoit aussi ses cartes (choisies d'office par le test)
if (process.env.COFFRE) {
  await hote.evaluate(() => {
    const etat = window.blockSurvivors.jeu.etat;
    etat.choixCoffre.restants += 1;
  });
  await hote.waitForFunction(() => window.blockSurvivors.jeu.reseau.combattants[0].equipement.portes().length > 0, null, { timeout: 90000 });
  const chezHote = await hote.evaluate(() => window.blockSurvivors.jeu.reseau.combattants[0].equipement.portes().map((o) => `${o.id}:${o.niveau}`).join(','));
  const chezLui = await invites[0].evaluate(() => window.blockSurvivors.jeu.equipement.portes().map((o) => `${o.id}:${o.niveau}`).join(','));
  console.log(`COFFRE : objets du joueur 1 chez lui [${chezLui}], chez l'hôte [${chezHote}]`);
}

// Minuteur des cartes : le joueur 1 ne choisit pas, une carte est prise d'office au bout du délai
if (process.env.MINUTEUR) {
  await invites[0].evaluate(() => {
    window.sansChoixAuto = true;
    // Durée de chaque ouverture de l'écran des cartes, et le bandeau affiché pendant
    window.durees = [];
    window.bandeaux = [];
    let ouverture = -1;
    const cartes = document.querySelector('#cartes');
    new MutationObserver(() => {
      const ouvert = !cartes.classList.contains('cache');
      if (ouvert && ouverture < 0) ouverture = performance.now();
      if (!ouvert && ouverture >= 0) {
        window.durees.push(((performance.now() - ouverture) / 1000).toFixed(1));
        ouverture = -1;
      }
    }).observe(cartes, { attributes: true });
    setInterval(() => {
      const texte = document.querySelector('.reseau-alerte:not(.cache)')?.textContent;
      if (texte && !window.bandeaux.includes(texte) && window.bandeaux.length < 3) window.bandeaux.push(texte);
    }, 1000);
  });
  await hote.evaluate(() => {
    const etat = window.blockSurvivors.jeu.etat;
    etat.choixCoffre.restants += 1;
  });
  await invites[0].waitForFunction(() => !document.querySelector('#cartes').classList.contains('cache'), null, { timeout: 30000 });
  await hote.waitForFunction(() => window.blockSurvivors.jeu.reseau.combattants[0].cartesEnAttente > 0, null, { timeout: 30000 });
  const debut = Date.now();
  await hote.waitForFunction(() => window.blockSurvivors.jeu.reseau.combattants[0].cartesEnAttente === 0, null, { timeout: 120000 });
  console.log(`MINUTEUR : choix fait d'office en ${((Date.now() - debut) / 1000).toFixed(1)} s`, JSON.stringify(await invites[0].evaluate(() => ({ bandeaux: window.bandeaux }))));
  await invites[0].evaluate(() => { window.sansChoixAuto = false; });
}

// Mode sans fin : victoire, l'hôte continue, le joueur 1 reprend aussi
if (process.env.SANS_FIN) {
  await hote.evaluate(() => { window.blockSurvivors.jeu.etat.tempsJeu = 15 * 60; });
  await invites[0].waitForFunction(() => window.blockSurvivors.jeu.etat.issue === 'victoire', null, { timeout: 120000 });
  console.log('SANS FIN : victoire reçue par le joueur 1');
  // La maîtrise monte, les records du solo ne bougent pas, ceux à plusieurs sont à part
  const sauvegardes = await invites[0].evaluate(() => {
    const lire = (cle) => JSON.parse(localStorage.getItem(cle) ?? '{}');
    const solo = lire('block-survivors.progression');
    const equipe = lire('block-survivors.progression.multi');
    return { maitriseSolo: solo.maitrise, recordsSolo: solo.records, recordsEquipe: equipe.records };
  });
  console.log('SAUVEGARDES du joueur 1 :', JSON.stringify(sauvegardes));
  await hote.evaluate(() => window.blockSurvivors.jeu.continuerSansFin());
  await invites[0].waitForFunction(() => window.blockSurvivors.jeu.etat.issue === null && window.blockSurvivors.jeu.etat.sansFin, null, { timeout: 60000 });
  console.log('SANS FIN : le joueur 1 continue avec l\'hôte');
}

// Réanimation : le joueur 1 tombe à côté de l'hôte, qui doit le relever
if (process.env.REANIMATION) {
  await hote.evaluate(() => {
    const jeu = window.blockSurvivors.jeu;
    const c = jeu.reseau.combattants[0];
    c.joueur.invincible = false;
    const j = c.etatJoueur;
    j.x = jeu.etat.joueur.x + 0.8;
    j.z = jeu.etat.joueur.z;
    j.pv = 0;
    j.mort = true;
    j.tempsMort = 0;
  });
  await attendre(1500);
  const pendant = await invites[0].evaluate(() => ({ mort: window.blockSurvivors.jeu.etat.joueur.mort, bandeau: document.querySelector('.reseau-alerte:not(.cache)')?.textContent ?? '' }));
  console.log('RÉANIMATION pendant', JSON.stringify(pendant));
  await hote.waitForFunction(() => !window.blockSurvivors.jeu.reseau.combattants[0].etatJoueur.mort, null, { timeout: 120000 });
  await attendre(1500);
  const apres = await invites[0].evaluate(() => ({ mort: window.blockSurvivors.jeu.etat.joueur.mort, pv: window.blockSurvivors.jeu.etat.joueur.pv }));
  console.log('RÉANIMATION après', JSON.stringify(apres));
}
await navigateur.close();
