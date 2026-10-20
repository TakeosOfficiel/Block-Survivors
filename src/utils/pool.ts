/**
 * Pool d'objets préalloués : aucune allocation pendant la partie.
 * Les éléments actifs occupent les indices [0, nombre) ; libérer un élément
 * l'échange avec le dernier actif (parcourir à rebours pour libérer en boucle).
 */
export class Pool<T> {
  readonly elements: T[];
  nombre = 0;

  constructor(capacite: number, fabrique: () => T) {
    this.elements = Array.from({ length: capacite }, fabrique);
  }

  get capacite(): number {
    return this.elements.length;
  }

  /** Élément libre (à initialiser par l'appelant), ou null si le pool est plein. */
  acquerir(): T | null {
    if (this.nombre >= this.elements.length) return null;
    return this.elements[this.nombre++];
  }

  liberer(index: number): void {
    const dernier = --this.nombre;
    if (index === dernier) return;
    const libre = this.elements[index];
    this.elements[index] = this.elements[dernier];
    this.elements[dernier] = libre;
  }
}
