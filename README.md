# Armada Fleet Print

Turns a fleet saved on [armada.ryankingston.com](https://armada.ryankingston.com/) into a printable PDF of its card images.

## Usage

Open the site and paste the link of a saved fleet, for example `https://armada.ryankingston.com/fleet/207536/`. The site is in French and English: it follows the browser's language, and the FR | EN links at the top switch it.

The PDF uses US Letter pages, in portrait or landscape: pick one with the radio buttons next to « Envoyer ». The choice is kept in a cookie and also applies to the PDF buttons of the game pages. In portrait:

- **Ships:** two per page, each with its upgrade cards alongside.
- **Squadrons:** a 3×2 grid, with a badge showing how many of each card the fleet has.
- **Objectives:** one row after the squadrons.
- **Footer:** the fleet name, squadron and total points, and the page number.

In landscape, ships are printed larger, the full height of the page, with their upgrade cards enlarged in the same proportion (and shrunk only when too many would not fit beside the ship). A ship with upgrades therefore takes a page of its own, and two ships without upgrades share one. Squadrons keep their size, four per line instead of three.

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

# The same, on landscape pages
cargo run -- render https://armada.ryankingston.com/fleet/207536/ -o flotte.pdf --orientation landscape
```

Card images are downloaded from the fleet builder the first time they are needed, then cached on disk in `pics/` under the data directory, next to the saved games in `games/`. The data directory is `data/` by default; `--data-dir` or `AFP_DATA_DIR` moves it.

### Card database

`data sync` builds a local JSON database of every Star Wars: Armada card (ships with their speed charts, squadrons, upgrades, objectives) from the community BattleScribe data of [BSData/star-wars-armada](https://github.com/BSData/star-wars-armada). It is an offline tool: the web server does not use it.

```sh
# Latest BSData release, written to data/armada.json
cargo run -- data sync

# A given tag, branch or commit, elsewhere
cargo run -- data sync --ref master -o /tmp/armada.json
```

The five data files are downloaded over HTTPS at the resolved commit, whose SHA is recorded in the output. Cards are sorted by faction then name, so two runs differ only by their `source` block. Conditional BattleScribe modifiers are not applied: the database holds base values. `data/` is not versioned, because the BSData repository has no license (see Acknowledgments).

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
