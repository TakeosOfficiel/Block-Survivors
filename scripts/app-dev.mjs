/**
 * npm run app:dev : lance le serveur Vite puis Electron dessus. Chaque
 * modification du code recharge le jeu dans la fenêtre, sans build.
 */
import { spawn } from 'node:child_process';
import electron from 'electron';
import { createServer } from 'vite';

const serveur = await createServer();
await serveur.listen();
const url = serveur.resolvedUrls?.local[0] ?? 'http://localhost:5173/';

const fenetre = spawn(electron, ['.'], {
  stdio: 'inherit',
  env: { ...process.env, BLOCK_SURVIVORS_URL: url },
});
fenetre.on('exit', async (code) => {
  await serveur.close();
  process.exit(code ?? 0);
});
