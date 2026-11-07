# Prompt pour la session Claude Code sur le PC du propriétaire

À coller tel quel dans une session Claude Code ouverte **sur le PC**, dans le dossier du
dépôt Block-Survivors (branche `rust` à jour : `git fetch origin rust && git checkout rust
&& git pull origin rust`).

---

Tu travailles sur mon PC, dans le dépôt Block-Survivors (TakeosOfficiel/Block-Survivors),
branche `rust`. La version Rust du jeu (dossier `rust/`) est finie. Il reste les étapes que
seul le propriétaire peut faire, parce qu'elles demandent mes comptes, mon serveur et mon PC.
Fais-les toi-même dans l'ordre ci-dessous, sans me demander la permission à chaque étape.
Arrête-toi seulement quand il faut une action que tu ne peux pas faire (mot de passe, clic
dans un navigateur, achat) : dis-moi alors exactement quoi faire, en une ou deux phrases.

Règles :
- Réponds en français, avec des messages courts.
- Ne pousse **jamais** sur `main` : `main` publie automatiquement la version Electron aux
  joueurs. Tout commit va sur `rust`.
- Pas de pull request.
- Les clés **privées racines** ne vont jamais sur GitHub, ni dans un commit, ni dans le
  chat. Écris-les dans un dossier hors du dépôt (`~/bs-cles-racines/`). Ensuite,
  dis-moi de les copier sur une clé USB et dans mon gestionnaire de mots de passe.
- Lis d'abord `docs/PUBLICATION-RUST.md` et `docs/RELAIS.md`. Ils décrivent tout en détail
  et font foi si ce prompt et eux divergent.
- Pour GitHub, utilise la CLI `gh`. Si elle n'est pas connectée, lance `gh auth login` et
  laisse-moi me connecter. Elle sert pour les variables, les secrets, l'environnement et
  le lancement des workflows.

## 1. Outils

Vérifie que sont installés : `git`, `gh`, Rust (`rustup`, toolchain stable) et Node 20+.
Installe ce qui manque. Puis, depuis `rust/`, lance `cargo build -p block-survivors` pour
vérifier que tout compile.

## 2. Clés de mise à jour

Depuis `rust/` :
1. Crée trois paires de clés avec `cargo run -q -p bs-maj --bin bs-signer -- cles` : racine
   1, racine 2 (secours) et clé de la CI. Range-les dans `~/bs-cles-racines/` (un fichier
   par clé, privée et publique).
2. Délégation d'un an de la racine 1 vers la clé de la CI, série 1 :
   `BS_CLE_PRIVEE=<privée racine 1> cargo run -q -p bs-maj --bin bs-signer -- deleguer <publique CI> 1 365`.
   Garde la ligne JSON qu'elle affiche.

## 3. Réglages GitHub

Dépôt `TakeosOfficiel/Block-Survivors` :
1. Variables du dépôt (`gh variable set`) :
   - `BS_CLES_MAJ` : les deux clés publiques racines, séparées par une virgule ;
   - `BS_DELEGATION` : la ligne JSON de la délégation.
2. Environnement `publication` (`gh api -X PUT repos/TakeosOfficiel/Block-Survivors/environments/publication`)
   avec mon compte comme « Required reviewer ». Si l'API refuse, dis-moi de l'ajouter dans
   Settings → Environments.
3. Secrets de l'environnement `publication` (`gh secret set --env publication`) :
   - `BS_CLE_PRIVEE` : la clé privée de la CI ;
   - `VERSION_TOKEN` : le jeton qui écrit sur `TakeosOfficiel/Block-Survivors-version`.
     Il existe sans doute déjà comme secret pour la version Electron. Sinon, demande-le-moi
     ou guide-moi pour créer un jeton « fine-grained » limité à ce dépôt (Contents :
     read/write).

## 4. Relais sur mon serveur (takeos.fr)

