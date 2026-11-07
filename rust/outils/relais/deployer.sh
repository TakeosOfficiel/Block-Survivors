#!/usr/bin/env bash
# Déploie le relais sur le serveur par SSH (docs/RELAIS.md) : copie le binaire à côté de
# l'ancien, bascule d'un coup et redémarre le service ; l'ancien reste (bs-relais.ancien)
# pour revenir en arrière. Les salons en cours sont coupés : les joueurs se reconnectent
# seuls (60 s), mais une partie en cours chez un hôte est perdue si l'hôte ne revient pas.
#   deployer.sh <binaire> <utilisateur@serveur> [port SSH]
set -euo pipefail
BINAIRE=$1 SERVEUR=$2 PORT=${3:-22}
scp -P "$PORT" -q "$BINAIRE" "$SERVEUR:/tmp/bs-relais.nouveau"
ssh -p "$PORT" "$SERVEUR" 'set -e
  sudo install -m 755 /tmp/bs-relais.nouveau /opt/bs-relais/bs-relais.nouveau
  [ -f /opt/bs-relais/bs-relais ] && sudo cp /opt/bs-relais/bs-relais /opt/bs-relais/bs-relais.ancien
  sudo mv /opt/bs-relais/bs-relais.nouveau /opt/bs-relais/bs-relais
  sudo systemctl restart bs-relais
  sleep 2
  curl -sf http://127.0.0.1:8787/etat'
echo
echo "relais déployé sur $SERVEUR"
