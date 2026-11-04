# Rust project template

[![Self-test](https://github.com/joelee/rust-template/actions/workflows/self-test.yml/badge.svg)](https://github.com/joelee/rust-template/actions/workflows/self-test.yml) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A [cargo-generate](https://github.com/cargo-generate/cargo-generate) template for Rust command-line applications and libraries on Linux and macOS. Generated projects come with layered configuration (CLI → environment → TOML file → defaults), a `config show` command that explains where every value came from, secrets kept out of config, and a single `just ci` quality gate covering formatting, clippy, nextest with 80% line coverage, doc-tests, docs, cargo-deny, gitleaks and a release build.

## Generate a project

```bash
cargo install cargo-generate
cargo generate gh:joelee/rust-template template --name my-app
```

The long form is equivalent:

```bash
cargo generate --git https://github.com/joelee/rust-template.git template --name my-app
```

To use a short name, add a favourite to `~/.cargo/cargo-generate.toml` and run `cargo generate rust --name my-app`:

```toml
[favorites.rust]
git = "https://github.com/joelee/rust-template.git"
subfolder = "template"
```

From a local clone, use `--path /path/to/rust-template/template` instead of the git source.

You are asked for:

| Placeholder | Default | Effect |
| --- | --- | --- |
| `description` | A Rust command-line application | `Cargo.toml`, README, AGENTS.md |
| `license` | MIT OR Apache-2.0 | `Cargo.toml` licence; review `deny.toml` to match |
| `msrv_profile` | application | `application`: MSRV equals the pinned toolchain. `library`: MSRV below it, checked with `just msrv` |

Then, in the new project:

```bash
just setup   # cargo tools, canonical Cargo.lock, git hooks
just ci      # full gate
git add -A && git commit -m "Initial commit"
```

## What you get

See [template/README.md](template/README.md) for the generated project's own README, and [template/AGENTS.md](template/AGENTS.md) for the engineering rules agents and humans follow in it.

## Maintaining the template

```bash
just setup       # once: cargo-generate, cargo-nextest, cargo-llvm-cov, cargo-deny
brew install gitleaks pre-commit
just check-tools # confirm everything the self-test needs is installed
just self-test   # generate one project per MSRV profile and run `just ci` in each
```

Edit files under `template/`. Rules for keeping it generatable are in [AGENTS.md](AGENTS.md). GitHub Actions runs the same self-test on Linux and macOS for every push to `main`, every pull request, and weekly ([workflow](.github/workflows/self-test.yml)).

## License

This template is released under the [MIT License](LICENSE). Projects you generate from it are yours: each one sets its own licence through the `license` placeholder.
