/**
 * Transfert d'un profil d'une version du jeu à l'autre (navigateur, Windows,
 * Linux), ou d'un ordinateur à l'autre : chacune garde ses données dans son
 * propre stockage local, que les autres ne voient pas. « Exporter » écrit le
 * profil actif (pseudo et skin importée, progression, Collection…) dans un
 * fichier ; « Importer » en fait un nouveau profil, puis le jeu se recharge
 * dessus. Les fichiers d'avant les profils (toute la sauvegarde) s'importent
 * aussi.
 *
 * Le fichier est chiffré (AES-GCM) : illisible, et la moindre retouche le
 * fait refuser à l'import (GCM vérifie l'intégrité). La clé est dans le jeu :
 * ça empêche de bricoler un fichier, pas un tricheur décidé.
 */
import { chargerProfil } from '../profil';
import {
  creerProfil, DONNEES_PROFIL, ecrireDonneeBrute, lireDonneeBrute, peutCreerProfil, profilActif, type DonneeProfil,
} from '../profils';
import { VERSION } from '../version';

/** Préfixe des clés dans les fichiers d'avant les profils. */
const PREFIXE = 'block-survivors.';
/** Fichiers d'avant les profils (toute la sauvegarde), puis d'un profil. */
const FORMAT = 'block-survivors-sauvegarde';
const FORMAT_PROFIL = 'block-survivors-profil';
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

/** Fichier d'un profil (depuis les profils). */
interface FichierProfil {
  format: typeof FORMAT_PROFIL;
  version: string;
  date: string;
  /** Valeur brute (JSON) de chaque donnée du profil, par nom (voir DONNEES_PROFIL). */
  donnees: Partial<Record<DonneeProfil, string>>;
}

/** Fichier d'avant les profils : toutes les clés « block-survivors.* » du stockage. */
interface FichierAncien {
  format: typeof FORMAT;
  donnees: Record<string, string>;
}

/** Télécharge le profil actif dans un fichier chiffré. */
export async function exporterProfil(): Promise<void> {
  const id = profilActif();
  const donnees: Partial<Record<DonneeProfil, string>> = {};
  for (const nom of DONNEES_PROFIL) {
    const valeur = lireDonneeBrute(id, nom);
    if (valeur !== null) donnees[nom] = valeur;
  }
  const fichier: FichierProfil = { format: FORMAT_PROFIL, version: VERSION, date: new Date().toISOString(), donnees };
  const pseudo = chargerProfil().pseudo.replace(/[^\p{L}\p{N}_-]+/gu, '_') || 'profil';
  const url = URL.createObjectURL(new Blob([await chiffrer(JSON.stringify(fichier))], { type: 'application/octet-stream' }));
  const lien = document.createElement('a');
  lien.href = url;
  lien.download = `block-survivors-${pseudo}-${new Date().toISOString().slice(0, 10)}.bssave`;
  document.body.append(lien);
  lien.click();
  lien.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}

/**
 * Crée un nouveau profil (devenu l'actif) à partir d'un fichier exporté, y
 * compris d'avant les profils ; renvoie le message d'erreur, ou null.
 */
export async function importerProfil(fichier: File): Promise<string | null> {
  if (!peutCreerProfil()) return 'Il y a déjà le nombre maximal de profils : supprimez-en un d\'abord.';
  const clair = await dechiffrer(await fichier.text());
  if (clair === null) return ILLISIBLE;
  let lu: Partial<FichierProfil> | Partial<FichierAncien>;
  try {
    lu = JSON.parse(clair) as Partial<FichierProfil> | Partial<FichierAncien>;
  } catch {
    return ILLISIBLE;
  }
  if (!lu.donnees || typeof lu.donnees !== 'object') return ILLISIBLE;
  const donnees: Partial<Record<DonneeProfil, string>> = {};
  for (const nom of DONNEES_PROFIL) {
    const valeur = lu.format === FORMAT_PROFIL
      ? (lu.donnees as Record<string, unknown>)[nom]
      : lu.format === FORMAT ? (lu.donnees as Record<string, unknown>)[PREFIXE + nom] : undefined;
    if (typeof valeur === 'string') donnees[nom] = valeur;
  }
  if (Object.keys(donnees).length === 0) return lu.format === FORMAT || lu.format === FORMAT_PROFIL ? 'Cette sauvegarde est vide.' : ILLISIBLE;
  try {
    creerProfil((id) => {
      for (const [nom, valeur] of Object.entries(donnees) as [DonneeProfil, string][]) ecrireDonneeBrute(id, nom, valeur);
    });
  } catch {
    return 'Impossible d\'enregistrer le profil (stockage plein ou bloqué).';
  }
  return null;
}
