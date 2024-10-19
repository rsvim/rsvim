#!/bin/bash

set -x

if [ $# -lt 1 ]; then
	echo "usage: ./release.sh [LEVEL] (--execute --no-verify)"
	echo "error: missing release arguments, exit..."
	exit 1
fi

export GIT_CLIFF_CONFIG=$PWD/cliff.toml
export GIT_CLIFF_WORKDIR=$PWD
export GIT_CLIFF_REPOSITORY=$PWD
export GIT_CLIFF_OUTPUT=$PWD/CHANGELOG.md

# git-cliff will create "CHANGELOG.md" for each packages, but we only need the one from rsvim_cli.
cargo release "$@"