Suis `docs/RELAIS.md`. Le relais sera à `wss://relais.takeos.fr/salon`. Demande-moi l'IP
ou l'hôte SSH du serveur et le compte à utiliser.
1. Sur le PC, crée la clé de déploiement : `ssh-keygen -t ed25519 -f ~/.ssh/relais-deploiement -N ""`.
2. Sur le serveur, en SSH :
   - crée l'utilisateur `deploiement` avec cette clé ;
   - ajoute sa règle `sudoers`, limitée à `install`, `cp`, `mv` et `systemctl restart bs-relais` ;
   - installe le service : `/opt/bs-relais` et `rust/outils/relais/bs-relais.service` ;
   - configure le pare-feu : SSH seulement.
3. Cloudflare Tunnel, si takeos.fr est chez Cloudflare. `cloudflared tunnel login` ouvre
   un navigateur, et c'est moi qui clique. Ensuite, crée le tunnel `bs-relais`, la route
   DNS `relais.takeos.fr`, `/etc/cloudflared/config.yml` vers `http://127.0.0.1:8787`, puis
   `cloudflared service install`. Si takeos.fr n'est pas chez Cloudflare, dis-le-moi et
   propose la solution la plus simple : un reverse proxy Caddy en WSS, avec le relais
   lancé avec `BS_RELAIS_DIRECT=1`.
4. Réglages GitHub :
   - secret `RELAIS_CLE_SSH` (environnement `publication`) : la clé privée de déploiement ;
   - variables `RELAIS_SERVEUR` (`deploiement@<ip>`) et `RELAIS_EMPREINTE_SSH` (sortie de
     `ssh-keyscan -t ed25519 <ip>`), plus `RELAIS_PORT_SSH` si le port n'est pas 22 ;
   - variable `BS_RELAIS` : `wss://relais.takeos.fr/salon`.
5. Déploie : `git tag relais-v1 && git push origin relais-v1`, puis dis-moi de valider le
   déploiement dans l'onglet Actions. Vérifie ensuite que
   `https://relais.takeos.fr/etat` répond en JSON.

## 5. Test sur mon PC (vraie carte graphique)

1. Depuis `rust/` : `cargo run --profile ship -p block-survivors`. Le build est long
   (environ 20 min). Le profil `release` suffit si c'est trop long.
2. Fais-moi jouer une partie, puis une partie en coop à plusieurs si possible (deux
   instances sur le PC suffisent pour tester, avec `BS_RELAIS=wss://relais.takeos.fr/salon`).
3. Relève les lignes `MESURE` du journal (images/s, pire image) en fin de partie.
   Écris-les dans `docs/PROTOTYPE-RUST.md`, section 8c, à la place de « restent à faire ».
   Commit et push sur `rust`.

## 6. Première bêta

1. Mets `version` à `0.1.0` dans `rust/Cargo.toml` (`[workspace.package]`) si ce n'est pas
   déjà fait. Complète les Nouveautés si besoin.
2. `git tag rust-v0.1.0-beta.1 && git push origin rust-v0.1.0-beta.1`.
3. Dis-moi de valider la publication dans Actions (environnement `publication`).
4. Vérifie que les releases `rust-win-beta` et `rust-linux-beta` contiennent l'installeur
   et le manifeste.

## 7. Signature Windows (à part, payante)

Sans signature Authenticode, Windows affiche « éditeur inconnu » (SmartScreen).
Explique-moi les options et leur prix : certificat OV/EV, ou Azure Trusted Signing
(environ 10 $/mois, le plus simple pour un particulier). N'achète rien toi-même. Une fois
que j'ai choisi, ajoute la signature au workflow « Publication Rust » sur la branche `rust`.

## 8. Bascule (seulement quand je le dis)

Ne le fais **pas** maintenant. Quand la bêta me convient et qu'une version stable Rust est
publiée, suis la section « Bascule » de `docs/PUBLICATION-RUST.md`.

## À la fin

Fais-moi un résumé court :
- ce qui est fait ;
- ce qui attend une action de ma part ;
- où sont rangées les clés racines, en me rappelant de les sauvegarder hors ligne.
