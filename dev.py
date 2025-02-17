#!/usr/bin/env python3

# This is a simple shell script to help developing rsvim.
# Formatted with black/isort.

import argparse
import os
import shutil

__DISABLE_SCCACHE = False

__TEST_NOT_SPECIFIED = "__TEST_NOT_SPECIFIED"
__BUILD_NOT_SPECIFIED = "__BUILD_NOT_SPECIFIED"


def clippy():
    command = "RUSTFLAGS='-Dwarnings'"
    if shutil.which("sccache") is not None and not __DISABLE_SCCACHE:
        command = f"{command} RUSTC_WRAPPER=$(which sccache)"

    command = f"{command} bacon -j clippy-all --headless"

    command = command.strip()
    print(command)
    os.system(command)


def test(name):
    if name is None:
        name = "--all"

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
        command = f"{command} cargo build --release"
    else:
        command = f"{command} cargo build"

    command = command.strip()
    print(command)
    os.system(command)


def start_doc():
    command = "cargo watch -s 'cargo doc && browser-sync start --ss target/doc -s target/doc --directory --no-open'"

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
        action="store_true",
        help="Run clippy with `RUSTFLAGS=-Dwarnings`",
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
        "--no-cache",
        action="store_true",
        help="Disable `sccache` when building cargo.",
    )

    parser = parser.parse_args()
    # print(parser)

    if parser.no_cache:
        __DISABLE_SCCACHE = True

    if parser.clippy:
        clippy()
    elif parser.test is None or parser.test != __TEST_NOT_SPECIFIED:
        test(parser.test)
    elif parser.list_test:
        list_test()
    elif parser.build is None or parser.build != __BUILD_NOT_SPECIFIED:
        build(parser.build)
    elif parser.doc:
        start_doc()
    else:
        print("Error: missing arguments, use -h/--help for more details.")
