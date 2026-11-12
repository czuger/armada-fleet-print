# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Armada Fleet Print: a small web app that takes a saved fleet URL from https://armada.ryankingston.com/ and produces a printable PDF of the card images (ships with their upgrades, squadrons, objectives). The interface is in French and English; card images are never translated.

## Commands

- Build: `cargo build` (edition 2024)
- Dev server: `./dev.sh`, which runs `cargo run -- serve`. It listens on `127.0.0.1:4567`; pass `--bind` or set `AFP_BIND` to change that, or `--socket`/`AFP_SOCKET` to listen on a Unix socket instead.
- Render one fleet without the server: `cargo run -- render https://armada.ryankingston.com/fleet/207536/ -o flotte.pdf [--orientation landscape] [--lang en]`
- Build the card database from BSData (offline tool, not used by the server): `cargo run -- data sync [--ref <git-ref>] [-o <file>]`, written to `<data-dir>/armada.json` by default. `data/` is gitignored because BSData has no license.
- Tests: `cargo test`. Run a single test with `cargo test --test layout drops_trailing_empty_page`, or `cargo test fleet_url::` for unit tests.
- Lint and format: `cargo clippy --all-targets`, `cargo fmt`

All persistent data lives under the data directory, `data/` relative to the CWD by default (gitignored); override with `--data-dir` or `AFP_DATA_DIR`. Card images are cached in its `pics/` subdirectory and multi-player games saved as JSON files in its `games/` subdirectory, both created on first write. The server uses no database.

## Architecture

The crate is a library (`src/lib.rs`, `#![warn(missing_docs)]`) plus a thin binary (`src/main.rs` → `cli::Cli::run`). A print request flows through `web::router` (axum) → `FleetPrinter::print` (`src/fleet_printer.rs`), which chains these steps:

1. **Fetch and parse** (`fleet_url`, `fleet_script`, `fleet_json`, `fleet`): `FleetUrl` only accepts `armada.ryankingston.com` links and always fetches over HTTPS. The page embeds the fleet as JSON in an inline script (`fleet: {...}, stock:`). `fleet_json` mirrors that JSON with serde, and `Fleet` is the reduced domain model. Upgrade slots keep the builder's order through `IndexMap`, and squadrons are merged per card with a count. Image names go through `CardName`, which rejects anything that isn't a plain file name, so the cache path can't be escaped.
2. **Card images** (`card_store`): `CardStore` downloads `https://armada.ryankingston.com/img/cards/<name>` once, at most 4 at a time, and writes each file through a temp file plus rename.
3. **Decode, layout, render**: this step runs in `spawn_blocking`. `CardImage` splits each image into RGB and optional alpha planes. `layout::lay_out` turns the fleet into a `DocumentPlan`, a list of pages of positioned `Element`s in PDF points. `pdf::render` writes the PDF with `pdf-writer`: each image is embedded once, alpha becomes an SMask, and text uses built-in Helvetica with WinAnsi encoding. `pdf/helvetica.rs` holds the encoding and the width table used for right-aligned and centered text.

