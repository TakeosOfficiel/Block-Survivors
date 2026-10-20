import { defineConfig } from 'vite';
import { viteSingleFile } from 'vite-plugin-singlefile';

export default defineConfig({
  // Chemins relatifs : dist/index.html s'ouvre en double-clic (file://) et dans Electron.
  base: './',
  // Tout le JS et le CSS sont intégrés dans un seul dist/index.html autonome.
  plugins: [viteSingleFile()],
  build: {
    target: 'es2022',
  },
});
