# Publier la version Rust

La chaîne est décrite dans le plan (docs/PLAN-RUST.md, section 5). En résumé :
- **Velopack** installe le jeu et le met à jour ;
- un **manifeste signé** (ed25519) dit au jeu quels paquets croire.

Rien n'est publié sur un simple push, sur `main` comme sur `rust`.

## Préparer (une fois)

1. **Clés**, créées sur votre PC (depuis `rust/`) :
   ```
   cargo run -q -p bs-maj --bin bs-signer -- cles   # racine 1
   cargo run -q -p bs-maj --bin bs-signer -- cles   # racine 2 (secours)
   cargo run -q -p bs-maj --bin bs-signer -- cles   # clé de la CI
   ```
   - Les deux clés privées **racines** ne vont jamais sur GitHub : gardez-les hors ligne
     (clé USB, gestionnaire de mots de passe), à deux endroits différents.
   - La clé privée **de la CI** va dans le secret `BS_CLE_PRIVEE` de l'environnement
     `publication`.
   - Une racine autorise la clé de la CI par une **délégation** (ici pour un an) :
     ```
     BS_CLE_PRIVEE=<privée racine 1> cargo run -q -p bs-maj --bin bs-signer -- \
       deleguer <publique CI> 1 365
     ```
2. **Variables du dépôt** (Settings → Secrets and variables → Actions → Variables) :
   - `BS_CLES_MAJ` : les deux clés **publiques racines**, séparées par une virgule. Elles
     sont écrites dans le jeu à la compilation (pas la clé de la CI).
   - `BS_DELEGATION` : la ligne JSON sortie par `deleguer`. Elle est jointe à chaque
     manifeste.
   - `BS_RELAIS` : adresse du relais (docs/RELAIS.md).
3. **Environnement `publication`** (Settings → Environments), avec validation à la main
   (Required reviewers) et ces secrets :
   - `BS_CLE_PRIVEE` ;
   - `VERSION_TOKEN` : le jeton qui écrit sur `TakeosOfficiel/Block-Survivors-version`,
     déjà utilisé par la version Electron.

## Publier une version

1. Changer `version` dans `rust/Cargo.toml` (`[workspace.package]`) et compléter les
   Nouveautés.
2. Créer le tag correspondant et le pousser :
   - voie stable : `rust-v0.2.0` ;
   - voie bêta : `rust-v0.2.0-beta.1` ;
   - par exemple `git tag rust-v0.2.0 && git push origin rust-v0.2.0`.
3. Le workflow « Publication Rust » :
   1. lance toutes les vérifications (format, clippy, tests, robot, chaîne de mise à jour) ;
   2. construit Windows et Linux, puis empaquette avec `vpk` ;
   3. **attend votre validation** ;
   4. signe et publie dans les releases `rust-win-<voie>` et `rust-linux-<voie>` du dépôt
      public. Le manifeste est publié en dernier.
4. Les joueurs reçoivent la version :
   - au lancement du jeu ;
   - ou dans les 30 minutes qui suivent, si le jeu est déjà ouvert.

   Le jeu la télécharge en arrière-plan, puis propose « Redémarrer » au menu. Sinon, elle
   s'installe au lancement suivant.

Installeurs à donner aux nouveaux joueurs (dans ces releases) :
- `BlockSurvivors-win-stable-Setup.exe` ;
- `BlockSurvivors-linux-stable.AppImage`.

## Tous les 60 jours au plus

Un manifeste expire au bout de 60 jours (`VALIDITE`) : ainsi, personne ne peut figer un
joueur sur une vieille version en lui resservant un ancien manifeste. Sans nouvelle version
pendant ce temps, lancez à la main le workflow « Publication Rust » (case « Re-signer »
cochée). Un manifeste expiré n'empêche pas de jouer : le jeu ne trouve simplement plus de
mise à jour.

## Renouveler la délégation

Une délégation expire (365 jours ci-dessus). Avant, refaire `deleguer` avec une **série
plus grande** et remplacer la variable `BS_DELEGATION`. Le jeu retient la plus grande
série vue et refuse ensuite les délégations plus anciennes.

## Si la clé de la CI est volée

1. Créer une nouvelle clé de CI (`bs-signer cles`), la mettre dans le secret
   `BS_CLE_PRIVEE`.
2. Avec une racine, déléguer à cette seule nouvelle clé, avec une série plus grande que la
   précédente, et remplacer `BS_DELEGATION`.
3. Publier (ou re-signer) une version : dès qu'un jeu a vu la nouvelle délégation, il
   refuse l'ancienne, donc la clé volée. Aucun nouveau build n'est nécessaire.

Le voleur ne peut ni changer les clés racines (elles sont dans le build), ni se signer une
délégation. Il garde la main jusqu'à ce que les joueurs aient vu la nouvelle délégation :
c'est pourquoi on publie tout de suite après. Si une **racine** est volée, il faut une
nouvelle version dont `BS_CLES_MAJ` ne contient plus qu'elle-même remplacée, signée avec
l'autre racine (`BS_CLE_PRIVEE=<racine 2>`, sans `BS_DELEGATION`).

## Voies

- **Stable** : tout le monde.
- **Bêta** : les joueurs qui cochent « Versions bêta » dans les Options. Le jeu passe alors
  au canal `<système>-beta`. En décochant, il revient à la dernière stable (seul cas où il
  accepte une version plus ancienne).

## Bascule : passer les joueurs Electron à la version Rust

Quand la version Rust est publiée en **stable** et qu'on veut que tout le monde y passe :

1. Dans `electron/passage.cjs`, mettre `BASCULE = true` et recopier dans `CLES` les clés
   publiques racines de la variable `BS_CLES_MAJ` (la délégation voyage dans le
   manifeste).
2. Publier cette version Electron comme d'habitude (push sur `main`) : c'est la
   **dernière**.
3. Au lancement suivant, chez chaque joueur Electron :
   1. l'appli lit et vérifie le manifeste Rust de son système (signature, expiration) ;
   2. elle écrit chaque profil en `.bssave` dans `depuis-electron/` des sauvegardes Rust ;
   3. elle télécharge l'installeur, vérifie sa taille et son SHA-256 (signés dans le
      manifeste, liste `installeurs`), puis :
      - sous Windows, désinstalle Electron sans effacer ses données, puis lance
        l'installeur Rust ;
      - sous Linux, remplace l'AppImage par celle de la version Rust.
4. La version Rust importe ces profils à son premier lancement, et l'ancien profil actif
   le redevient.

En cas d'échec (hors ligne, manifeste refusé…), le jeu Electron s'ouvre comme d'habitude
et réessaie au lancement suivant. Le dossier de données d'Electron n'est jamais effacé.
