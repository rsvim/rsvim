#!/usr/bin/env python3

# This is a simple shell script to help developing rsvim.
# Formatted with black/isort.

import argparse
import os
import pathlib
import shutil

__TEST_NOT_SPECIFIED = "__TEST_NOT_SPECIFIED"
__BUILD_NOT_SPECIFIED = "__BUILD_NOT_SPECIFIED"
__CLIPPY_NOT_SPECIFIED = "__CLIPPY_NOT_SPECIFIED"
__DOC_NOT_SPECIFIED = "__DOC_NOT_SPECIFIED"


def clippy(mode, recache):
    command = "RUSTFLAGS='-Dwarnings'"
    if shutil.which("sccache") is not None:
        if recache:
            command = f"{command} SCCACHE_RECACHE=1"
        command = f"{command} RUSTC_WRAPPER=$(which sccache)"

    if isinstance(mode, str) and mode.lower().startswith("w"):
        print("Run 'clippy' as service")
        command = f"{command} bacon -j clippy-all --headless --all-features"
    else:
        print("Run 'clippy' only once")
        command = f"{command} cargo clippy --workspace --all-features --all-targets"

    command = command.strip()
    print(command)
    os.system(command)


def test(name, recache):
    if name is None:
        name = "--all"
        print("Run 'test' for all cases")
    else:
        print(f"Run 'test' for '{name}'")

    command = "RSVIM_LOG=trace"
    if shutil.which("sccache") is not None:
        if recache:
            command = f"{command} SCCACHE_RECACHE=1"
        command = f"{command} RUSTC_WRAPPER=$(which sccache)"

    command = f"{command} cargo nextest run --no-capture {name}"

    command = command.strip()
    print(command)
    os.system(command)


def list_test(recache):
    command = ""
    if shutil.which("sccache") is not None:
        if recache:
            command = f"{command} SCCACHE_RECACHE=1"
        command = f"{command} RUSTC_WRAPPER=$(which sccache)"

    command = f"{command} cargo nextest list"

    command = command.strip()
    print(command)
    os.system(command)


def build(release, recache):
    command = ""
    if shutil.which("sccache") is not None:
        if recache:
            command = f"{command} SCCACHE_RECACHE=1"
        command = f"{command} RUSTC_WRAPPER=$(which sccache)"

    if isinstance(release, str) and release.lower().startswith("r"):
        print("Run 'build' for 'release'")
        command = f"{command} cargo build --release"
    else:
        print("Run 'build' for 'debug'")
        command = f"{command} cargo build"

    command = command.strip()
    print(command)
    os.system(command)


def doc(mode):

    command = "cargo doc && browser-sync start --ss target/doc -s target/doc --directory --startPath rsvim --no-open"
    if isinstance(mode, str) and mode.lower().startswith("w"):
        print("Run 'doc' as service")
        command = f"cargo watch -s '{command}'"
    else:
        print("Run 'doc' only once")

    command = command.strip()
    print(command)
    os.system(command)


def release(execute, level):
    cwd_path = pathlib.Path.cwd()
    git_root_path = cwd_path / ".git"
    assert git_root_path.is_dir(), "The $CWD/$PWD must be git repo root!"

    command = f"GIT_CLIFF_CONFIG=$PWD/cliff.toml GIT_CLIFF_WORKDIR=$PWD GIT_CLIFF_REPOSITORY=$PWD GIT_CLIFF_OUTPUT=$PWD/CHANGELOG.md cargo release {level}"
    if execute:
        print(f"Run 'release' with '--execute' (no dry run), in level: {level}")
        command = f"{command} --execute --no-verify"
    else:
        print(f"Run 'release' in dry run, in level: {level}")

    command = command.strip()
    print(command)
    os.system(command)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Help running linter/tests for developing rsvim."
    )
    parser.add_argument(
        "-c",
        "--clippy",
        nargs="?",
        default=__CLIPPY_NOT_SPECIFIED,
        metavar="WATCH",
        help="Run clippy with `RUSTFLAGS=-Dwarnings` once or as a service (watch file changes and run again), by default is only once. Use `w(atch)` to start service.",
    )
    parser.add_argument(
        "-t",
        "--test",
        nargs="?",
        default=__TEST_NOT_SPECIFIED,
        help="Run [TEST] with `RSVIM_LOG=trace`, by default run all test cases.",
    )
    parser.add_argument(
        "-l", "--list-test", action="store_true", help="List all test cases."
    )
    parser.add_argument(
        "-b",
        "--build",
        nargs="?",
        default=__BUILD_NOT_SPECIFIED,
        metavar="TARGET",
        help="Build debug/release [TARGET], by default is debug. Use `r(elease)` to build release.",
    )
    parser.add_argument(
        "-d",
        "--doc",
        nargs="?",
        default=__DOC_NOT_SPECIFIED,
        metavar="WATCH",
        help="Start cargo doc service on `http://localhost:3000/rsvim`, build document for once or as a service (watch file changes and build again), by default is only once. Use `w(atch)` to start service.",
    )
    parser.add_argument(
        "-r",
        "--release",
        choices=["alpha", "beta", "rc", "major", "minor", "patch"],
        help="Release cargo crates with [LEVEL].",
    )
    parser.add_argument(
        "-e",
        "--execute",
        action="store_true",
        help="Release cargo crates with additional `--execute --no-verify` option.",
    )
    parser.add_argument(
        "--recache",
        action="store_true",
        help="Rebuild cache for `sccache`.",
    )

    parser = parser.parse_args()
    # print(parser)

    if parser.clippy is None or parser.clippy != __CLIPPY_NOT_SPECIFIED:
        clippy(parser.clippy, parser.recache)
    elif parser.test is None or parser.test != __TEST_NOT_SPECIFIED:
        test(parser.test, parser.recache)
    elif parser.list_test:
        list_test(parser.recache)
    elif parser.build is None or parser.build != __BUILD_NOT_SPECIFIED:
        build(parser.build, parser.recache)
    elif parser.doc is None or parser.doc != __DOC_NOT_SPECIFIED:
        doc(parser.doc)
    elif parser.release:
        release(parser.execute, parser.release)
    else:
        print("Error: missing arguments, use -h/--help for more details.")
