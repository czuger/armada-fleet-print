#!/usr/bin/env bash
# ONE-TIME server preparation. Run from your laptop: make setup
#
# Creates the directory layout, checks the prerequisites that only an admin can fix
# (systemd lingering, the socket directory mounted into nginx), then installs the
# nginx snippet. Safe to re-run: it never touches data/.
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

echo "==> Cible : $SSH_HOST:$REMOTE_DIR"

# 1. Tools the server side relies on.
echo "==> Vérification des outils sur le serveur"
for tool in systemctl loginctl journalctl curl docker; do
  ssh "$SSH_HOST" "command -v $tool >/dev/null" \
    || { echo "erreur : $tool est introuvable sur $SSH_HOST" >&2; exit 1; }
done
ssh "$SSH_HOST" 'docker info >/dev/null 2>&1' || {
  echo "erreur : 'docker info' échoue sur $SSH_HOST — l'utilisateur SSH doit être dans le groupe docker." >&2
  exit 1
}

# 2. Directory layout + lingering + socket directory. `init` explains what to do by
#    hand if a prerequisite is missing, and stops there.
echo "==> Préparation de $REMOTE_DIR et vérification des prérequis"
upload_remote_script
remote init

# 3. The nginx snippet: rendered from deploy/nginx/, dropped in the proxy's config
#    directory, validated there, and only then does nginx reload. Setup-only —
#    routing does not change from one deploy to the next.
echo "==> Installation de la configuration nginx ($NGINX_CONF_NAME -> $NGINX_CONF_DIR)"
install_nginx_conf

cat <<EOT

Serveur prêt : $SSH_HOST:$REMOTE_DIR
  releases/   binaires déployés (les $KEEP_RELEASES derniers sont conservés)
  current     lien vers le binaire en service
  data/pics/  cache des images de cartes, commun à toutes les versions

L'application n'ouvre aucun port : elle écoute sur $SOCKET_DIR/$SOCKET_NAME,
que nginx voit en $NGINX_SOCKET_DIR/$SOCKET_NAME et sert sous $URL_PREFIX/.

Ensuite : make deploy
EOT
