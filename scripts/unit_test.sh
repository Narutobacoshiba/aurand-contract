#!/usr/bin/env bash

set -euxo pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ROOT_DIR="$SCRIPT_DIR/.."

CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='aurand-test-%p-%m.profraw' cargo test

[ ! -d "$ROOT_DIR/target/coverage" ] && mkdir "$ROOT_DIR/target/coverage"

grcov $ROOT_DIR --binary-path $ROOT_DIR/target/debug/deps/ -s $ROOT_DIR -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" \
--excl-br-start "mod unit_tests \{" --excl-start "mod unit_tests \{"  -o "$ROOT_DIR/target/coverage"

rm -f $ROOT_DIR/*.profraw
rm -f $ROOT_DIR/contracts/aurand/*.profraw