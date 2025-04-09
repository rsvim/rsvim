#!/usr/bin/env python3

# This is a simple shell script to help developing rsvim.
# Formatted with black/isort.

import argparse
import os
import pathlib
import platform

__DOC_NOT_SPECIFIED = "__DOC_NOT_SPECIFIED"

WINDOWS = platform.system().startswith("Windows") or platform.system().startswith(
    "CYGWIN_NT"
)


def set_env(command, name, value):
    assert isinstance(command, str)
    if WINDOWS:
        os.environ[name] = value
    else:
        command = f"{command} {name}={value}"
    return command.strip()


def set_sccache(command, recache):
    if recache:
        command = set_env(command, "SCCACHE_RECACHE", "1")
    command = set_env(command, "RUSTC_WRAPPER", "sccache")
    return command.strip()


def clippy(watch, recache):
    command = set_env("", "RUSTFLAGS", "-Dwarnings")
    command = set_sccache(command, recache)

    if watch:
        print("Run 'clippy' as a service and watching file changes")
        command = f"{command} bacon -j clippy-all --headless --all-features"
    else:
        print("Run 'clippy' only once")
        command = f"{command} cargo clippy --workspace --all-features --all-targets"

    command = command.strip()
    print(command)
    os.system(command)


def test(name, recache):
    if len(name) == 0:
        name = ["--all"]
        print("Run 'test' for all cases")
    else:
        name = " ".join(list(dict.fromkeys(name)))
        print(f"Run 'test' for '{name}'")

    command = set_env("", "RSVIM_LOG", "trace")
    command = set_sccache(command, recache)

    command = f"{command} cargo nextest run --no-capture {name}"

    command = command.strip()
    print(command)
    os.system(command)


def list_test(recache):
    command = set_sccache("", recache)

    command = f"{command} cargo nextest list"

    command = command.strip()
    print(command)
    os.system(command)


def build(release, recache):
    command = set_sccache("", recache)

    if release:
        print("Run 'build' for 'release'")
        command = f"{command} cargo build --release"
    else:
        print("Run 'build' for 'debug'")
        command = f"{command} cargo build"

    command = command.strip()
    print(command)
    os.system(command)


def doc(watch):
    command = "cargo doc && browser-sync start --ss target/doc -s target/doc --directory --startPath rsvim --no-open"
    if watch:
        print("Run 'doc' as a service and watching file changes")
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
    subparsers = parser.add_subparsers(dest="subcommand")

    clippy_subparser = subparsers.add_parser(
        "clippy",
        aliases=["c"],
        help="Run `cargo clippy` with `RUSTFLAGS=-Dwarnings`",
    )
    clippy_subparser.add_argument(
        "-w",
        "--watch",
        action="store_true",
        help="Running clippy as a service and watching file changes, by default is `false`",
    )
    clippy_subparser.add_argument(
        "--recache",
        action="store_true",
        help="Rebuild cache in `sccache`",
    )

    test_subparser = subparsers.add_parser(
        "test",
        aliases=["t"],
        help="Run `cargo test` with `RSVIM_LOG=trace`, by default runs all test cases",
    )
    test_subparser.add_argument(
        "-l",
        "--list",
        action="store_true",
        dest="list_test",
        help="List all test cases instead of running them",
    )
    test_subparser.add_argument(
        "name",
        nargs="*",
        default=[],
        help="Multiple test names that need to run, by default is empty (runs all test cases)",
    )
    test_subparser.add_argument(
        "--recache",
        action="store_true",
        help="Rebuild cache in `sccache`",
    )

    build_subparser = subparsers.add_parser(
        "build",
        aliases=["b"],
        help="Build debug/release target with `sccache`, by default is debug",
    )
    build_subparser.add_argument(
        "-r", "--release", action="store_true", help="Build release target"
    )
    build_subparser.add_argument(
        "--recache",
        action="store_true",
        help="Rebuild cache in `sccache`",
    )

    doc_subparser = subparsers.add_parser(
        "doc",
        aliases=["d"],
        help="Start `cargo doc` service on `http://localhost:3000/rsvim`",
    )
    doc_subparser.add_argument(
        "-w",
        "--watch",
        action="store_true",
        help="Running cargo doc as a service and watching file changes, by default is `false`",
    )

    release_subparser = subparsers.add_parser(
        "release",
        aliases=["r"],
        help="Run `cargo release` to publish crates",
    )
    release_subparser.add_argument(
        "level",
        choices=["alpha", "beta", "rc", "major", "minor", "patch"],
        help="Release [LEVEL]",
    )
    release_subparser.add_argument(
        "-e",
        "--execute",
        action="store_true",
        help="Execute `cargo release` with `--no-verify`",
    )

    parser = parser.parse_args()
    print(parser)

    if parser.subcommand == "clippy" or parser.subcommand == "c":
        clippy(parser.watch, parser.recache)
    elif parser.subcommand == "test" or parser.subcommand == "t":
        if parser.list_test:
            list_test(parser.recache)
        else:
            test(parser.name, parser.recache)
    elif parser.subcommand == "build" or parser.subcommand == "b":
        build(parser.release, parser.recache)
    elif parser.subcommand == "doc" or parser.subcommand == "d":
        doc(parser.watch)
    elif parser.release:
        release(parser.execute, parser.release)
    else:
        print("Error: missing arguments, use -h/--help for more details.")
