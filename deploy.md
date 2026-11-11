Standardize this project's deployment on the model described below.
Start in plan mode: read this project first (stack, how it is served today, existing
Makefile/scripts/Docker files, persistent data, env vars), then propose a plan that maps every
point below onto it, and list what does not fit before writing anything.

## Target architecture
- The server runs the app as a **systemd user service** — no Docker for the app,
  no sudo during deploys. nginx runs in Docker (an existing shared proxy that fronts other sites)
  and reaches the app through a **Unix socket**: no port is opened.
- The site is served under a **sub-path** of an existing domain (`URL_PREFIX`, e.g. `/myapp`).
  nginx strips the prefix; the app is mounted at `/` and uses **relative links** only, so it never
  knows the prefix. Fix absolute links/redirects in the app if there are any.
- The build runs on the local machine, never on the server, and produces **one self-contained
  artifact per release** for the server's platform (`BUILD_TARGET`): a cross-compiled binary for
  a compiled language, a bundled archive otherwise. Tell me if the stack does not fit "one
  artifact per release".

```
browser ──https──▶ $NGINX_CONTAINER (container)
                      │ proxy_pass http://unix:/sockets/<app>.sock:/
                      ▼
       $SOCKET_DIR/<app>.sock   (bind-mounted :ro on /sockets)
                      ▲ listens
             systemd user service <app> (on the host)
```

## App-side requirements
- Listen on TCP by default for dev (`127.0.0.1:<port>`, `--bind` / `<PREFIX>_BIND`) or on a Unix
  socket when `--socket` / `<PREFIX>_SOCKET` is set.
- On a socket: remove a stale *socket* at the path before binding (refuse if the path is anything
  else), `chmod 0666` after bind (nginx runs under another uid in its container), remove it on exit.
- Graceful shutdown on SIGTERM and Ctrl-C (drain open requests).
- Every persistent path (caches, user data) configurable by flag + env var, pointing under
  `$REMOTE_DIR/data/` in production so data survives deploys.
- `GET /` must answer quickly without network access: it is the health check.
- Logs to stdout/stderr in plain text (journald), `NO_COLOR=1` honored.

## Files to create
- **`.env.example`** (committed) and **`.env`** (gitignored): server-specific values only —
  `SSH_HOST`, `REMOTE_DIR`, `SOCKET_DIR`, `NGINX_CONTAINER`, `NGINX_CONF_DIR`; commented optional
  ones with defaults (`NGINX_SOCKET_DIR=/sockets`, `URL_PREFIX`, `BUILD_TARGET`, `KEEP_RELEASES=3`).
  The repository may be public: **never hard-code a host, user, path or container name anywhere
  tracked** (scripts, templates, README, CLAUDE.md).
- **`deploy/config.sh`** (sourced by every script, never run alone) — the single source of truth:
  - `load_env_file`: reads `.env` line by line as data (not `source`), rejects invalid keys,
    strips quotes, and **never overrides a variable already set in the environment**
    (`SSH_HOST=other make deploy` wins).
  - `require_vars`: fails early listing *all* missing required variables and pointing to `.env.example`.
  - Defaults: `SERVICE_NAME`, `BINARY_NAME`, `BUILD_TARGET`, `NGINX_SOCKET_DIR`, `SOCKET_NAME`,
    `NGINX_CONF_NAME`, `URL_PREFIX` (trailing slash stripped), `KEEP_RELEASES`.
  - `render_template <dir> <name>`: `sed` of `__PREFIX__`, `__REMOTE_DIR__`, `__SOCKET_DIR__`,
    `__NGINX_SOCKET_DIR__`, `__SOCKET_NAME__` from `deploy/<dir>/<name>.template` into `dist/`
    (gitignored). Both the systemd unit and the nginx snippet are rendered from here, so the socket
    path and prefix cannot drift.
  - `upload_remote_script` (scp `deploy/remote.sh` before every command), `upload_systemd_unit`
    (to `~/.config/systemd/user/`, on every deploy *and* rollback), `install_nginx_conf`
    (scp the snippet, then `remote nginx_reload`), and `remote <subcommand>` which runs
    `remote.sh` over SSH passing every setting as env vars, so the server keeps no stale copy.
