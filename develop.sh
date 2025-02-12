#!/bin/bash

# This is a simple shell script to help developing rsvim.

usage() {
  echo "Usage: $0 [-h] [-c] [-l] [-r TEST]"
  echo "  -h          Display help message and quit."
  echo "  -c          Run clippy with 'RUSTFLAGS=-Dwarnings'."
  echo "  -l          List all test cases."
  echo "  -r [TEST]   Run [TEST] with 'RUST_LOG=trace'."
  echo "              Note: use '--all' to run all tests."
}

cargo_clippy() {
  if type "sccache" >/dev/null 2>&1; then
    echo -n "(using 'sccache') "
    export RUSTC_WRAPPER=$(which sccache)
  else
    echo -n "('sccache' not found) "
  fi
  export RUSTFLAGS="-Dwarnings"
  if type "bacon" >/dev/null 2>&1; then
    echo "run 'bacon -j clippy-all --headless'"
    bacon -j clippy-all --headless
  else
    echo "run 'cargo clippy --all-features --workspace'"
    cargo clippy --all-features --workspace
  fi
}

cargo_test_run() {
  if type "sccache" >/dev/null 2>&1; then
    echo -n "(using 'sccache') "
    export RUSTC_WRAPPER=$(which sccache)
  else
    echo -n "('sccache' not found) "
  fi
  export RUST_LOG=trace
  if type "cargo-nextest" >/dev/null 2>&1; then
    echo "run 'cargo nextest run $1 --no-capture'"
    cargo nextest run $1 --no-capture
  else
    echo "run 'cargo test $1 -- --nocapture'"
    cargo test $1 -- --nocapture
  fi
}

cargo_test_list() {
  if type "sccache" >/dev/null 2>&1; then
    echo -n "(using 'sccache') "
    export RUSTC_WRAPPER=$(which sccache)
  else
    echo -n "('sccache' not found) "
  fi
  if type "cargo-nextest" >/dev/null 2>&1; then
    echo "run 'cargo nextest list'"
    cargo nextest list
  else
    echo "run 'cargo test -- --list'"
    cargo test -- --list
  fi
}

optspec="hclr:"
while getopts "$optspec" optchar; do
  case "${optchar}" in
  h)
    usage
    exit
    ;;
  c)
    cargo_clippy
    exit
    ;;
  l)
    cargo_test_list ${OPTARG}
    exit
    ;;
  r)
    cargo_test_run ${OPTARG}
    exit
    ;;
  *)
    echo "Error: unknown arguments!"
    usage
    exit
    ;;
  esac
done

echo "Error: missing arguments!"
usage
exit
