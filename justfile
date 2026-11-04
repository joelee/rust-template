set shell := ["bash", "-euo", "pipefail", "-c"]

template := justfile_directory() / "template"
out := justfile_directory() / "target" / "self-test"

# List available recipes
default:
    @just --list

# Install the cargo tools the template and generated projects need
setup:
    cargo install --locked cargo-generate cargo-nextest cargo-llvm-cov cargo-deny
    @command -v gitleaks >/dev/null 2>&1 || echo "Also install gitleaks with your package manager, e.g. brew install gitleaks"
    @command -v pre-commit >/dev/null 2>&1 || echo "Also install pre-commit with your package manager, e.g. brew install pre-commit"

# Fail early, listing every tool the self-test needs that is missing
check-tools:
    #!/usr/bin/env bash
    set -uo pipefail
    missing=()
    for sub in generate nextest llvm-cov deny; do
        cargo "$sub" --version >/dev/null 2>&1 || missing+=("cargo-$sub")
    done
    for bin in rustup git gitleaks; do
        command -v "$bin" >/dev/null 2>&1 || missing+=("$bin")
    done
    if (( ${#missing[@]} )); then
        echo "Missing tools: ${missing[*]}" >&2
        echo "Run 'just setup' for the cargo tools; install the others with your package manager (e.g. brew install gitleaks)." >&2
        exit 1
    fi
    echo "All tools present."

# Generate a project from the template, prompting for placeholders
generate name dest=invocation_directory():
    @cargo generate --version >/dev/null 2>&1 || { echo "cargo-generate is not installed: run 'just setup'" >&2; exit 1; }
    cargo generate --path {{template}} --name {{name}} --destination {{dest}}

# Generate one throwaway project per MSRV profile and run its full gate
self-test: check-tools (self-test-one "demo-app" "application") (self-test-one "acme-tool" "library")

# Generate NAME with PROFILE under target/self-test and run `just ci` in it
self-test-one name profile:
    rm -rf {{out}}/{{profile}}
    mkdir -p {{out}}/{{profile}}
    cargo generate --path {{template}} --name {{name}} --destination {{out}}/{{profile}} \
        --silent --force-git-init \
        --define description="Template self-test" \
        --define license="MIT" \
        --define msrv_profile={{profile}}
    cd {{out}}/{{profile}}/{{name}} && cargo fetch && git add -A \
        && git -c user.name=self-test -c user.email=self-test@example.invalid commit -qm "self-test" \
        && just ci
