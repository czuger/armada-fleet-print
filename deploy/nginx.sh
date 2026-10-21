#!/usr/bin/env bash
# Re-render and re-install the nginx snippet on its own: make nginx
#
# Same code path as the last step of `make setup` — use it after editing
# deploy/nginx/*.template or changing the prefix/socket in deploy/config.sh. The
# server validates the file before nginx reloads, and drops it if it does not parse.
#
# Changing the prefix or socket also changes the systemd unit: redeploy afterwards.
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

echo "==> Installation de $NGINX_CONF_NAME sur $SSH_HOST:$NGINX_CONF_DIR"
install_nginx_conf
