/** Lecture des pixels d'une image chargée (CORS requis pour les images distantes). */

export function lirePixels(image: CanvasImageSource & { width: number; height: number }): ImageData {
  const canvas = document.createElement('canvas');
  canvas.width = image.width;
  canvas.height = image.height;
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (!ctx) throw new Error('Canvas 2D indisponible');
  ctx.drawImage(image, 0, 0);
  return ctx.getImageData(0, 0, image.width, image.height);
}
