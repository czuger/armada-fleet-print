#!/usr/bin/env bash
# THE SERVER SIDE of the deploy. Never run this from your laptop.
#
# The local scripts upload this file before every run and then call one of its
# subcommands over SSH, so that how the app is (re)started lives in exactly one
# place: a rollback must restart the app exactly like a deploy does.
#
# Usage: bash remote.sh <init | nginx_reload | activate VERSION | rollback
#                        | prune | status | health | versions>
set -euo pipefail

# Everything comes from deploy/config.sh through the environment: no server detail
# is written in the repository.
: "${REMOTE_DIR:?REMOTE_DIR non défini}"
: "${SOCKET_DIR:?SOCKET_DIR non défini}"
: "${NGINX_CONTAINER:?NGINX_CONTAINER non défini}"
: "${NGINX_CONF_DIR:?NGINX_CONF_DIR non défini}"
SERVICE_NAME="${SERVICE_NAME:-armada-fleet-print}"
KEEP_RELEASES="${KEEP_RELEASES:-3}"
NGINX_SOCKET_DIR="${NGINX_SOCKET_DIR:-/sockets}"
SOCKET_NAME="${SOCKET_NAME:-armada-fleet-print.sock}"
NGINX_CONF_NAME="${NGINX_CONF_NAME:-armada-fleet-print.conf}"

RELEASES_DIR="$REMOTE_DIR/releases"
DATA_DIR="$REMOTE_DIR/data"
CARDS_DIR="$DATA_DIR/pics"
CURRENT_LINK="$REMOTE_DIR/current"
CURRENT_FILE="$REMOTE_DIR/current_version.txt"
PREVIOUS_FILE="$REMOTE_DIR/previous_version.txt"
SOCKET_PATH="$SOCKET_DIR/$SOCKET_NAME"
UNIT="$SERVICE_NAME.service"

die() { echo "erreur : $*" >&2; exit 1; }

read_version() { [ -f "$1" ] && cat "$1" || true; }

# --- prerequisites -----------------------------------------------------------

# Without lingering, the user's systemd instance — and the app with it — stops
# when the last SSH session closes.
check_linger() {
  [ "$(loginctl show-user "$USER" -p Linger --value 2>/dev/null)" = "yes" ] || die \
    "le lingering systemd n'est pas activé pour $USER : l'application s'arrêterait à la
fin de la session SSH. Activez-le une fois avec :  sudo loginctl enable-linger $USER"
}

# The socket directory belongs to the proxy, which mounts it — we only use it.
# Deliberately NOT created here: if it is missing when nginx's compose starts,
# docker creates it as root and the app can no longer write its socket.
check_socket_dir() {
  [ -d "$SOCKET_DIR" ] || die "$SOCKET_DIR n'existe pas.$(socket_dir_help)"
  [ -w "$SOCKET_DIR" ] || die "$SOCKET_DIR n'est pas accessible en écriture pour $USER."

  local mounted
  mounted="$(docker inspect "$NGINX_CONTAINER" \
    --format '{{range .Mounts}}{{if eq .Source "'"$SOCKET_DIR"'"}}{{.Destination}}{{end}}{{end}}' \
    2>/dev/null)" || die "aucun conteneur nommé '$NGINX_CONTAINER' — le proxy tourne-t-il ?"
  [ "$mounted" = "$NGINX_SOCKET_DIR" ] || die \
    "$NGINX_CONTAINER ne monte pas $SOCKET_DIR sur $NGINX_SOCKET_DIR (actuellement : '${mounted:-rien}').$(socket_dir_help)"
}

socket_dir_help() {
  cat <<HELP

Dans le projet du proxy nginx, une seule fois :
  mkdir -p $SOCKET_DIR        (en tant que $USER, AVANT de relancer le proxy)
  docker-compose.yml, service nginx, volumes :
    - $SOCKET_DIR:$NGINX_SOCKET_DIR:ro
  docker compose up -d        (recrée le conteneur avec le nouveau montage)
HELP
}

# --- service -----------------------------------------------------------------

# Point `current` at a version and (re)start the unit on it. This is the single
# source of truth for how the app is started; the unit itself was just uploaded.
start_service() {
  local version="$1"
  local binary="$RELEASES_DIR/$version"

  [ -x "$binary" ] || die "$binary est absent ou non exécutable"
  check_linger
  check_socket_dir

  ln -sfn "$binary" "$CURRENT_LINK"
  systemctl --user daemon-reload
  systemctl --user enable "$UNIT" >/dev/null 2>&1
  systemctl --user restart "$UNIT"
  echo "$UNIT démarré sur la version $version"
}

# Probes the app through its socket, as nginx does: the form page needs no network.
probe_health() {
  curl -fsS --max-time 3 --unix-socket "$SOCKET_PATH" -o /dev/null http://localhost/
}

# The app may answer on the host while nginx still cannot see the socket (mount
# missing or pointing elsewhere): check from inside the container too.
probe_nginx_view() {
  docker exec "$NGINX_CONTAINER" test -S "$NGINX_SOCKET_DIR/$SOCKET_NAME" 2>/dev/null
}

# Poll until the app answers (it starts in well under a second, but the restart
# is asynchronous). Returns non-zero if it never does.
wait_for_health() {
  local attempts="${1:-15}"
  for _ in $(seq 1 "$attempts"); do
    if probe_health 2>/dev/null; then
      echo "contrôle de santé ok (/ via $SOCKET_PATH)"
      probe_nginx_view && echo "socket visible depuis $NGINX_CONTAINER ($NGINX_SOCKET_DIR/$SOCKET_NAME)" \
        || echo "attention : $NGINX_CONTAINER ne voit pas $NGINX_SOCKET_DIR/$SOCKET_NAME" >&2
      return 0
    fi
    sleep 1
  done
  echo "contrôle de santé ÉCHOUÉ (/ via $SOCKET_PATH)" >&2
  journalctl --user -u "$UNIT" --no-pager -n 40 >&2 || true
  return 1
}

