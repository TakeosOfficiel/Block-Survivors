/**
 * Capture du rendu WebGPU lue de l'intérieur de la page (la capture d'écran de Chromium
 * sans écran sort blanche) : le canevas est configuré avec COPY_SRC et sa texture copiée
 * dans un tampon juste après l'envoi de Bevy, dans la même image ; écrit un PNG.
 *
 *   node rust/outils/pixels-webgpu.mjs rust/web/dist capture.png
 */
import http from 'node:http'; import fs from 'node:fs'; import path from 'node:path';
const m = await import(process.env.PLAYWRIGHT ?? "playwright"); const { chromium } = m.chromium ? m : m.default;
const racine = process.argv[2];
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm' };
const srv = http.createServer((req, res) => { const u = decodeURIComponent(req.url.split('?')[0]); const f = path.join(racine, u === '/' ? 'index.html' : u); fs.readFile(f, (e, d) => { if (e) { res.writeHead(404); res.end(); return; } res.writeHead(200, { 'content-type': types[path.extname(f)] ?? 'application/octet-stream' }); res.end(d); }); }).listen(5303);
const nav = await chromium.launch({ args: ['--enable-unsafe-webgpu', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--enable-features=Vulkan', '--use-webgpu-adapter=swiftshader'] });
const page = await nav.newPage({ viewport: { width: 480, height: 270 } });
page.on('pageerror', (e) => console.log('ERREUR', e.message));
await page.addInitScript(() => {
  const configure = GPUCanvasContext.prototype.configure;
  GPUCanvasContext.prototype.configure = function (c) { this._dev = c.device; this._fmt = c.format; return configure.call(this, { ...c, usage: (c.usage ?? GPUTextureUsage.RENDER_ATTACHMENT) | GPUTextureUsage.COPY_SRC }); };
  const courante = GPUCanvasContext.prototype.getCurrentTexture;
  GPUCanvasContext.prototype.getCurrentTexture = function () { const t = courante.call(this); window.__ctx = this; window.__tex = t; return t; };
  // Copie faite juste après l'envoi de Bevy, dans la même image (après présentation, la texture est vide)
  const submit = GPUQueue.prototype.submit;
  GPUQueue.prototype.submit = function (cmds) {
    const r = submit.call(this, cmds);
    const dem = window.__demande;
    if (dem && window.__tex && !dem.lance) {
      dem.lance = true;
      const ctx = window.__ctx, t = window.__tex, d = ctx._dev;
      const largeur = t.width, hauteur = t.height, ligne = Math.ceil(largeur * 4 / 256) * 256;
      const b = d.createBuffer({ size: ligne * hauteur, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ });
      const e = d.createCommandEncoder(); e.copyTextureToBuffer({ texture: t }, { buffer: b, bytesPerRow: ligne }, [largeur, hauteur]); submit.call(this, [e.finish()]);
      b.mapAsync(GPUMapMode.READ).then(() => { const o = new Uint8Array(b.getMappedRange());
        let bin = ''; for (let y = 0; y < hauteur; y++) { const l = o.subarray(y * ligne, y * ligne + largeur * 4); for (let k = 0; k < l.length; k += 4096) bin += String.fromCharCode.apply(null, l.subarray(k, k + 4096)); }
        dem.fin({ brut: btoa(bin), format: ctx._fmt, largeur, hauteur, pixels: dem.points.map(([fx, fy]) => { const i = Math.floor(fy * hauteur) * ligne + Math.floor(fx * largeur) * 4; return Array.from(o.slice(i, i + 4)); }) }); });
    }
    return r;
  };
  window.lirePixels = (points) => new Promise((fin) => { window.__demande = { points, fin }; });
});
await page.goto('http://localhost:5303/?rendu=webgpu');
await page.waitForFunction(() => window.__tex, null, { timeout: 60000 });
await page.waitForTimeout(Number(process.env.ATTENTE ?? 20000));
// Centre (le bloc), coin (le ciel)
const r = await page.evaluate(() => window.lirePixels([[0.5, 0.5], [0.05, 0.05], [0.5, 0.45]]));
const { brut, ...reste } = r; console.log(JSON.stringify(reste));
// PNG minimal (RGBA 8 bits, sans filtre)
const zlib = await import('node:zlib'); const px = Buffer.from(brut, 'base64');
const lignes = Buffer.alloc((r.largeur * 4 + 1) * r.hauteur);
for (let y = 0; y < r.hauteur; y++) px.copy(lignes, y * (r.largeur * 4 + 1) + 1, y * r.largeur * 4, (y + 1) * r.largeur * 4);
const crc = (b) => { let c = ~0; for (const o of b) { c ^= o; for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1)); } return ~c >>> 0; };
const bloc = (t, d) => { const l = Buffer.alloc(4); l.writeUInt32BE(d.length); const td = Buffer.concat([Buffer.from(t), d]); const c = Buffer.alloc(4); c.writeUInt32BE(crc(td)); return Buffer.concat([l, td, c]); };
const ihdr = Buffer.alloc(13); ihdr.writeUInt32BE(r.largeur, 0); ihdr.writeUInt32BE(r.hauteur, 4); ihdr[8] = 8; ihdr[9] = 6;
fs.writeFileSync(process.argv[3] ?? '/tmp/webgpu.png', Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), bloc('IHDR', ihdr), bloc('IDAT', zlib.deflateSync(lignes)), bloc('IEND', Buffer.alloc(0))]));
await nav.close(); srv.close();
