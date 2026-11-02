# Publier la version Rust

La chaîne est décrite dans le plan (docs/PLAN-RUST.md, section 5). En résumé :
- **Velopack** installe le jeu et le met à jour ;
- un **manifeste signé** (ed25519) dit au jeu quels paquets croire.

Rien n'est publié sur un simple push, sur `main` comme sur `rust`.

## Préparer (une fois)

1. **Deux paires de clés**, créées sur votre PC (depuis `rust/`) :
   ```
   cargo run -q -p bs-maj --bin bs-signer -- cles   # clé de la CI
   cargo run -q -p bs-maj --bin bs-signer -- cles   # clé de secours
   ```
   - La clé privée **de la CI** va dans le secret `BS_CLE_PRIVEE` de l'environnement
     `publication`.
   - La clé privée **de secours** ne va jamais sur GitHub : gardez-la hors ligne (clé USB,
     gestionnaire de mots de passe).
2. **Variables du dépôt** (Settings → Secrets and variables → Actions → Variables) :
   - `BS_CLES_MAJ` : les deux clés **publiques**, séparées par une virgule (CI, secours).
     Elles sont écrites dans le jeu à la compilation.
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

## Si la clé de la CI est volée

1. Signer un manifeste à la main avec la **clé de secours**
   (`BS_CLE_PRIVEE=<secours> bs-signer manifeste …`) pour une version dont le build ne
   contient plus que la clé de secours et une nouvelle clé de CI.
2. Remplacer le secret `BS_CLE_PRIVEE` et la variable `BS_CLES_MAJ`.

Limite actuelle : le jeu accepte les deux clés à égalité. Avec la clé de la CI, un voleur
pourrait donc lui aussi publier une version qui retire la clé de secours. Réserver ce
pouvoir à la clé de secours demande des manifestes à deux niveaux (comme TUF). C'est à
faire avant la sortie de la bêta.

## Voies

- **Stable** : tout le monde.
- **Bêta** : les joueurs qui cochent « Versions bêta » dans les Options. Le jeu passe alors
  au canal `<système>-beta`. En décochant, il revient à la dernière stable (seul cas où il
  accepte une version plus ancienne).

## Bascule : passer les joueurs Electron à la version Rust

Quand la version Rust est publiée en **stable** et qu'on veut que tout le monde y passe :

1. Dans `electron/passage.cjs`, mettre `BASCULE = true` et recopier dans `CLES` les clés
   publiques de la variable `BS_CLES_MAJ` (même ordre).
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
