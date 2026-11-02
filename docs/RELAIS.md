# Relais de coop : déploiement

Le relais (`rust/crates/bs-relais`) fait suivre les messages entre l'hôte d'une partie et
ses invités. Il ne simule rien. Le jeu publié le trouve à l'adresse écrite dans son build :
la variable de dépôt `BS_RELAIS` (par exemple `wss://relais.exemple.fr/salon`).

## Ce qu'il faut

- Un petit serveur Linux en Europe (1 vCPU, 512 Mo à 1 Go suffisent pour commencer). Le
  coût dépend du trafic : chaque instantané (15 par seconde) est reçu une fois, puis réémis
  vers chaque invité.
- Un domaine géré par Cloudflare (offre gratuite).

## Installation (une fois)

1. **Utilisateur de déploiement**, avec une clé SSH dédiée (créée sur votre PC :
   `ssh-keygen -t ed25519 -f relais-deploiement`). Il doit pouvoir lancer `sudo install`,
   `cp`, `mv` et `systemctl restart bs-relais` sans mot de passe (règle `sudoers`
   limitée à ces commandes).
2. **Le service** :
   ```
   sudo mkdir -p /opt/bs-relais
   sudo cp rust/outils/relais/bs-relais.service /etc/systemd/system/
   sudo systemctl daemon-reload && sudo systemctl enable bs-relais
   ```
   Le relais écoute seulement en local (`127.0.0.1:8787`) : aucun port n'est ouvert.
3. **Cloudflare Tunnel** (`cloudflared`), qui expose le relais en HTTPS/WSS :
   ```
   cloudflared tunnel login
   cloudflared tunnel create bs-relais
   cloudflared tunnel route dns bs-relais relais.exemple.fr
   ```
   Dans `/etc/cloudflared/config.yml` :
   ```
   tunnel: bs-relais
   credentials-file: /etc/cloudflared/<id du tunnel>.json
   ingress:
     - hostname: relais.exemple.fr
       service: http://127.0.0.1:8787
     - service: http_status:404
   ```
   Puis `sudo cloudflared service install`. Cloudflare transmet l'adresse des joueurs dans
   `CF-Connecting-IP` ; le relais s'en sert pour ses limites (salons, tentatives de code).
   Exposé directement, sans Cloudflare, il faudrait le lancer avec `BS_RELAIS_DIRECT=1`.
4. **Pare-feu** : SSH seulement (`ufw allow OpenSSH && ufw enable`).

## Réglages du dépôt GitHub

Dans Settings → Environments, l'environnement **`publication`** doit demander une
validation à la main (Required reviewers). Il porte les secrets :

| Nom | Sorte | Valeur |
|---|---|---|
| `RELAIS_CLE_SSH` | secret de l'environnement `publication` | clé privée de déploiement (`relais-deploiement`) |
| `RELAIS_SERVEUR` | variable du dépôt | `deploiement@ip-du-serveur` |
| `RELAIS_PORT_SSH` | variable du dépôt (facultatif) | port SSH, 22 par défaut |
| `RELAIS_EMPREINTE_SSH` | variable du dépôt | ligne de `ssh-keyscan -t ed25519 ip-du-serveur` (vérifiée hors ligne) |
| `BS_RELAIS` | variable du dépôt | `wss://relais.exemple.fr/salon`, écrit dans le jeu publié |

## Déployer

Pousser un tag `relais-vN` (ou lancer le workflow « Relais » à la main) :
1. tests du relais ;
2. build Linux ;
3. après validation, copie par SSH (`outils/relais/deployer.sh`).

Le script bascule d'un coup, redémarre le service et lit `/etat`. L'ancien binaire reste
dans `/opt/bs-relais/bs-relais.ancien` pour revenir en arrière.

**Serveur actuel (`5.39.102.138`) :** son port SSH est filtré et GitHub ne peut pas le
joindre. L'étape de déploiement du workflow échoue donc (`scp: Connection closed` au bout
d'environ 2 min). Le build reste bon : il faut déployer depuis le PC autorisé, dans Git Bash,
avec la clé `~/.ssh/relais-deploiement` chargée (`ssh-add`) et l'empreinte du serveur dans
`~/.ssh/known_hosts` :
```
gh run download <id du run> -n bs-relais -D relais-binaire   # hors du dépôt
rust/outils/relais/deployer.sh relais-binaire/bs-relais deploiement@5.39.102.138
```
Ce serveur diffère aussi de l'installation décrite plus haut :
- **Apache au lieu de Cloudflare Tunnel.** Apache expose le relais en HTTPS/WSS et lui
  donne la vraie adresse des joueurs avec `RequestHeader set CF-Connecting-IP
  "%{REMOTE_ADDR}s"`. `BS_RELAIS_DIRECT` reste donc inutile.
- **Conteneur LXC sans nesting.** L'isolation de `bs-relais.service` échoue
  (`226/NAMESPACE`) : le service a besoin du fichier
  `/etc/systemd/system/bs-relais.service.d/lxc.conf`, qui n'existe que sur le serveur.

Redémarrer coupe les salons en cours. Les joueurs se reconnectent seuls (place gardée
60 s), mais les salons eux-mêmes sont perdus. On déploie donc le relais seulement quand son
protocole ou ses limites changent, et à une heure creuse.

## Surveillance

`https://relais.exemple.fr/etat` répond en JSON : protocole, salons, salons publics,
joueurs, places réservées, connexions. Un service de surveillance gratuit (UptimeRobot,
healthchecks.io…) peut l'interroger toutes les 5 minutes et prévenir par e-mail s'il ne
répond plus.

## Versions

Le relais n'accepte que les versions du protocole qu'il connaît (`Reglages::versions`). Si
le protocole change (`VERSION_PROTOCOLE`), il faut déployer le nouveau relais **avant** de
publier le jeu qui l'utilise, et garder l'ancien protocole accepté le temps que tout le
monde se mette à jour.
