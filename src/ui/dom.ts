/** Accès typé aux éléments déclarés dans index.html. */

export function element<T extends HTMLElement = HTMLElement>(id: string): T {
  const trouve = document.getElementById(id);
  if (!trouve) throw new Error(`Élément #${id} introuvable dans index.html`);
  return trouve as T;
}
