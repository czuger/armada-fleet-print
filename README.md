# Armada Fleet Print

Turns a fleet saved on [armada.ryankingston.com](https://armada.ryankingston.com/) into a printable PDF of its card images.

## Usage

Go to https://apps.ieroe.com/armada/ and paste the link of a saved fleet, for example `https://armada.ryankingston.com/fleet/207536/`. The site is in French; an English version is available on request.

The PDF uses US Letter pages:

- **Ships:** two per page, each with its upgrade cards alongside.
- **Squadrons:** a 3×2 grid, with a badge showing how many of each card the fleet has.
- **Objectives:** one row after the squadrons.
- **Footer:** the fleet name, squadron and total points, and the page number.

## Running it yourself

You need a Rust toolchain with edition 2024 support (1.85 or later).

```sh
# Web server on http://127.0.0.1:4567
./dev.sh

# One fleet to a PDF file, without the server
cargo run -- render https://armada.ryankingston.com/fleet/207536/ -o flotte.pdf
```

Card images are downloaded from the fleet builder the first time they are needed, then cached on disk.

### Configuration

| Option        | Environment variable | Default          | Purpose                           |
|---------------|----------------------|------------------|-----------------------------------|
| `--bind`      | `AFP_BIND`           | `127.0.0.1:4567` | Address the web server listens on |
| `--socket`    | `AFP_SOCKET`         | none             | Unix socket to listen on instead of `--bind` |
| `--cards-dir` | `AFP_CARDS_DIR`      | `pics`           | Card image cache directory        |
|               | `RUST_LOG`           | `info`           | Log filter                        |

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
