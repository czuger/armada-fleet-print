#!/usr/bin/env bash
# Go back to the previously deployed version, in one command: make rollback
#
# Purely server-side: the previous binary is still in releases/, so this builds and
# transfers nothing. The two version files trade places, which means running it
# twice returns you to where you started. The card image cache is shared by all
# versions and left as is.
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

echo "==> Retour arrière sur $SSH_HOST"
upload_remote_script
remote versions
echo

# Uploaded every time so deploy and rollback start the app the same way.
upload_systemd_unit

if ! remote rollback; then
  echo >&2
  echo "erreur : le retour arrière a échoué — voir 'make status' et 'make logs'" >&2
  exit 1
fi

echo
remote versions
