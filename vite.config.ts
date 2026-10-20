import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { defineConfig } from 'vite';
import { viteSingleFile } from 'vite-plugin-singlefile';

const paquet = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8')) as { version: string };

/** Commit construit : fourni par GitHub Actions, sinon lu dans le dépôt local. */
function commit(): string {
  if (process.env.GITHUB_SHA) return process.env.GITHUB_SHA;
  try {
    return execSync('git rev-parse HEAD', { encoding: 'utf8' }).trim();
  } catch {
    return 'inconnu';
  }
}

export default defineConfig({
  // Chemins relatifs : dist/index.html s'ouvre en double-clic (file://) et dans Electron.
  base: './',
  // Tout le JS et le CSS sont intégrés dans un seul dist/index.html autonome.
  plugins: [viteSingleFile()],
  // Version affichée sur l'accueil (et comparée à la dernière version publiée)
  define: {
    __VERSION__: JSON.stringify(paquet.version),
    __COMMIT__: JSON.stringify(commit()),
    __DATE_CONSTRUCTION__: JSON.stringify(new Date().toISOString()),
  },
  build: {
    target: 'es2022',
  },
});
