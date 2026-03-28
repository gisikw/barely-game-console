# === Project: barely-game-console ===
#
# Justfile conventions (see exocortex/scripts/justfile.template):
#
#   Universal:  develop, test, build, check, fmt, ship
#   Optional:   dev, clean
#
#   `ship` commits, pushes, and lets CI handle deployment.
#   `build` is the fast iterative check ("does the compiler love this?").
#   Deployment is a CI concern — see .forgejo/workflows/deploy.yml.

default:
    @just --list

# --- Environment ---

# Enter nix development shell
develop:
    nix develop

# --- Quality ---

# Run tests
test:
    cargo test

# Format code
fmt:
    cargo fmt

# Run all quality checks
check:
    cargo fmt -- --check
    cargo test

# --- Build ---

# Build the project (iterative/dev build — "does the compiler love this?")
build:
    cargo build

# --- Shipping ---

# Commit and push. CI handles deployment.
ship message="ship":
    #!/usr/bin/env bash
    set -euo pipefail
    # Commit if tree is dirty
    if ! git diff --quiet HEAD 2>/dev/null \
        || ! git diff --cached --quiet 2>/dev/null \
        || [ -n "$(git ls-files --others --exclude-standard)" ]; then
        git add -A
        git commit -m "{{message}}"
    fi
    git push
