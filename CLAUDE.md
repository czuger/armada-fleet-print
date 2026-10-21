# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Armada Fleet Print: a small web app that takes a saved fleet URL from https://armada.ryankingston.com/ and produces a printable PDF of the card images (ships with their upgrades, squadrons, objectives). Live at https://apps.ieroe.com/armada/. The UI text is in French.

## Commands

- Build: `cargo build` (edition 2024)
- Dev server: `./dev.sh`, which runs `cargo run -- serve`. It listens on `127.0.0.1:4567`; pass `--bind` or set `AFP_BIND` to change that, or `--socket`/`AFP_SOCKET` to listen on a Unix socket instead.
- Deploy: `make setup` (once), `make deploy`, `make rollback`, `make status`, `make logs`. `make deploy` cross-builds for `x86_64-unknown-linux-gnu` and runs the binary on the server as a systemd user service, behind the dockerized nginx through a Unix socket, under `/armada`. Server-specific values (SSH host, paths, nginx container) live only in the gitignored `.env` (template: `.env.example`); never hard-code them in the repository, which is public. `deploy/config.sh` loads `.env`, lets the environment win, and renders `deploy/systemd/` and `deploy/nginx/`. The server side lives in `deploy/remote.sh`. See the README's Deployment section.
- Render one fleet without the server: `cargo run -- render https://armada.ryankingston.com/fleet/207536/ -o flotte.pdf`
- Tests: `cargo test`. Run a single test with `cargo test --test layout drops_trailing_empty_page`, or `cargo test fleet_url::` for unit tests.
- Lint and format: `cargo clippy --all-targets`, `cargo fmt`

Card images are cached in `pics/` relative to the CWD (gitignored). Override with `--cards-dir` or `AFP_CARDS_DIR`. No database is used.

## Architecture

The crate is a library (`src/lib.rs`, `#![warn(missing_docs)]`) plus a thin binary (`src/main.rs` → `cli::Cli::run`). A print request flows through `web::router` (axum) → `FleetPrinter::print` (`src/fleet_printer.rs`), which chains these steps:

1. **Fetch and parse** (`fleet_url`, `fleet_script`, `fleet_json`, `fleet`): `FleetUrl` only accepts `armada.ryankingston.com` links and always fetches over HTTPS. The page embeds the fleet as JSON in an inline script (`fleet: {...}, stock:`). `fleet_json` mirrors that JSON with serde, and `Fleet` is the reduced domain model. Upgrade slots keep the builder's order through `IndexMap`, and squadrons are merged per card with a count. Image names go through `CardName`, which rejects anything that isn't a plain file name, so the cache path can't be escaped.
2. **Card images** (`card_store`): `CardStore` downloads `https://armada.ryankingston.com/img/cards/<name>` once, at most 4 at a time, and writes each file through a temp file plus rename.
3. **Decode, layout, render**: this step runs in `spawn_blocking`. `CardImage` splits each image into RGB and optional alpha planes. `layout::lay_out` turns the fleet into a `DocumentPlan`, a list of pages of positioned `Element`s in PDF points. `pdf::render` writes the PDF with `pdf-writer`: each image is embedded once, alpha becomes an SMask, and text uses built-in Helvetica with WinAnsi encoding. `pdf/helvetica.rs` holds the encoding and the width table used for right-aligned and centered text.

The layout in `src/layout/fleet_layout.rs` uses US Letter pages with 36 pt margins and card sizes hard-coded as constants. It places blocks in builder order with a next-fit shelf (`src/layout/shelf.rs`): a block goes right of the previous one if it fits on the line (546 pt, 3 pt gap), otherwise on a new line under the tallest block of the current one (10 pt gap), otherwise on a new page.
- **Ships:** 200 pt wide, with upgrades 100×150 pt to their right, 3 per row. A ship with upgrades takes a whole line; ships without upgrades sit side by side.
- **Squadrons:** 180 pt cards, three per line, with a white count badge on each.
- **Objectives:** 180 pt cards on a page of their own.
- **Cleanup:** empty pages are dropped.
- **Footer:** "<name> (squadron pts/total) - site" and "page N sur M".

`web::WebError` maps each failure to an HTTP status and a French message on `src/web/error.html`, and logs the English error chain (`error_chain`). The HTML pages are static files embedded with `include_str!`.

`tests/fixtures/fleet_207536.html` is a real saved fleet page used by the parsing tests.

## misc/

`misc/final_data/units.json` (faction → unit name → `{name, image, points}`) and `misc/final_data/names.json` (faction → ships/squadrons/named-squadrons name lists) hold card data scraped from the fleet builder. `misc/wip/` holds the intermediate files; the Ruby scripts that generated them were removed. The app does not use these files at runtime.

## Language

- Code is English, whatever language the prompt is in: identifiers, function names, comments, docstrings, commit messages and log lines are English, even when the discussion is in French.
- User-facing text is French and stays French: error messages shown to users, CLI output, labels, templates. Keep it UTF-8 with accents. Do not translate it, and write new user-facing text in French.

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
