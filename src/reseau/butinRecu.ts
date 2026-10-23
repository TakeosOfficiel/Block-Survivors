/**
 * Orbes d'XP, objets au sol et coffres dans les instantanés : l'hôte écrit
 * ceux qui entourent le joueur (les coffres : tous), le joueur reconstruit ses
 * pools à chaque instantané (rien à interpoler, ils bougent peu).
 */
import type { Coffres } from '../entites/coffres';
import type { Objets } from '../entites/objets';
import type { Orbes } from '../entites/orbes';
import type { Ecrivain, Lecteur } from './protocole';

export interface Butin {
  readonly orbes: Orbes;
  readonly objets: Objets;
  readonly coffres: Coffres;
}

/** Hôte : le butin à moins de √r2 de (cx, cz), tant qu'il reste de la place. */
export function ecrireButin(w: Ecrivain, { orbes, objets, coffres }: Butin, cx: number, cz: number, r2: number): void {
  let compte = w.reserver16();
  let n = 0;
  for (let i = 0; i < orbes.pool.nombre && w.reste > 12 * 1024; i++) {
    const o = orbes.pool.elements[i];
    if ((o.x - cx) ** 2 + (o.z - cz) ** 2 > r2) continue;
    w.coordonnee(o.x);
    w.coordonnee(o.z);
    w.u16(Math.min(65535, Math.round(o.valeur)));
    n++;
  }
  w.ecrire16(compte, n);

  compte = w.reserver16();
  n = 0;
  for (let i = 0; i < objets.pool.nombre && w.reste > 8 * 1024; i++) {
    const o = objets.pool.elements[i];
    if ((o.x - cx) ** 2 + (o.z - cz) ** 2 > r2) continue;
    w.u8(o.indexType);
    w.coordonnee(o.x);
    w.coordonnee(o.z);
    w.u8(Math.min(255, o.valeur));
    n++;
  }
  w.ecrire16(compte, n);

  w.u16(coffres.ouverts);
  compte = w.reserver16();
  n = 0;
  for (let i = 0; i < coffres.pool.nombre && w.reste > 6 * 1024; i++) {
    const o = coffres.pool.elements[i];
    w.u8(o.indexType);
    w.coordonnee(o.x);
    w.coordonnee(o.z);
    w.angle(o.angle);
    w.f32(o.ouvert);
    n++;
  }
  w.ecrire16(compte, n);
}

/** Joueur : remplace orbes, objets et coffres par ceux de l'instantané. */
export function lireButin(l: Lecteur, { orbes, objets, coffres }: Butin): void {
  let n = l.u16();
  orbes.pool.nombre = 0;
  for (let i = 0; i < n; i++) {
    const x = l.coordonnee();
    const z = l.coordonnee();
    const valeur = l.u16();
    const o = orbes.pool.acquerir();
    if (!o) continue;
    o.x = x;
    o.z = z;
    o.valeur = valeur;
    o.aimantee = false;
    o.phase = x * 3.7 + z * 5.3;
  }

  n = l.u16();
  objets.pool.nombre = 0;
  for (let i = 0; i < n; i++) {
    const type = l.u8();
    const x = l.coordonnee();
    const z = l.coordonnee();
    const valeur = l.u8();
    const o = objets.pool.acquerir();
    if (!o) continue;
    o.indexType = type;
    o.x = x;
    o.z = z;
    o.valeur = valeur;
    o.phase = x + z;
    o.aimante = false;
  }

  coffres.ouverts = l.u16();
  n = l.u16();
  coffres.pool.nombre = 0;
  for (let i = 0; i < n; i++) {
    const type = l.u8();
    const x = l.coordonnee();
    const z = l.coordonnee();
    const angle = l.angle();
    const ouvert = l.f32();
    const o = coffres.pool.acquerir();
    if (!o) continue;
    o.indexType = type;
    o.x = x;
    o.z = z;
    o.angle = angle;
    o.ouvert = ouvert;
  }
}
