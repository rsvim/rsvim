#!/usr/bin/env python3

# Formatted with black/isort.

import argparse
import logging
import os
import pathlib
import platform
import shutil
import subprocess

WINDOWS = platform.system().startswith("Windows") or platform.system().startswith(
    "CYGWIN_NT"
)
MACOS = platform.system().startswith("Darwin")

SCCACHE_FULLPATH = shutil.which("sccache")
RECACHE_SCCACHE = False
LLD_NAME = None
if WINDOWS:
    LLD_NAME = "lld-link"
elif MACOS:
    LLD_NAME = "ld64.lld"
else:
    LLD_NAME = "ld.lld"
LLD_FULLPATH = shutil.which(LLD_NAME)
USE_LLD_LINKER = False


def set_env(command, name, value):
    assert isinstance(command, str)
    if WINDOWS:
        os.environ[name] = value
    else:
        command = f"{command} {name}={value}"
    return command.strip()


def set_sccache(command):
    if SCCACHE_FULLPATH is None:
        return command
    if RECACHE_SCCACHE:
        command = set_env(command, "SCCACHE_RECACHE", "1")
    command = set_env(command, "RUSTC_WRAPPER", SCCACHE_FULLPATH)
    return command.strip()


def set_lld(command):
    if not USE_LLD_LINKER:
        return command

    if LLD_FULLPATH is None:
        logging.warning(f"'lld' ({LLD_NAME}) is not found")
        return command

    arch = subprocess.check_output(["rustc", "--version", "--verbose"], text=True)
    arch = [l.strip() for l in arch.splitlines()]
    host = [l for l in arch if l.startswith("host:")]
    host = host[0][5:].strip()
    # logging.debug(f"host:{host}")
    cargo_target_rustflags = f"CARGO_TARGET_{host.replace('-', '_').upper()}_RUSTFLAGS"
    command = (
        f'{cargo_target_rustflags}="-C link-arg=-fuse-ld={LLD_FULLPATH}" {command}'
    )
    return command.strip()


def clippy(watch):
    command = set_env("", "RUSTFLAGS", "-Dwarnings")
    command = set_sccache(command)
    command = set_lld(command)

    if watch:
        logging.info("Run 'clippy' as a service and watching file changes")
        command = f"{command} bacon -j clippy-all --headless --all-features"
    else:
        logging.info("Run 'clippy' only once")
        command = f"{command} cargo clippy --workspace --all-features --all-targets"

    command = command.strip()
    logging.info(command)
    os.system(command)


def test(name, miri):
    if len(name) == 0:
        name = None
        logging.info("Run 'test' for all cases")
    else:
        name = " ".join(list(dict.fromkeys(name)))
        logging.info(f"Run 'test' for '{name}'")

    if miri is not None:
        command = set_env(
            "", "MIRIFLAGS", "'-Zmiri-disable-isolation -Zmiri-permissive-provenance'"
        )
        command = set_sccache(command)
        command = set_lld(command)
        if name is None:
            name = ""
        command = f"{command} cargo +nightly miri nextest run -F unicode_lines --no-default-features -p {miri} {name}"
    else:
        command = set_env("", "RSVIM_LOG", "trace")
        command = set_sccache(command)
        command = set_lld(command)
        if name is None:
            name = "--all"
        command = f"{command} cargo nextest run --no-capture {name}"

    command = command.strip()
    logging.info(command)
    os.system(command)


def list_test():
    command = set_sccache("")
    command = set_lld(command)

    command = f"{command} cargo nextest list"

    command = command.strip()
    logging.info(command)
    os.system(command)


def build(release, features, all_features):
    command = set_sccache("")
    command = set_lld(command)

    feature_flags = ""
    if all_features:
        feature_flags = "--all-features"
    elif len(features) > 0:
        feature_flags = " ".join([f"--features {f}" for feat in features for f in feat])

    fmt = lambda ff: "default features" if len(ff) == 0 else ff

    if release:
        logging.info(f"Run 'build' for 'release' with {fmt(feature_flags)}")
        command = f"{command} cargo build --release {feature_flags}"
    else:
        logging.info(f"Run 'build' for 'debug' with {fmt(feature_flags)}")
        command = f"{command} cargo build {feature_flags}"

    command = command.strip()
    logging.info(command)
    os.system(command)


def doc(watch):
    command = "cargo doc && browser-sync start --ss target/doc -s target/doc --directory --startPath rsvim --no-open"
    if watch:
        logging.info("Run 'doc' as a service and watching file changes")
        command = f"cargo watch -s '{command}'"
    else:
        logging.info("Run 'doc' only once")

    command = command.strip()
    logging.info(command)
    os.system(command)


def release(level, execute):
    cwd_path = pathlib.Path.cwd()
    git_root_path = cwd_path / ".git"
    assert git_root_path.is_dir(), "The $CWD/$PWD must be git repo root!"

    command = f"GIT_CLIFF_CONFIG=$PWD/cliff.toml GIT_CLIFF_WORKDIR=$PWD GIT_CLIFF_REPOSITORY=$PWD GIT_CLIFF_OUTPUT=$PWD/CHANGELOG.md cargo release {level}"
    if execute:
        logging.info(f"Run 'release' with '--execute' (no dry run), level: {level}")
        command = f"{command} --execute --no-verify"
    else:
        logging.info(f"Run 'release' in dry run, level: {level}")

    command = command.strip()
    logging.info(command)
    os.system(command)


if __name__ == "__main__":
    logging.basicConfig(format="%(levelname)s: %(message)s", level=logging.INFO)

    parser = argparse.ArgumentParser(
        description="help running linter/tests when developing rsvim"
    )
    parser.add_argument(
        "-r",
        "--recache",
        action="store_true",
        help="Build with `sccache` cache (if available)",
    )
    parser.add_argument(
        "-l",
        "--lld",
        action="store_true",
        help="Build with `lld` linker (if available)",
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
        help="Running clippy as a service and watching file changes, by default is false",
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
        "--miri",
        metavar="PACKAGE",
        help="Run `cargo +nightly miri test` on specified [PACKAGE]",
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
        "-f",
        "--features",
        nargs="+",
        default=[],
        action="append",
        help="Build with specified features",
    )
    build_subparser.add_argument(
        "--all-features",
        dest="all_features",
        action="store_true",
        help="Build with all features",
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
        help="Running cargo doc as a service and watching file changes, by default is false",
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
    logging.debug(parser)

    if parser.recache:
        RECACHE_SCCACHE = True
    if parser.lld:
        USE_LLD_LINKER = True

    if parser.subcommand == "clippy" or parser.subcommand == "c":
        clippy(parser.watch)
    elif parser.subcommand == "test" or parser.subcommand == "t":
        if parser.list_test:
            list_test()
        else:
            test(parser.name, parser.miri)
    elif parser.subcommand == "build" or parser.subcommand == "b":
        build(parser.release, parser.features, parser.all_features)
    elif parser.subcommand == "doc" or parser.subcommand == "d":
        doc(parser.watch)
    elif parser.subcommand == "release" or parser.subcommand == "r":
        release(parser.level, parser.execute)
    else:
        logging.error("Missing arguments, use -h/--help for more details.")
