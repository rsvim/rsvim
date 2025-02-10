#!/bin/bash

# This is a simple shell script to help developing rsvim.

# Usage/help
usage() {
	echo "Usage: $0 [-h/--help] [-c/--clippy] [-t/--test TARGET]"
	echo " -h/--help            Display help message and quit."
	echo " -c/--clippy          Run 'cargo clippy' with 'RUSTFLAGS=-Dwarnings'."
	echo "                      Use 'sccache', 'bacon' if exists."
	echo " -t/--test [TARGET]   Run 'cargo test' on [TARGET] with 'RUST_LOG=trace'."
	echo "                      Use 'sccache', 'cargo-nextest' if exists."
}

cargo_clippy() {
	if type "sccache" >/dev/null 2>&1; then
		echo -n "('sccache' exists) "
		export RUSTC_WRAPPER=$(which sccache)
	else
		echo -n "('sccache' not found) "
	fi
	export RUSTFLAGS="-Dwarnings"
	if type "bacon" >/dev/null 2>&1; then
		echo "run 'bacon -j clippy-all'"
		bacon -j clippy-all
	else
		echo "run 'cargo clippy --all-features --workspace'"
		cargo clippy --all-features --workspace
	fi
}

cargo_test() {
	if type "sccache" >/dev/null 2>&1; then
		echo -n "('sccache' exists) "
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

optspec="hct:"
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

	t)
		cargo_test ${OPTARG}
		exit
		;;

	*)
		echo "Error: unknown arguments!"
		usage
		exit
		;;
	esac
done
