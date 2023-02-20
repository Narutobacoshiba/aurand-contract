#!/usr/bin/env bash

set -euxo pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ROOT_DIR="$SCRIPT_DIR/.."

cd "$ROOT_DIR/integration-test/packages/aura-test-tube/libauratesttube" && go build

cd "$ROOT_DIR/integration-test/packages/aura-test-tube" && cargo build

cd "$ROOT_DIR/integration-test/packages" && cargo test