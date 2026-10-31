/**
 * Banc d'essai du son dans le navigateur (pari 2) : kira + cpal (WebAudio) sous une charge
 * du fil principal. cpal enchaîne deux tampons WebAudio remplis par le fil principal ; si
 * le remplissage arrive trop tard, le tampon part en retard : un trou dans le son.
 * On mesure ces trous en interceptant AudioBufferSourceNode.start(quand).
 *
 *   outils/banc-son.sh   (construit le module puis lance ce script)
 *   node outils/banc-son.mjs <dossier wasm-bindgen>
 *
 * Variables : TAMPONS (0,4096,8192 ; 0 = défaut de cpal, 2048), CHARGES (ms de calcul
 * par image : 0,16,30,50,80), PICS (ms d'une image lente toutes les 2 s : 0,100,250),
 * DUREE (s par essai, 10), PLAYWRIGHT (chemin du module).
 */
import http from 'node:http'; import fs from 'node:fs'; import path from 'node:path';
const m = await import(process.env.PLAYWRIGHT ?? 'playwright'); const { chromium } = m.chromium ? m : m.default;
const racine = process.argv[2];
const page_html = `<!doctype html><meta charset="utf-8"><script type="module">
const p = new URLSearchParams(location.search);
const tampon = +p.get('tampon'), charge = +p.get('charge'), pic = +p.get('pic'), duree = +p.get('duree');
// Interception : chaque tampon de cpal, sa date prévue et l'heure de l'horloge son
const departs = [];
const start = AudioBufferSourceNode.prototype.start;
AudioBufferSourceNode.prototype.start = function (quand) {
  departs.push({ quand, maintenant: this.context.currentTime, duree: this.buffer.duration });
  return start.call(this, quand);
};
const { default: init, demarrer, jouer } = await import('./banc_son.js');
await init();
demarrer(tampon);
let image = 0, dernier_pic = performance.now();
const t0 = performance.now();
function boucle() {
  const debut = performance.now();
  let attente = charge;
  if (pic && debut - dernier_pic > 2000) { attente = pic; dernier_pic = debut; }
  while (performance.now() - debut < attente) {}
  if (image % 6 === 0) jouer(image / 6);
  image++;
  if (performance.now() - t0 < duree * 1000) requestAnimationFrame(boucle);
  else {
    // Trous : début effectif d'un tampon (au plus tôt maintenant) après la fin du précédent
    let fin = null, trous = 0, silence = 0, chevauchements = 0, retards = 0, retard_max = 0;
    for (const d of departs) {
      const effectif = Math.max(d.quand, d.maintenant);
      if (d.quand < d.maintenant) { retards++; retard_max = Math.max(retard_max, d.maintenant - d.quand); }
      if (fin !== null && effectif - fin > 0.003) { trous++; silence += effectif - fin; }
      // Tampon parti en avance sur la fin du précédent (curseur de cpal resté en retard) : les deux se mêlent
      if (fin !== null && fin - effectif > 0.003) chevauchements++;
      fin = Math.max(fin ?? 0, effectif + d.duree);
    }
    window.resultat = { tampons: departs.length, retards, trous, chevauchements, silence_ms: Math.round(silence * 1000),
      retard_max_ms: Math.round(retard_max * 1000), images_s: Math.round(image / duree),
      latence_ms: Math.round(departs.at(-1).duree * 1000) };
  }
}
requestAnimationFrame(boucle);
</script>`;
const types = { '.js': 'text/javascript', '.wasm': 'application/wasm' };
const srv = http.createServer((req, res) => {
  const u = decodeURIComponent(req.url.split('?')[0]);
  if (u === '/') { res.writeHead(200, { 'content-type': 'text/html' }); res.end(page_html); return; }
  fs.readFile(path.join(racine, u), (e, d) => { if (e) { res.writeHead(404); res.end(); return; } res.writeHead(200, { 'content-type': types[path.extname(u)] ?? 'application/octet-stream' }); res.end(d); });
}).listen(5304);
const nav = await chromium.launch({ args: ['--autoplay-policy=no-user-gesture-required'] });
const liste = (v, d) => (process.env[v] ?? d).split(',').map(Number);
const duree = Number(process.env.DUREE ?? 10);
for (const tampon of liste('TAMPONS', '0,4096,8192'))
  for (const charge of liste('CHARGES', '0,16,30,50,80'))
    for (const pic of liste('PICS', '0,100,250')) {
      if (charge && pic) continue; // l'un ou l'autre
      const page = await nav.newPage();
      page.on('pageerror', (e) => console.log('ERREUR', e.message));
      await page.goto(`http://localhost:5304/?tampon=${tampon}&charge=${charge}&pic=${pic}&duree=${duree}`);
      await page.waitForFunction(() => window.resultat, null, { timeout: (duree + 30) * 1000 });
      console.log(JSON.stringify({ tampon: tampon || 2048, charge, pic, ...(await page.evaluate(() => window.resultat)) }));
      await page.close();
    }
await nav.close(); srv.close();
