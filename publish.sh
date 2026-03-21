#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

if [[ $# -ne 0 ]]; then
    echo "Usage: ./publish.sh" >&2
    exit 1
fi

echo "==> Running tests"
cargo test --quiet

echo "==> Packaging english-core"
cargo package -p english-core --locked --allow-dirty

echo "==> Checking english package contents"
# english depends on the just-bumped english-core version, so full cargo package
# verification tries to resolve crates.io before english-core has been published.
# Listing the package contents still checks what will ship without needing the
# unpublished dependency version to exist remotely.
cargo package --list -p english --locked --allow-dirty

echo "==> Publishing english-core"
cargo publish -p english-core --locked

echo "==> Publishing english"
cargo publish -p english --locked