- **`deploy/remote.sh`** — the whole server side, uploaded before each run; subcommands
  `init | nginx_reload | activate VERSION | rollback | prune | status | health | versions`:
  - `init`: create `releases/` and `data/...` (`chmod 700 data`), check systemd **lingering**
    (`loginctl show-user $USER -p Linger`) and that `SOCKET_DIR` exists, is writable, and is
    mounted in `$NGINX_CONTAINER` at `$NGINX_SOCKET_DIR` (`docker inspect` on `.Mounts`). Never
    create `SOCKET_DIR` itself (docker would make it root-owned); print exact fix-up instructions
    instead (`sudo loginctl enable-linger`, the compose volume line, `docker compose up -d`).
  - `activate`: chmod the binary, write `previous_version.txt`/`current_version.txt` **before**
    restarting (so rollback is ready if the start fails), `ln -sfn` `current` → `releases/<v>`,
    `daemon-reload`, `enable`, `restart`, then `wait_for_health`.
  - `wait_for_health`: poll `curl --unix-socket $SOCKET_PATH http://localhost/` up to 15 s; on
    success also check `docker exec $NGINX_CONTAINER test -S <socket>` (warn if nginx can't see it);
    on failure dump the last 40 journal lines and exit non-zero.
  - `rollback`: swap the two version files (running it twice returns to the start), restart, health check.
  - `prune`: keep the `KEEP_RELEASES` newest, never delete current or previous.
  - `nginx_reload`: `nginx -t` inside the container **first**; if it fails, delete our snippet and
    stop (a broken include would take down every site on the proxy); else `nginx -s reload`
    (reload, not restart).
  - `status`: versions, `systemctl --user status`, socket health, nginx visibility.
- **`deploy/deploy.sh`**: check the build toolchain is installed → version
  `$(date -u +%Y%m%d-%H%M%S)` → build → verify the artifact targets `BUILD_TARGET` (e.g. `file -b`
  on a binary; fail before shipping) →
  upload remote.sh + unit → `scp -C` to `releases/<version>` → `remote activate` (on failure:
  "make rollback" hint) → `remote prune` only after a healthy start → `remote versions`.
- **`deploy/rollback.sh`**: no build, no transfer; upload remote.sh + unit, `remote rollback`.
- **`deploy/setup_server.sh`**: one-time and re-runnable; check `systemctl loginctl journalctl
  curl docker` exist and `docker info` works for the SSH user, `remote init`, `install_nginx_conf`,
  print the resulting layout and "next: make deploy".
- **`deploy/nginx.sh`**: reinstall the snippet only. Deploys never touch nginx.
- **`deploy/systemd/<service>.service.template`**: `Type=exec`,
  `ExecStart=__REMOTE_DIR__/current <args>`, `Environment=` for the socket path, data dirs,
  the app's log level, `NO_COLOR=1`; `UMask=0077`, `NoNewPrivileges=yes`,
  `Restart=on-failure`, `RestartSec=2`, `WantedBy=default.target`. Header comment: rendered and
  overwritten, don't edit on the server.
- **`deploy/nginx/<app>.conf.template`**: `location` blocks only (the proxy's `server{}` owns
  `server_name`/TLS and includes `NGINX_CONF_DIR/*.conf`):
  `location = __PREFIX__ { return 301 __PREFIX__/; }` and `location __PREFIX__/ { proxy_pass
  http://unix:__NGINX_SOCKET_DIR__/__SOCKET_NAME__:/; ... }` with `Host`, `X-Real-IP`,
  `X-Forwarded-For`, `X-Forwarded-Proto` headers and a `proxy_read_timeout` suited to the app.
- **`Makefile`**: thin shortcuts only — `help` (self-documenting from `## ` comments), `setup`,
  `nginx`, `deploy`, `rollback`, `logs` (`ssh -t ... journalctl --user -u $SERVICE_NAME -f -n 100`),
  `status`, `versions`, `test`; `status`/`versions` via
  `REMOTE = bash -c 'source deploy/config.sh && upload_remote_script && remote "$$0"'`.
- **`.gitignore`**: `.env`, `/dist`.
- **`dev.sh`**: runs the app locally on TCP.

## Server layout
```
$REMOTE_DIR/
  releases/<YYYYMMDD-HHMMSS>   one artifact per version (KEEP_RELEASES kept)
  current                      symlink to the running one
  current_version.txt / previous_version.txt
  data/...                     persistent data shared by all versions
  remote.sh                    re-uploaded by every command
~/.config/systemd/user/<service>.service
$NGINX_CONF_DIR/<app>.conf
```

## Script conventions
- `#!/usr/bin/env bash` + `set -euo pipefail` (not in `config.sh`, which is sourced).
- Must run with bash 3.2 or later (e.g. no `case` with `)` inside `$(...)`; use a function).
- Comments explain *why* (each design choice above has a reason — keep it next to the code).
  Code and comments in English; messages printed to the user in the project's CLI language.
- Every failure says what to do next (missing var, missing prereq, failed health → `make rollback`).

## Docs
- README "Deployment" section: the diagram, how `.env` works and that the environment wins,
  prerequisites (local machine: the build toolchain, and Docker if it needs it; server, once:
  create `SOCKET_DIR` as the SSH user *before* restarting the proxy and mount it `:ro` on `/sockets`, `sudo loginctl enable-linger <user>`,
  SSH user in the `docker` group), the commands table, the server layout, what a deploy does,
  the platform requirements of the build target (e.g. minimum libc version).
- CLAUDE.md "Commands": the make targets, and the rule that server values live only in `.env`.

## Verification
- `bash -n` every script; `shellcheck` if available.
- Run the app locally on a socket and `curl --unix-socket ... http://localhost/`.
- Render the templates (`source deploy/config.sh && render_template ...` with a dummy `.env`)
  and check no `__PLACEHOLDER__` remains.
- Do **not** run `make setup` or `make deploy` yourself: they act on my server. Tell me when to.