The layout in `src/layout/fleet_layout.rs` uses US Letter pages, portrait or landscape (`layout::Orientation`, carried by the `DocumentPlan` and written as the page tree's MediaBox), with 36 pt margins and card sizes hard-coded as constants. `Geometry` derives from the orientation how many 180 pt cards fit on a line, letting them run a few points into the right margin: 3 in portrait (546 pt line), 4 in landscape (729 pt line). It places blocks in builder order with a next-fit shelf (`src/layout/shelf.rs`): a block goes right of the previous one if it fits on the line (3 pt gap), otherwise on a new line under the tallest block of the current one (10 pt gap), otherwise on a new page.
- **Ships:** 200 pt wide in portrait, the whole content height (540 pt) in landscape (`Geometry::ship_size`). Upgrades sit to their right, 10 pt apart, drawn at 1.5 times their width; that width is 100 pt scaled like the ship (100 pt in portrait, about 157 pt in landscape). `UpgradeGrid::fit` (`src/layout/upgrade_grid.rs`) picks the column count that draws them largest within that width and the area left of the line and page, so they shrink only when too many to fit. A ship with upgrades takes a whole line; ships without upgrades sit side by side.
- **Squadrons:** 180 pt cards, three per line in portrait and four in landscape, with a white count badge on each.
- **Objectives:** 180 pt cards on a page of their own.
- **Cleanup:** empty pages are dropped.
- **Footer:** "<name> (squadron pts/total) - site" and "page N sur M" or "page N of M" (`lay_out` takes a `lang::Lang`).

### Multi-player games (`src/game/`)

A game is reached only through its secret link `games/<GameId>/` (128 random bits as 32 hex digits; `GameId` parsing only accepts that shape, so it is a safe file name). The trailing slash matters: the page's forms and redirects are relative (`stock`, `fleets`, `./`, `../../print`) so they work under any URL prefix. `GameStore` saves one JSON file per game with `write_atomic` and serializes read-modify-write cycles with a mutex. `GameService` holds the operations:
- **Stock:** `Stock::parse` reads a spreadsheet paste (tabs, or 2+ spaces when tabs were lost; columns Owner, Faction, Figurine Name, Size, Type, Qty; optional header). Each name is matched once, at import, to a builder model (`type` field of the builder cards, e.g. `mc80-liberty`) by `ModelCatalogue::resolve` and `name_matcher`: an exact title or type wins, otherwise words score by rarity among the candidates of the same kind and faction, and a tie matches nothing. The catalogue comes from the builder's `cards.js`, downloaded by `CatalogueSource` and cached in memory for a day.
- **Fleets:** a fleet is validated by fetching it when added or replaced, then re-read on every page view (`FleetRoster`: miniatures per model, unique card titles). Replacing keeps the fleet's id and place: the row's button opens `games/<id>/?replace=<fleet>`, which turns the add form into a form posting to `replace` (a sibling of `fleets`, so a rejected form re-renders at the same depth), prefilled with the fleet's player and link.
- **Report:** `GameReport` compares the used miniatures with the stock per model, and lists unique cards held more than once, compared by name without the trailing "(…)" variant.

`tests/fixtures/builder_cards.js` is a reduced extract of the builder card list (title, type, faction, unique) and `tests/fixtures/stock.tsv` a real stock sheet.

Pages are askama templates in `src/web/templates/` (configured by `askama.toml`), which escape HTML and resolve bare calls as methods of the template struct. `base.html` holds the head and is extended by `index.html`, `error.html` and `game.html`; `lang_switch.html` is the FR | EN switch, whose links come from each page's `switch_link` method. Errors caused by form input re-render the game page with the message and the typed text.

**Languages.** `lang::Lang` (`Fr`, the default, or `En`). The `localize` middleware (`web/localize.rs`) gives each request its language, read by `lang_cookie::request_lang`: the `lang` cookie, else `Accept-Language`, else French. Handlers take it as `Extension<Lang>`. `GET lang?to=en[&game=<id>]` saves the cookie and redirects to the game (only a valid `GameId`, so no open redirect) or the home page; it lives at the app root so the cookie, which has no `Path`, covers the whole app. Interface text lives in `src/web/text/`: one struct of `&'static str` per page (`PageText`, `IndexText`, `GameText`) with a `FR` and an `EN` const, so a missing translation does not compile, and the `ErrorMessage` enum. Messages with arguments (stock and game form errors) are in `web/game_form_error.rs`, one `match` arm per language. `web::WebError` maps each failure to an HTTP status and an `ErrorMessage` put in the response extensions, and logs the English error chain (`error_chain`); `localize` then writes the error page in the request language, keeping the response headers. The printed PDF follows the language too (footer, `flotte.pdf`/`fleet.pdf`).

The orientation radio buttons of the home page post `orientation` with the link; `POST print` saves it in an `orientation` cookie (`web/orientation_cookie.rs`, no `Path` so it covers the app root behind any prefix) even when printing fails, and uses the cookie when the form has no `orientation`, as with the PDF buttons of the game page. `GET /` checks the saved choice.

`tests/fixtures/fleet_207536.html` is a real saved fleet page used by the parsing tests.

### Card database (`src/armada_data/`)

`DataSync` resolves the ref (latest GitHub release, else `master`) to a SHA with `GithubSource`, downloads `Star_Wars_Armada.gst` and the four faction `.cat` files from `raw.githubusercontent.com` at that SHA, parses each with `BsCatalogue` (roxmltree, namespaces ignored, `<modifiers>` ignored), builds an `ArmadaDatabase` and writes it with `atomic_file::write_atomic`. The output (`data/armada.json` by default) holds `schema_version`, `source` (repo, ref, commit SHA, `generated_at`) and four arrays: `ships`, `squadrons`, `upgrades` and `objectives`, each entry keyed by its BSData `id`. The server does not read it yet. `CardKind` maps the profile `typeName` to ship, squadron, speed chart, upgrade or objective. Pitfalls handled there:
- a card is unique if the profile name **or** the entry name starts with `•` (Mon Karren);
- speed chart profiles are sorted by decreasing name length, since the XML order is reversed for the Acclamator; an empty cell means the speed is unavailable, `-` means 0;
- upgrades are merged across files by (lowercase name, type, points) and accumulate `factions`; the points keep the two Ahsoka Tano officers apart;
- the objective station (`1.5 Stations`, in the `.gst`) is a ship with no faction and no size.

`tests/fixtures/bsdata/` holds reduced extracts of release 1.7.8.

## misc/

`misc/final_data/units.json` (faction → unit name → `{name, image, points}`) and `misc/final_data/names.json` (faction → ships/squadrons/named-squadrons name lists) hold card data scraped from the fleet builder. `misc/wip/` holds the intermediate files; the Ruby scripts that generated them were removed. The app does not use these files at runtime.

## Language

- Code is English, whatever language the prompt is in: identifiers, function names, comments, docstrings, commit messages and log lines are English, even when the discussion is in French.
- Web interface text exists in French and English: add or change it in both, in `src/web/text/` (or `web/game_form_error.rs` for messages with arguments), never as literals in templates. Keep French UTF-8 with accents.
- CLI help and output stay French only.

## Rust code style

Follow idiomatic Rust and the clippy/rustfmt defaults except where these rules say otherwise.

### Imports
- Prefer `crate::`-rooted paths across modules.
- `super::` is fine inside tightly nested modules, for example a `tests` submodule, or a submodule using its direct parent's types. Avoid `super::super::` chains; restructure the module tree instead.
- No glob imports, except for preludes designed for them and `use super::*;` in tests.

### Types
- Annotate every `pub` function signature (parameters and return type). Let inference handle local bindings.
- Prefer concrete types over generics, unless the generic is actually called with more than one type or is part of a public API meant to be generic.
- Avoid `Box<dyn Any>` and other type erasure unless strictly necessary, and justify it with a short comment.

### Docs and comments
- Every `pub` item gets a `///` rustdoc, with `# Arguments`, `# Returns`, `# Errors` and `# Panics` sections as applicable. Enforce this with `#![warn(missing_docs)]`, or `#![deny(missing_docs)]` for libraries.
- Private functions get a docstring only when the logic isn't obvious from the name and signature.
- Keep docstrings proportionate, and don't restate what the signature already says.
- Comment only genuinely complex or easily misread logic. Keep comments short, about intent or reasoning, and never caption the obvious.

### Error handling
- Use `Result<T, E>` for recoverable errors and `Option<T>` for absence. Never use panics as control flow.
- Propagate errors with `?`, unless the error needs to be transformed or given context at that point.
- Define concrete error enums, plain or with `thiserror`. Avoid stringly-typed errors and `Box<dyn Error>` in library code. `Box<dyn Error>` is acceptable at binary boundaries such as `main`.
- No `unwrap()` or `expect()` in production paths; they're for tests, examples and prototypes. If failure is provably impossible, justify it with a comment such as `// safe: regex is a compile-time constant`.
- Never silently discard errors with `.ok()`, `let _ =` or similar. Handle, log or propagate them.
- Keep `panic!` and `unreachable!` for genuine invariant violations (bugs), and leave no `todo!` or `unimplemented!` in shipped code.

### Functions
- Soft limit of 50 lines per function. Split longer ones into well-named sub-functions, each with a single clear responsibility and a short docstring.
- Prefer composing small functions, and flatten nesting with early returns and `?` rather than nested `match`/`if let` pyramids.

### Files and modules
- One primary public type (struct, enum or trait) per file, together with its `impl` blocks. The snake_case file name matches the type: `UserAccount` → `user_account.rs`.
- Small, tightly coupled types can share the primary type's file: its error enum, its config or builder struct, its result type.
- A family of small types with no independent identity, such as AST nodes or state-machine states, can live in one file named for the concept. Split it out once it grows large.
- A trait implemented by several unrelated types gets its own file. A trait with a single implementor can live with that type.
- No monolithic files. Split by topic into modules as soon as a file does too much, and use directory module trees for larger topics.
- Keep `mod.rs` and module-declaration files thin, with `pub use` re-exports only. Keep `lib.rs` and `main.rs` as thin entry points (module declarations, wiring, re-exports), with no business logic.
- Prefer composition, and use traits for shared behavior rather than deep hierarchies. Favor readable, explicit, small, testable units over cleverness.
