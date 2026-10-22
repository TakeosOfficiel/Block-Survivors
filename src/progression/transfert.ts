/**
 * Transfert de la sauvegarde d'une version du jeu à l'autre (navigateur,
 * Windows, Linux) : chacune garde ses données dans son propre stockage local,
 * que les autres ne voient pas. « Exporter » écrit toutes les clés
 * « block-survivors.* » (progression, profil et skin importée, options…) dans
 * un fichier ; « Importer » les remet, puis le jeu se recharge pour les
 * prendre en compte.
 *
 * Le fichier est chiffré (AES-GCM) : illisible, et la moindre retouche le
 * fait refuser à l'import (GCM vérifie l'intégrité). La clé est dans le jeu :
 * ça empêche de bricoler un fichier, pas un tricheur décidé.
 */
import { VERSION } from '../version';

const PREFIXE = 'block-survivors.';
const FORMAT = 'block-survivors-sauvegarde';
/** En-tête du fichier, puis base64 de : vecteur d'initialisation (12 octets) + données chiffrées. */
const ENTETE = 'BSSAVE1:';
const SECRET = 'Block Survivors · sauvegarde · 7c1e5a93d2f84b60';
const TAILLE_IV = 12;
const ILLISIBLE = 'Ce fichier n\'est pas une sauvegarde de Block Survivors, ou il a été modifié.';

/** Clé AES-GCM tirée du secret du jeu. */
async function cle(): Promise<CryptoKey> {
  const brut = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(SECRET));
  return crypto.subtle.importKey('raw', brut, 'AES-GCM', false, ['encrypt', 'decrypt']);
}

async function chiffrer(texte: string): Promise<string> {
  const iv = crypto.getRandomValues(new Uint8Array(TAILLE_IV));
  const chiffre = new Uint8Array(await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, await cle(), new TextEncoder().encode(texte)));
  const tout = new Uint8Array(TAILLE_IV + chiffre.length);
  tout.set(iv);
  tout.set(chiffre, TAILLE_IV);
  let binaire = '';
  for (let i = 0; i < tout.length; i++) binaire += String.fromCharCode(tout[i]);
  return ENTETE + btoa(binaire);
}

/** Texte déchiffré, ou null si le fichier n'est pas une sauvegarde ou a été retouché. */
async function dechiffrer(contenu: string): Promise<string | null> {
  if (!contenu.startsWith(ENTETE)) return null;
  try {
    const binaire = atob(contenu.slice(ENTETE.length).trim());
    const tout = Uint8Array.from(binaire, (c) => c.charCodeAt(0));
    const clair = await crypto.subtle.decrypt({ name: 'AES-GCM', iv: tout.subarray(0, TAILLE_IV) }, await cle(), tout.subarray(TAILLE_IV));
    return new TextDecoder().decode(clair);
  } catch {
    return null;
  }
}

interface FichierSauvegarde {
  format: string;
  version: string;
  date: string;
  /** Valeur brute (JSON) de chaque clé du stockage. */
  donnees: Record<string, string>;
}

/** Télécharge la sauvegarde complète dans un fichier chiffré. */
export async function exporterSauvegarde(): Promise<void> {
  const donnees: Record<string, string> = {};
  try {
    for (let i = 0; i < window.localStorage.length; i++) {
      const cle = window.localStorage.key(i);
      const valeur = cle?.startsWith(PREFIXE) ? window.localStorage.getItem(cle) : null;
      if (cle && valeur !== null) donnees[cle] = valeur;
    }
  } catch {
    // Stockage indisponible : fichier vide
  }
  const fichier: FichierSauvegarde = { format: FORMAT, version: VERSION, date: new Date().toISOString(), donnees };
  const url = URL.createObjectURL(new Blob([await chiffrer(JSON.stringify(fichier))], { type: 'application/octet-stream' }));
  const lien = document.createElement('a');
  lien.href = url;
  lien.download = `block-survivors-sauvegarde-${new Date().toISOString().slice(0, 10)}.bssave`;
  document.body.append(lien);
  lien.click();
  lien.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}

/** Remet la sauvegarde d'un fichier exporté ; renvoie le message d'erreur, ou null si tout est en place. */
export async function importerSauvegarde(fichier: File): Promise<string | null> {
  const clair = await dechiffrer(await fichier.text());
  if (clair === null) return ILLISIBLE;
  let lu: Partial<FichierSauvegarde>;
  try {
    lu = JSON.parse(clair) as Partial<FichierSauvegarde>;
  } catch {
    return ILLISIBLE;
  }
  if (lu.format !== FORMAT || !lu.donnees || typeof lu.donnees !== 'object') return ILLISIBLE;
  const entrees = Object.entries(lu.donnees).filter(([cle, valeur]) => cle.startsWith(PREFIXE) && typeof valeur === 'string');
  if (entrees.length === 0) return 'Cette sauvegarde est vide.';
  try {
    for (const [cle, valeur] of entrees) window.localStorage.setItem(cle, valeur);
  } catch {
    return 'Impossible d\'enregistrer la sauvegarde (stockage plein ou bloqué).';
  }
  return null;
}
