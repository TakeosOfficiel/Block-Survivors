/**
 * Ouvre le build web (rust/web/dist) dans Chromium sans écran, en WebGL2 puis WebGPU :
 * temps de chargement, images/s, erreurs de la console, capture d'écran.
 *
 *   node rust/outils/page-web.mjs rust/web/dist
 *
 * Variables : RENDUS (webgl2,webgpu), SORTIE (dossier des captures, /tmp), ATTENTE (ms
 * avant la mesure, 5000), PLAYWRIGHT (chemin du module). Sans GPU, Chromium rend par
 * SwiftShader (lent : les images/s ne valent rien). Attention : la capture d'un canevas
 * WebGPU sort blanche sans écran, même pour une page WebGPU minimale ; pour voir le rendu
 * WebGPU, utiliser pixels-webgpu.mjs.
 */
import http from 'node:http'; import fs from 'node:fs'; import path from 'node:path';
const m = await import(process.env.PLAYWRIGHT ?? "playwright"); const { chromium } = m.chromium ? m : m.default;
const racine = process.argv[2];
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm' };
const srv = http.createServer((req, res) => { const f = path.join(racine, decodeURIComponent(req.url.split('?')[0]) === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0])); fs.readFile(f, (e, d) => { if (e) { res.writeHead(404); res.end(); return; } res.writeHead(200, { 'content-type': types[path.extname(f)] ?? 'application/octet-stream' }); res.end(d); }); }).listen(5300);
const nav = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--enable-unsafe-webgpu', '--enable-features=Vulkan', '--use-webgpu-adapter=swiftshader'] });
for (const rendu of (process.env.RENDUS ?? 'webgl2,webgpu').split(',')) {
  const page = await nav.newPage({ viewport: { width: 960, height: 540 } });
  const erreurs = [];
  page.on('pageerror', (e) => erreurs.push(e.message.slice(0, 300)));
  page.on('console', (c) => { if (c.type() === 'error') erreurs.push(c.text().slice(0, 300)); });
  const t0 = Date.now();
  await page.goto(`http://localhost:5300/?rendu=${rendu}`);
  try { await page.waitForFunction(() => !document.getElementById('message'), null, { timeout: 120000 }); } catch { erreurs.push('délai dépassé'); }
  const charge = (Date.now() - t0) / 1000;
  await page.waitForTimeout(Number(process.env.ATTENTE ?? 5000));
  const ips = await page.evaluate(() => new Promise((r) => { let n = 0; const f = () => { n++; requestAnimationFrame(f); }; requestAnimationFrame(f); setTimeout(() => r(n / 3), 3000); }));
  const msg = await page.evaluate(() => document.getElementById('message')?.textContent ?? null);
  await page.screenshot({ path: `${process.env.SORTIE ?? '/tmp'}/web-${rendu}.png` });
  console.log(JSON.stringify({ rendu, charge, ips, message: msg, erreurs: erreurs.slice(0, 5) }));
  await page.close();
}
await nav.close(); srv.close();
