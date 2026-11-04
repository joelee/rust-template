# Configuration

## Layers

Effective configuration is built from four layers, highest precedence first:

1. CLI flags (only keys that have a flag)
2. Per-key environment overrides: `{{crate_name | upcase}}__<SECTION>__<KEY>`
3. The selected config file
4. Built-in defaults, so the application runs with no config file

Run `{{project-name}} config show` to see the effective value of every key and which layer supplied it.

## Config file selection

The first source that exists wins; files are never merged.

1. `--config <path>`: error (exit 78) if the file does not exist
2. `{{crate_name | upcase}}_CONFIG_FILE`: error if the file does not exist; an empty value counts as unset
3. `$XDG_CONFIG_HOME/{{project-name}}/config.toml`: only when `XDG_CONFIG_HOME` is an absolute path
4. `$HOME/.config/{{project-name}}/config.toml`: also on macOS
5. `/etc/{{project-name}}/config.toml`
6. `./config.toml`
7. None found: defaults plus overrides only

Run `{{project-name}} config paths` to see the search order and the selected file.

## Keys

`config.example.toml` lists every key with its default; a test keeps it in sync with the schema.

| Key | Type | Default | Environment override | CLI flag | Description |
| --- | --- | --- | --- | --- | --- |
| `log.level` | string: `error`, `warn`, `info`, `debug`, `trace` | `"info"` | `{{crate_name | upcase}}__LOG__LEVEL` | `--log-level` | Minimum level written to stderr. |
| `greeting.message` | string | `"Hello"` | `{{crate_name | upcase}}__GREETING__MESSAGE` | | Greeting word. |
| `greeting.repeat` | integer (0 or more) | `1` | `{{crate_name | upcase}}__GREETING__REPEAT` | | Times to print each greeting. |
| `greeting.names` | array of strings | `["world"]` | `{{crate_name | upcase}}__GREETING__NAMES` | `greet --name` (repeatable) | Names to greet. |

## Environment override rules

- Variables start with `{{crate_name | upcase}}__`; each further `__` separates a table level. Matching after the prefix is case-insensitive.
- Values are parsed as the key's type: strings are taken literally, booleans and numbers use TOML syntax, arrays use TOML inline arrays such as `["a", "b"]`. An integer is accepted for a float key.
- These are startup errors (exit 78) that name the variable: an unknown key, a key that is a whole table, an unparseable value, a value of the wrong type or out of range, and an empty value for a non-string key.
- An override value must be a single TOML value; extra lines are rejected.

## Secrets

Secrets are read from environment variables only and are never part of the config file or overrides. For local development they can live in `.env` (git-ignored); real environment variables take precedence. `config show` reports only whether each secret is set.

| Variable | Required | Description |
| --- | --- | --- |
| `{{crate_name | upcase}}_API_TOKEN` | No | Example secret; replace with your own. |

Keep `.env.example` listing every supported secret with a safe placeholder value.
