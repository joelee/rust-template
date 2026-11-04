# {{project-name}}

{{description}}

## Quick start

```bash
just setup          # once: cargo tools, Cargo.lock, git hooks
cargo run -- greet --name Ada
cargo run -- config show
just ci             # full quality gate, same as CI
```

Requires Linux or macOS with `rustup`, `just`, `gitleaks` and `pre-commit`. The toolchain version is pinned in `rust-toolchain.toml` and installed automatically by `rustup`.

## Usage

```text
{{project-name}} [--config PATH] [--log-level LEVEL] [COMMAND]

Commands:
  greet          Print a greeting (default)
  config show    Show the effective configuration and where each value came from
  config paths   List the config file search order
```

Configuration comes from CLI flags, `{{crate_name | upcase}}__SECTION__KEY` environment variables, a TOML config file, and built-in defaults, in that order of precedence. See [docs/configuration.md](docs/configuration.md).

## Project facts

| Item | Value |
| --- | --- |
| MSRV profile | {{msrv_profile}} (see `AGENTS.md`) |
| CI platform | _decide per project; the pipeline runs `just ci`_ |
| Licence | {{license}} |

## Documentation

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Directory structure](docs/directory-structure.md)
- [Developer guide](docs/developer-guide.md)
- [Backlog](docs/backlog.md)
- [Changelog](CHANGELOG.md)
