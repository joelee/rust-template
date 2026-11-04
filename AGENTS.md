# AGENTS.md — Rust template repository

This repository is a cargo-generate template. Everything a generated project contains lives under `template/`; `template/AGENTS.md` is the guide for those projects. This file covers maintaining the template itself.

## Commands

| Task | Command |
| --- | --- |
| Install cargo tools (once) | `just setup` |
| Check required tools | `just check-tools` |
| Generate a project | `just generate <name>` |
| Self-test both MSRV profiles | `just self-test` |

Run `just self-test` before handing work back. It generates a project per profile under `target/self-test/` and runs that project's full `just ci`.

## Rules

- Files under `template/` are rendered with Liquid. Placeholders: `{{project-name}}` (kebab-case), `{{crate_name}}` (snake_case), `{{crate_name | upcase}}` (environment prefix), plus `description`, `license` and `msrv_profile` from `template/cargo-generate.toml`.
- Keep Rust sources free of placeholders so they read as normal code. Derive names at runtime from `env!("CARGO_PKG_NAME")` (see `template/src/app_id.rs`). The only exceptions are `use {{crate_name}}::…` lines in `src/main.rs`, `tests/config_example.rs`, and doc-test examples.
- Keep `use {{crate_name}}::…` imports in their own block, separated by a blank line, so `rustfmt` ordering does not depend on the project name.
- `template/justfile` is excluded from Liquid processing because `just` uses `{{ }}` itself. Add any other file that needs literal `{{` or `{%` to `exclude` in `template/cargo-generate.toml`, or wrap the text in `{% raw %}…{% endraw %}`.
- `template/Cargo.lock` is committed with `name = "{{project-name}}"`, so generated projects start with tested dependency versions. When dependencies change, regenerate it in a self-test project and copy it back with that substitution.
- When the pinned toolchain changes, update `template/rust-toolchain.toml` and both `rust-version` values in `template/Cargo.toml` (application = toolchain, library = toolchain minus 2, never below 1.85).
- cargo-generate does not copy symlinks, so `template/CLAUDE.md` is a one-line file that imports `AGENTS.md` (`@AGENTS.md`) rather than a symlink.
- Follow the engineering rules in `template/AGENTS.md` for any code you change under `template/`.
