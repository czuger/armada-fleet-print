#!/usr/bin/env bash
# Shared settings for the deploy scripts, sourced by all of them (never run alone).
#
# Server-specific values (host, paths, container) are not in the repository: they
# come from .env at the repository root (gitignored, copy .env.example). Any value
# set in the environment wins over .env:
#   SSH_HOST=user@other-box make deploy
#
# This is the single source of truth for both the systemd unit and the nginx
# snippet: the socket path and URL prefix are rendered into each of them from
# here, so the two sides cannot disagree.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="$REPO_ROOT/.env"

# Read KEY=VALUE lines from .env without overriding what the environment already
# sets. Not `source`: .env is data, and sourcing it would let the file win.
load_env_file() {
  local key value
  [ -f "$ENV_FILE" ] || return 0
  while IFS='=' read -r key value || [ -n "$key" ]; do
    case "$key" in
      ''|'#'*) continue ;;
    esac
    case "$key" in
      *[!A-Za-z0-9_]*|[0-9]*)
        echo "erreur : ligne invalide dans $ENV_FILE : $key=$value" >&2
        return 1
        ;;
    esac
    [ -n "${!key+set}" ] && continue
    value="${value%\"}"; value="${value#\"}"
    value="${value%\'}"; value="${value#\'}"
    export "$key=$value"
  done < "$ENV_FILE"
}

# Fail early, naming every missing variable, rather than ssh-ing to an empty host.
require_vars() {
  local name missing=""
  for name in "$@"; do
    [ -n "${!name:-}" ] || missing="$missing $name"
  done
  [ -z "$missing" ] && return 0
  echo "erreur : variable(s) non définie(s) :$missing" >&2
  echo "         renseignez-les dans $ENV_FILE (modèle : .env.example)" >&2
  return 1
}

load_env_file || exit 1
require_vars SSH_HOST REMOTE_DIR SOCKET_DIR NGINX_CONTAINER NGINX_CONF_DIR || exit 1

# Where to deploy (.env): SSH_HOST, and REMOTE_DIR, the app's directory there.

# What runs there: a systemd *user* unit, so deploys never need sudo. It only
# survives logout if lingering is enabled for the SSH user (`make setup` checks).
SERVICE_NAME="${SERVICE_NAME:-armada-fleet-print}"
BINARY_NAME="armada-fleet-print"

# Architecture of the SERVER, not of your laptop: cross builds for it from the Mac.
# The gnu target links dynamically against glibc, so the server's glibc must be at
# least the build image's (Debian 13 ships 2.41).
BUILD_TARGET="${BUILD_TARGET:-x86_64-unknown-linux-gnu}"

# The app publishes no port: it listens on a Unix socket in a host directory that
# the nginx container bind-mounts. That directory belongs to the proxy's project,
# like the docker network does for containerized apps — we only put our socket in
# it, and refuse to run if nginx does not see it (a socket nginx cannot reach would
# look fine here and 502 at request time).
#   SOCKET_DIR        the directory, as seen on the host (.env)
#   NGINX_SOCKET_DIR  the same directory, as seen inside the nginx container
NGINX_SOCKET_DIR="${NGINX_SOCKET_DIR:-/sockets}"
SOCKET_NAME="${SOCKET_NAME:-armada-fleet-print.sock}"

# The dockerized nginx (.env): NGINX_CONTAINER, and NGINX_CONF_DIR, the host
# directory its server{} includes *.conf from. `make setup` / `make nginx` render
# the snippet into that directory and reload nginx; a deploy never touches either.
NGINX_CONF_NAME="${NGINX_CONF_NAME:-armada-fleet-print.conf}"

# Sub-path the site is served under, i.e. https://<host>$URL_PREFIX. nginx strips it
# before proxying. The app never needs it: its links are relative.
URL_PREFIX="${URL_PREFIX:-/armada}"

# How many binaries stay in releases/. 3 means "current, previous, and one more" —
# rollback only ever needs the previous one.
KEEP_RELEASES="${KEEP_RELEASES:-3}"

# Everything below is derived; no need to touch it.
REMOTE_SCRIPT="$REMOTE_DIR/remote.sh"
URL_PREFIX="${URL_PREFIX%/}"

# The server side of every script lives in deploy/remote.sh and is uploaded before
# each run, so deploy and rollback can never drift apart.
upload_remote_script() {
  ssh "$SSH_HOST" "mkdir -p '$REMOTE_DIR'"
  scp -q "$REPO_ROOT/deploy/remote.sh" "$SSH_HOST:$REMOTE_SCRIPT"
  ssh "$SSH_HOST" "chmod +x '$REMOTE_SCRIPT'"
}

# Render deploy/<dir>/<name>.template into dist/<name> and print the rendered path.
render_template() {
  local template="$REPO_ROOT/deploy/$1/$2.template"
  local rendered="$REPO_ROOT/dist/$2"

  [ -f "$template" ] || { echo "erreur : $template introuvable" >&2; return 1; }
  mkdir -p "$REPO_ROOT/dist"
  sed -e "s|__PREFIX__|$URL_PREFIX|g" \
      -e "s|__REMOTE_DIR__|$REMOTE_DIR|g" \
      -e "s|__SOCKET_DIR__|$SOCKET_DIR|g" \
      -e "s|__NGINX_SOCKET_DIR__|$NGINX_SOCKET_DIR|g" \
      -e "s|__SOCKET_NAME__|$SOCKET_NAME|g" \
      "$template" > "$rendered"
  echo "$rendered"
}

# The unit is uploaded by every deploy and rollback, like remote.sh: how the app is
# started (socket, card cache path) must match the version being started.
upload_systemd_unit() {
  local rendered
  rendered="$(render_template systemd "$SERVICE_NAME.service")"
  ssh "$SSH_HOST" "mkdir -p ~/.config/systemd/user"
  scp -q "$rendered" "$SSH_HOST:.config/systemd/user/$SERVICE_NAME.service"
  rm -f "$rendered"
}

# Render the nginx snippet, ship it, and have the server validate it before
# reloading nginx. Used by setup_server.sh and `make nginx`; no deploy ever calls
# it — routing does not change between versions.
install_nginx_conf() {
  local rendered
  rendered="$(render_template nginx "$NGINX_CONF_NAME")"
  ssh "$SSH_HOST" "mkdir -p '$NGINX_CONF_DIR'"
  scp -q "$rendered" "$SSH_HOST:$NGINX_CONF_DIR/$NGINX_CONF_NAME"
  rm -f "$rendered"

  upload_remote_script
  remote nginx_reload
}

# Runs remote.sh on the server with this file's settings passed through, so the
# server never keeps its own stale copy of them.
remote() {
  ssh "$SSH_HOST" \
    "REMOTE_DIR='$REMOTE_DIR' SERVICE_NAME='$SERVICE_NAME' KEEP_RELEASES='$KEEP_RELEASES' \
     SOCKET_DIR='$SOCKET_DIR' NGINX_SOCKET_DIR='$NGINX_SOCKET_DIR' SOCKET_NAME='$SOCKET_NAME' \
     NGINX_CONTAINER='$NGINX_CONTAINER' NGINX_CONF_DIR='$NGINX_CONF_DIR' \
     NGINX_CONF_NAME='$NGINX_CONF_NAME' \
     bash '$REMOTE_SCRIPT' $*"
}
