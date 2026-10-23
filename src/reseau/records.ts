/** Records des parties à plusieurs (temps tenu par arène, parties, boss), à part de ceux du solo. */
import { CLE_MULTIJOUEUR, Sauvegarde, type BilanPartie } from '../progression/sauvegarde';

export function enregistrerRecordsEquipe(bilan: BilanPartie): void {
  Sauvegarde.charger(CLE_MULTIJOUEUR).terminerPartie({ ...bilan, xp: 0, enEquipe: false });
}
