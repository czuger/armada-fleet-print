#!/usr/bin/env bash
# Build here, ship the binary over SSH, restart there. No docker on the server side.
# Run it with: make deploy
#
# Steps: cross build -> architecture check -> scp -> version bookkeeping ->
# restart the systemd unit -> health check through the socket -> prune old ones.
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

# UTC so two machines deploying the same day still sort chronologically.
VERSION="$(date -u +%Y%m%d-%H%M%S)"
BINARY="$REPO_ROOT/target/$BUILD_TARGET/release/$BINARY_NAME"

command -v cross >/dev/null || {
  echo "erreur : cross n'est pas installé :" >&2
  echo "         cargo install cross --git https://github.com/cross-rs/cross" >&2
  exit 1
}

echo "==> Déploiement de $BINARY_NAME $VERSION sur $SSH_HOST:$REMOTE_DIR"

# 1. Build for the SERVER's architecture, not the laptop's. cross runs the build in
#    a container holding the target's toolchain and C libraries (aws-lc is compiled
#    from C sources), emulated on Apple Silicon.
echo "==> Compilation pour $BUILD_TARGET"
(cd "$REPO_ROOT" && cross build --release --target "$BUILD_TARGET")

# Confirm what actually came out before shipping it: a wrong-architecture binary
# would only fail on the server, as "Exec format error" in the journal.
#
# A function rather than `$(case ...)`: macOS's bash 3.2 cannot parse a case
# pattern's `)` inside a command substitution.
elf_arch_name() {
  case "$1" in
    x86_64-*) echo "x86-64" ;;
    aarch64-*) echo "aarch64" ;;
    *) echo "${1%%-*}" ;;
  esac
}
expected_arch="$(elf_arch_name "$BUILD_TARGET")"
description="$(file -b "$BINARY")"
case "$description" in
  ELF*"$expected_arch"*) ;;
  *)
    echo "erreur : $BINARY n'est pas un binaire ELF $expected_arch :" >&2
    echo "         $description" >&2
    exit 1
    ;;
esac
echo "    architecture : $expected_arch (ok)"
du -h "$BINARY" | cut -f1 | xargs echo "    taille :"

# 2. Ship it. remote.sh and the unit go first so the server side always matches
#    this checkout.
echo "==> Envoi"
upload_remote_script
upload_systemd_unit
ssh "$SSH_HOST" "mkdir -p '$REMOTE_DIR/releases'"
scp -C "$BINARY" "$SSH_HOST:$REMOTE_DIR/releases/$VERSION"

# 3. Record the version swap, restart, wait for the app to answer. Any failure here
#    stops the script (set -e) with the old version still recorded as previous, so
#    `make rollback` is the immediate way out.
echo "==> Activation de $VERSION sur le serveur"
if ! remote activate "$VERSION"; then
  echo >&2
  echo "erreur : $VERSION ne répond pas. Pour revenir en arrière : make rollback" >&2
  exit 1
fi

# 4. Housekeeping — only after a healthy start, so a failed deploy never deletes
#    the binary we might need to go back to.
echo "==> Nettoyage des anciennes versions (conservées : $KEEP_RELEASES)"
remote prune

echo
echo "Version $VERSION déployée : https://<hôte>$URL_PREFIX/"
remote versions
