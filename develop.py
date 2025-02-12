#!/usr/bin/env python3

# This is a simple shell script to help developing rsvim.
# Formatted with black/isort.

import argparse
import os
import shutil

__DISABLE_EXTENDED_TOOLS = False


def clippy():
    command = "RUSTFLAGS='-Dwarnings'"
    if shutil.which('sccache') is not None and not __DISABLE_EXTENDED_TOOLS:
        command = f"{command} RUSTC_WRAPPER=$(which sccache)"

    if shutil.which('bacon') is not None and not __DISABLE_EXTENDED_TOOLS:
        command = f"{command} bacon -j clippy-all --headless"
    else:
        command = f"{command} cargo clippy --all-features --workspace"

    print(command)
    os.system(command)


__TEST_NOT_SPECIFIED = '__TEST_NOT_SPECIFIED'


def test(name):
    if name is None:
        name = '--all'

    command = "RUST_LOG=trace"
    if shutil.which('sccache') is not None and not __DISABLE_EXTENDED_TOOLS:
        command = f"{command} RUSTC_WRAPPER=$(which sccache)"

    if shutil.which(
            'cargo-nextest') is not None and not __DISABLE_EXTENDED_TOOLS:
        command = f"{command} cargo nextest run --no-capture {name}"
    else:
        command = f"{command} cargo test {name} -- --nocapture"

    print(command)
    os.system(command)


def list_test():
    command = ""
    if shutil.which('sccache') is not None and not __DISABLE_EXTENDED_TOOLS:
        command = f"{command} RUSTC_WRAPPER=$(which sccache)"

    if shutil.which(
            'cargo-nextest') is not None and not __DISABLE_EXTENDED_TOOLS:
        command = f"{command} cargo nextest list"
    else:
        command = f"{command} cargo test -- --list"

    print(command)
    os.system(command)


parser = argparse.ArgumentParser(
    description='Help running linter/tests for developing rsvim.')
parser.add_argument('-c',
                    '--clippy',
                    action='store_true',
                    help='Run clippy with `RUSTFLAGS=-Dwarnings`')
parser.add_argument(
    '-t',
    '--test',
    nargs='?',
    default=__TEST_NOT_SPECIFIED,
    help='Run [TEST] with `RUST_LOG=trace`, by default run all test cases.')
parser.add_argument('--list-test',
                    action='store_true',
                    help='List all test cases.')
parser.add_argument(
    '--no-extend',
    action='store_true',
    help=
    'Disable third-party extended tools such as `sccache`, `bacon`, `cargo-nextest`, etc.'
)

parser = parser.parse_args()

if parser.no_extended:
    __DISABLE_EXTENDED_TOOLS = True

if parser.clippy:
    clippy()
elif parser.test is None or parser.test != __TEST_NOT_SPECIFIED:
    test(parser.test)
elif parser.list_test:
    list_test()
else:
    print("Error: missing arguments, use -h/--help for more details.")
