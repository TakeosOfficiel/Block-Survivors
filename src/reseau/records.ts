/** Records des parties à plusieurs (temps tenu par arène, parties, boss), à part de ceux du solo. */
import { Sauvegarde, type BilanPartie } from '../progression/sauvegarde';
import { cleProfil } from '../profils';

export function enregistrerRecordsEquipe(bilan: BilanPartie): void {
  Sauvegarde.charger(cleProfil('progression.multi')).terminerPartie({ ...bilan, xp: 0, enEquipe: false });
}