# --- subcommands -------------------------------------------------------------

# Switch to the freshly uploaded binary and record the version swap.
cmd_activate() {
  local version="${1:?activate attend une version}"

  chmod +x "$RELEASES_DIR/$version"

  # Version bookkeeping happens *before* the restart: if the new version fails to
  # come up, previous_version.txt already points at what to roll back to.
  local current
  current="$(read_version "$CURRENT_FILE")"
  if [ -n "$current" ] && [ "$current" != "$version" ]; then
    echo "$current" > "$PREVIOUS_FILE"
  fi
  echo "$version" > "$CURRENT_FILE"

  start_service "$version"
  wait_for_health
}

# Swap back to previous_version.txt. The two version files trade places, so a
# second rollback returns to where you started.
cmd_rollback() {
  local current previous
  current="$(read_version "$CURRENT_FILE")"
  previous="$(read_version "$PREVIOUS_FILE")"

  [ -n "$previous" ] || die "aucune version précédente enregistrée — rien à restaurer"
  [ "$previous" != "$current" ] || die "la version précédente est la version actuelle ($current)"

  start_service "$previous"

  echo "$previous" > "$CURRENT_FILE"
  if [ -n "$current" ]; then
    echo "$current" > "$PREVIOUS_FILE"
  else
    rm -f "$PREVIOUS_FILE"
  fi

  echo "retour à $previous (à la place de $current)"
  wait_for_health
}

# Keep the $KEEP_RELEASES newest binaries; never touch what runs or what rollback needs.
cmd_prune() {
  local current previous kept=0
  current="$(read_version "$CURRENT_FILE")"
  previous="$(read_version "$PREVIOUS_FILE")"

  for binary in $(ls -1t "$RELEASES_DIR" 2>/dev/null); do
    kept=$((kept + 1))
    if [ "$kept" -le "$KEEP_RELEASES" ] || [ "$binary" = "$current" ] || [ "$binary" = "$previous" ]; then
      continue
    fi
    echo "suppression de $binary"
    rm -f "$RELEASES_DIR/$binary"
  done
}

cmd_status() {
  cmd_versions_short
  echo
  systemctl --user status "$UNIT" --no-pager -n 0 2>&1 | head -5 || true
  echo
  echo "socket  : $SOCKET_PATH -> $NGINX_CONTAINER:$NGINX_SOCKET_DIR/$SOCKET_NAME"
  if probe_health 2>/dev/null; then
    echo "santé   : ok"
  else
    echo "santé   : NE RÉPOND PAS"
  fi
  if probe_nginx_view; then
    echo "nginx   : voit le socket"
  else
    echo "nginx   : NE VOIT PAS le socket"
  fi
}

cmd_versions_short() {
  echo "actuelle  : $(read_version "$CURRENT_FILE")"
  echo "précédente: $(read_version "$PREVIOUS_FILE")"
}

cmd_versions() {
  cmd_versions_short
  echo "binaires conservés :"
  ls -1t "$RELEASES_DIR" 2>/dev/null | sed 's/^/  /' || echo "  (aucun)"
}

# One-time server preparation: directory layout + prerequisites check.
cmd_init() {
  mkdir -p "$RELEASES_DIR" "$CARDS_DIR"
  chmod 700 "$DATA_DIR"
  echo "arborescence prête sous $REMOTE_DIR"

  # Fail here rather than at the first deploy.
  check_linger
  echo "lingering systemd activé pour $USER"
  check_socket_dir
  echo "$SOCKET_DIR monté dans $NGINX_CONTAINER sur $NGINX_SOCKET_DIR"
}

# Validate the snippet that was just uploaded, then reload nginx.
#
# `nginx -t` FIRST, and remove the file if it fails: this proxy fronts other
# sites, and a broken include would take them all down at its next restart.
# Reload rather than restart: the other sites keep their connections.
cmd_nginx_reload() {
  local conf="$NGINX_CONF_DIR/$NGINX_CONF_NAME"

  docker container inspect "$NGINX_CONTAINER" >/dev/null 2>&1 \
    || die "aucun conteneur nommé '$NGINX_CONTAINER' — le proxy tourne-t-il ?"
  [ -f "$conf" ] || die "$conf n'a pas été envoyé"

  echo "==> Validation de la configuration nginx"
  if ! docker exec "$NGINX_CONTAINER" nginx -t; then
    rm -f "$conf"
    die "nginx refuse la configuration — $conf supprimé, nginx inchangé"
  fi

  echo "==> Rechargement de $NGINX_CONTAINER"
  docker exec "$NGINX_CONTAINER" nginx -s reload
  echo "$NGINX_CONTAINER rechargé avec $conf"
}

command="${1:-}"
shift || true
case "$command" in
  init)         cmd_init "$@" ;;
  nginx_reload) cmd_nginx_reload "$@" ;;
  activate)     cmd_activate "$@" ;;
  rollback)     cmd_rollback "$@" ;;
  prune)        cmd_prune "$@" ;;
  status)       cmd_status "$@" ;;
  health)       wait_for_health "${1:-1}" ;;
  versions)     cmd_versions "$@" ;;
  *)            die "commande inconnue '$command'" \
                    "(init|nginx_reload|activate|rollback|prune|status|health|versions)" ;;
esac
