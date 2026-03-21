#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

if [[ $# -ne 0 ]]; then
    echo "Usage: ./publish.sh" >&2
    exit 1
fi

publish_crate() {
    local crate="$1"
    local output
    local status

    echo "==> Publishing $crate"

    set +e
    output="$(cargo publish -p "$crate" --locked 2>&1)"
    status=$?
    set -e

    printf '%s\n' "$output"

    if [[ $status -eq 0 ]]; then
        return 0
    fi

    if [[ "$output" == *"already exists on crates.io index"* ]]; then
        echo "==> Skipping $crate: version already published"
        return 0
    fi

    return "$status"
}

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

publish_crate english-core
publish_crate english
