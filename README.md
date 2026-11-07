# Armada Fleet Print

Turns a fleet saved on [armada.ryankingston.com](https://armada.ryankingston.com/) into a printable PDF of its card images.

## Usage

Go to https://apps.ieroe.com/armada/ and paste the link of a saved fleet, for example `https://armada.ryankingston.com/fleet/207536/`. The site is in French; an English version is available on request.

The PDF uses US Letter pages:

- **Ships:** two per page, each with its upgrade cards alongside.
- **Squadrons:** a 3×2 grid, with a badge showing how many of each card the fleet has.
- **Objectives:** one row after the squadrons.
- **Footer:** the fleet name, squadron and total points, and the page number.

### Multi-player games

The home page also creates a game shared by several players. There is no account: the game lives at a secret link (`games/<32 hex digits>/`), and anyone holding it can view and edit it. On the game page you can:

- **Paste the stock** of miniatures from a spreadsheet, with the columns Owner, Faction, Figurine Name, Size, Type (`Ship` or `Squadron`) and Qty. Copied cells are tab-separated; the header line is optional. Each name is matched to a fleet builder model ("Nebulon B Frigate" → `nebulon-b`) using the builder's card list, and names that match nothing are listed.
- **Add fleets**, each with a player name and a fleet builder link. Fleets are read again whenever the page is shown, so later edits on the builder are taken into account.
- **See the result:** miniatures used per model against the stock (a named squadron counts as its craft, so Luke Skywalker uses an X-wing), and unique cards taken by more than one fleet, matched by name ("Darth Vader (TIE Defender)" clashes with the commander Darth Vader).

Each game is saved as one JSON file in the games directory.

## Running it yourself

You need a Rust toolchain with edition 2024 support (1.85 or later).

```sh
# Web server on http://127.0.0.1:4567
./dev.sh

# One fleet to a PDF file, without the server
cargo run -- render https://armada.ryankingston.com/fleet/207536/ -o flotte.pdf
```

Card images are downloaded from the fleet builder the first time they are needed, then cached on disk.

### Card database

`data sync` builds a local JSON database of every Star Wars: Armada card (ships with their speed charts, squadrons, upgrades, objectives) from the community BattleScribe data of [BSData/star-wars-armada](https://github.com/BSData/star-wars-armada). It is an offline tool: the web server does not use it.

```sh
# Latest BSData release, written to data/armada.json
cargo run -- data sync

# A given tag, branch or commit, elsewhere
cargo run -- data sync --ref master -o /tmp/armada.json
```

The five data files are downloaded over HTTPS at the resolved commit, whose SHA is recorded in the output. Cards are sorted by faction then name, so two runs differ only by their `source` block. Conditional BattleScribe modifiers are not applied: the database holds base values. `data/` is not versioned, because the BSData repository has no license (see Acknowledgments).

### Configuration

| Option        | Environment variable | Default          | Purpose                           |
|---------------|----------------------|------------------|-----------------------------------|
| `--bind`      | `AFP_BIND`           | `127.0.0.1:4567` | Address the web server listens on |
| `--socket`    | `AFP_SOCKET`         | none             | Unix socket to listen on instead of `--bind` |
| `--cards-dir` | `AFP_CARDS_DIR`      | `pics`           | Card image cache directory        |
| `--games-dir` | `AFP_GAMES_DIR`      | `games`          | Directory of saved multi-player games (`serve` only) |
|               | `RUST_LOG`           | `info`           | Log filter                        |

## Deployment

The server (Linux, x86_64) runs the app as a **systemd user service**, without Docker. The binary is built on the Mac with [cross](https://github.com/cross-rs/cross) for `x86_64-unknown-linux-gnu`. nginx runs in Docker and reaches the app through a **Unix socket**, so no port is opened. The site is served under a sub-path of an existing domain, `/armada/` by default.

```
browser ──https──▶ $NGINX_CONTAINER (container)
                      │ proxy_pass http://unix:/sockets/armada-fleet-print.sock:/
                      ▼
       $SOCKET_DIR/armada-fleet-print.sock   (mounted :ro on /sockets)
                      ▲
                      │ listens
             systemd user service armada-fleet-print (on the host)
```

Server-specific settings are not in the repository: copy `.env.example` to `.env` (gitignored) and fill in the SSH host, the remote directory, the socket directory and the nginx container and config directory. `deploy/config.sh` reads `.env`, adds defaults for the rest (prefix, socket name, build target…), and stops with the list of missing variables if one is not set. A variable set in the environment wins over `.env`: `SSH_HOST=user@other make deploy`. `deploy/config.sh` renders both the nginx snippet (`deploy/nginx/`) and the systemd unit (`deploy/systemd/`), so the socket path and the prefix cannot drift apart. nginx strips the `/armada` prefix before proxying; the app's links are relative, so it never needs to know the prefix.

The snippet only holds `location` blocks: the proxy already has a `server {}` block with the domain and TLS, which includes every `*.conf` file of `NGINX_CONF_DIR`. `make setup` and `make nginx` install the snippet there, run `nginx -t` in the container (and remove the file if it fails, since a broken include would take down every site on the proxy), then reload nginx. Deploys never touch nginx.

### Prerequisites

On the Mac: `cargo install cross --git https://github.com/cross-rs/cross`, with Docker running.

On the server, once:

1. `SOCKET_DIR`, created by the SSH user **before** the proxy restarts (otherwise Docker creates it as root), is mounted into nginx. In the proxy's `docker-compose.yml`, nginx service: `- <SOCKET_DIR>:/sockets:ro`, then `docker compose up -d`.
2. systemd lingering for the SSH user, otherwise the service stops when the SSH session ends: `sudo loginctl enable-linger <user>`.
3. The SSH user is in the `docker` group.

Then, from the Mac: `make setup`, then `make deploy`. `make setup` checks both prerequisites and explains what to do if one is missing.

### Commands

| Command         | Purpose                                                                  |
|-----------------|--------------------------------------------------------------------------|
| `make setup`    | One-time server preparation (directories, prerequisites, nginx config)   |
| `make deploy`   | Builds with cross, uploads, restarts, checks that the app answers        |
| `make rollback` | Goes back to the previous version, without rebuilding                    |
| `make nginx`    | Reinstalls the nginx snippet and reloads the proxy                       |
| `make status`   | Service state, answer through the socket, socket visibility from nginx   |
| `make logs`     | Follows the service journal (`journalctl --user`)                        |
| `make versions` | Versions kept on the server                                              |

### On the server

```
$REMOTE_DIR/
  releases/<YYYYMMDD-HHMMSS>   one binary per version (the last 3 are kept)
  current                      symlink to the running binary
  current_version.txt          running version
  previous_version.txt         version restored by make rollback
  data/pics/                   card image cache, shared by all versions
  data/games/                  multi-player games, one JSON file each: back it up
  remote.sh                    server side of the scripts, re-uploaded by every command
~/.config/systemd/user/armada-fleet-print.service
$NGINX_CONF_DIR/armada-fleet-print.conf
```

The unit and the nginx snippet are overwritten on every install: edit them in `deploy/systemd/` and `deploy/nginx/`, not on the server.

A deploy builds the binary and checks its architecture, uploads it, records the previous version, restarts the service, checks that `/` answers through the socket and that nginx sees the socket, and only then prunes old versions. A few requests may get a 502 during the restart. The binary needs glibc 2.38 or later (Debian 13 ships 2.41) and the system CA certificates (`ca-certificates`).

## Development

```sh
cargo test
cargo clippy --all-targets
cargo fmt
```

See [CLAUDE.md](CLAUDE.md) for the architecture and code conventions. The card data in `misc/final_data/` was scraped from the fleet builder and is not used by the app at runtime.

## Author

**Cédric ZUGER**

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

Thanks to Ryan Kingston for the great work on the fleet builder :)

The card database built by `data sync` comes from [BSData/star-wars-armada](https://github.com/BSData/star-wars-armada), maintained by the BSData community, which is not endorsed by BattleScribe or any publisher. That repository has no license file, so its data is not redistributed here: each user downloads it. Star Wars: Armada and its card contents are © Fantasy Flight Games, Atomic Mass Games and Lucasfilm Ltd.
