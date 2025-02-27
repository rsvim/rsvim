#!/usr/bin/env python3

# This is a simple shell script to help developing rsvim.
# Formatted with black/isort.

import argparse
import os
import pathlib
import shutil

__DISABLE_SCCACHE = False

__TEST_NOT_SPECIFIED = "__TEST_NOT_SPECIFIED"
__BUILD_NOT_SPECIFIED = "__BUILD_NOT_SPECIFIED"
__CLIPPY_NOT_SPECIFIED = "__CLIPPY_NOT_SPECIFIED"


def clippy(mode):
    command = "RUSTFLAGS='-Dwarnings'"
    if shutil.which("sccache") is not None and not __DISABLE_SCCACHE:
        command = f"{command} RUSTC_WRAPPER=$(which sccache)"

    if isinstance(mode, str) and mode.lower().startswith("s"):
        print("Run 'clippy' as background service")
        command = f"{command} bacon -j clippy-all --headless"
    else:
        print("Run 'clippy' only once")
        command = f"{command} cargo clippy --workspace"

    command = command.strip()
    print(command)
    os.system(command)


def test(name):
    if name is None:
        name = "--all"
        print("Run 'test' for all cases")
    else:
        print("Run 'test' for: {name}")

    command = "RUST_LOG=trace"
    if shutil.which("sccache") is not None and not __DISABLE_SCCACHE:
        command = f"{command} RUSTC_WRAPPER=$(which sccache)"

    command = f"{command} cargo nextest run --no-capture {name}"

    command = command.strip()
    print(command)
    os.system(command)


def list_test():
    command = ""
    if shutil.which("sccache") is not None and not __DISABLE_SCCACHE:
        command = f"{command} RUSTC_WRAPPER=$(which sccache)"

    command = f"{command} cargo nextest list"

    command = command.strip()
    print(command)
    os.system(command)


def build(release):
    command = ""
    if shutil.which("sccache") is not None and not __DISABLE_SCCACHE:
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


def start_doc():
    command = "cargo watch -s 'cargo doc && browser-sync start --ss target/doc -s target/doc --directory --no-open'"

    command = command.strip()
    print(command)
    os.system(command)


def release(execute, level):
    cwd_path = pathlib.Path.cwd()
    git_root_path = cwd_path / ".git"
    assert git_root_path.is_dir(), "The $CWD/$PWD must be git repo root!"

    command = f"GIT_CLIFF_CONFIG=$PWD/cliff.toml GIT_CLIFF_WORKDIR=$PWD GIT_CLIFF_REPOSITORY=$PWD GIT_CLIFF_OUTPUT=$PWD/CHANGELOG.md cargo release {level}"
    if execute:
        print("Run 'release' with '--execute' (no dry run), in level: {level}")
        command = f"{command} --execute --no-verify"
    else:
        print("Run 'release' in dry run, in level: {level}")

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
        metavar="SERVICE",
        help="Run clippy with `RUSTFLAGS=-Dwarnings` once or as background service (watch file changes), by default is only once. Use `s(ervice)` to start background service",
    )
    parser.add_argument(
        "-t",
        "--test",
        nargs="?",
        default=__TEST_NOT_SPECIFIED,
        help="Run [TEST] with `RUST_LOG=trace`, by default run all test cases.",
    )
    parser.add_argument("--list-test", action="store_true", help="List all test cases.")
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
        action="store_true",
        help="Start cargo doc service on `http://localhost:3000/rsvim`.",
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
        "--no-cache",
        action="store_true",
        help="Disable `sccache` when building cargo.",
    )

    parser = parser.parse_args()
    # print(parser)

    if parser.no_cache:
        __DISABLE_SCCACHE = True

    if parser.clippy is None or parser.clippy != __CLIPPY_NOT_SPECIFIED:
        clippy(parser.clippy)
    elif parser.test is None or parser.test != __TEST_NOT_SPECIFIED:
        test(parser.test)
    elif parser.list_test:
        list_test()
    elif parser.build is None or parser.build != __BUILD_NOT_SPECIFIED:
        build(parser.build)
    elif parser.doc:
        start_doc()
    elif parser.release:
        release(parser.execute, parser.release)
    else:
        print("Error: missing arguments, use -h/--help for more details.")
