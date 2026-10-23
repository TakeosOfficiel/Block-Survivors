/**
 * Profils : plusieurs joueurs (ou plusieurs départs) sur la même installation.
 * Chaque profil a son pseudo et son apparence (profil.ts), sa progression
 * (maîtrise, talents, records, déblocages), sa Collection, ses records à
 * plusieurs et son dernier choix de partie, rangés sous ses propres clés :
 * « block-survivors.p.<id>.<nom> ». Les options (volume, vue…) restent
 * communes à l'installation.
 *
 * Le profil actif est fixé au chargement de la page : en changer recharge le
 * jeu, qui relit tout. Au premier lancement de cette version, la sauvegarde
 * d'avant les profils devient le premier profil, sans rien perdre.
 */
import { ecrireStockage, lireStockage } from './options';

/** Données propres à un profil (nom de leur clé dans le profil). */
export const DONNEES_PROFIL = ['profil', 'progression', 'progression.multi', 'collection', 'choix'] as const;
export type DonneeProfil = (typeof DONNEES_PROFIL)[number];

const CLE_REGISTRE = 'block-survivors.profils';
const PREFIXE_ANCIEN = 'block-survivors.';
const MAX_PROFILS = 8;

interface Registre {
  version: 1;
  actif: string;
  ids: string[];
}

let registre: Registre | null = null;

/** Clé du stockage d'une donnée du profil actif. */
export function cleProfil(nom: DonneeProfil, id = profilActif()): string {
  return `block-survivors.p.${id}.${nom}`;
}

export function profilActif(): string {
  return lireRegistre().actif;
}

/** Identifiants des profils, dans l'ordre de création. */
export function idsProfils(): readonly string[] {
  return lireRegistre().ids;
}

export function peutCreerProfil(): boolean {
  return lireRegistre().ids.length < MAX_PROFILS;
}

/** Crée un profil vide (ses données, s'il y en a, déjà écrites par `remplir`) et le rend actif. */
export function creerProfil(remplir?: (id: string) => void): string {
  const r = lireRegistre();
  let n = r.ids.length + 1;
  while (r.ids.includes(String(n))) n++;
  const id = String(n);
  remplir?.(id);
  r.ids.push(id);
  r.actif = id;
  ecrireStockage(CLE_REGISTRE, r);
  return id;
}

export function changerProfil(id: string): void {
  const r = lireRegistre();
  if (!r.ids.includes(id)) return;
  r.actif = id;
  ecrireStockage(CLE_REGISTRE, r);
}

/** Supprime un profil et toutes ses données (jamais le dernier) ; l'actif passe au premier restant. */
export function supprimerProfil(id: string): void {
  const r = lireRegistre();
  if (r.ids.length <= 1 || !r.ids.includes(id)) return;
  for (const nom of DONNEES_PROFIL) retirerStockage(cleProfil(nom, id));
  r.ids = r.ids.filter((autre) => autre !== id);
  if (r.actif === id) r.actif = r.ids[0];
  ecrireStockage(CLE_REGISTRE, r);
}

/** Valeur brute (JSON) d'une donnée d'un profil, ou null. */
export function lireDonneeBrute(id: string, nom: DonneeProfil): string | null {
  try {
    return window.localStorage.getItem(cleProfil(nom, id));
  } catch {
    return null;
  }
}

export function ecrireDonneeBrute(id: string, nom: DonneeProfil, valeur: string): void {
  window.localStorage.setItem(cleProfil(nom, id), valeur);
}

/** Recharge le jeu sur le menu (après un changement de profil ou un import). */
export function relancerSurLeMenu(): void {
  window.location.replace(window.location.href.split('?')[0]);
}

function lireRegistre(): Registre {
  if (registre) return registre;
  const lu = lireStockage<Partial<Registre>>(CLE_REGISTRE);
  const ids = Array.isArray(lu?.ids) ? lu.ids.filter((id): id is string => typeof id === 'string' && /^\d+$/.test(id)) : [];
  if (ids.length > 0) {
    registre = { version: 1, actif: typeof lu?.actif === 'string' && ids.includes(lu.actif) ? lu.actif : ids[0], ids };
    return registre;
  }
  // Premier lancement avec les profils : la sauvegarde d'avant devient le profil 1
  registre = { version: 1, actif: '1', ids: ['1'] };
  for (const nom of DONNEES_PROFIL) {
    const ancienne = PREFIXE_ANCIEN + nom;
    try {
      const valeur = window.localStorage.getItem(ancienne);
      if (valeur === null) continue;
      window.localStorage.setItem(cleProfil(nom, '1'), valeur);
      window.localStorage.removeItem(ancienne);
    } catch {
      // Stockage indisponible : profil vide pour cette session
    }
  }
  ecrireStockage(CLE_REGISTRE, registre);
  return registre;
}

function retirerStockage(cle: string): void {
  try {
    window.localStorage.removeItem(cle);
  } catch {
    // Stockage indisponible : rien à retirer
  }
}
