/** Courbe d'expérience et montées de niveau. */
import { PROGRESSION } from '../donnees/progression';
import type { EtatPartie } from '../etat';

/** XP nécessaire pour passer du niveau `niveau` au suivant. */
export function xpPourNiveau(niveau: number): number {
  let xp = PROGRESSION.xpPremierNiveau;
  let n = 1;
  for (const palier of PROGRESSION.paliers) {
    while (n < niveau && n < palier.jusquAuNiveau) {
      xp += palier.increment;
      n++;
    }
  }
  return xp;
}

/** Ajoute de l'XP ; chaque niveau franchi s'ajoute aux cartes en attente. */
export function gagnerXp(etat: EtatPartie, valeur: number): void {
  const j = etat.joueur;
  j.xp += valeur;
  while (j.xp >= j.xpNiveau) {
    j.xp -= j.xpNiveau;
    j.niveau++;
    j.xpNiveau = xpPourNiveau(j.niveau);
    etat.niveauxEnAttente++;
  }
}
