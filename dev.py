#!/usr/bin/env python3

# Formatted with ruff.

import argparse
import logging
import os
import pathlib
import platform
import shutil

WINDOWS = platform.system().startswith("Windows") or platform.system().startswith(
    "CYGWIN_NT"
)
MACOS = platform.system().startswith("Darwin")
LINUX = not WINDOWS and not MACOS

SCCACHE_FULLPATH = shutil.which("sccache")
RECACHE_SCCACHE = False
NO_SCCACHE = False

RUSTFLAGS = []


def set_env(name, value):
    logging.info(f"Set env {name}={value}")
    os.environ[name] = value


def append_rustflags(opt):
    global RUSTFLAGS
    RUSTFLAGS.append(opt)


def set_rustflags():
    global RUSTFLAGS
    if len(RUSTFLAGS) > 0:
        rustflags = " ".join([f for f in RUSTFLAGS])
        set_env("RUSTFLAGS", rustflags)


def set_sccache():
    if SCCACHE_FULLPATH is None:
        logging.warning("'sccache' not found!")
        return

    if NO_SCCACHE:
        logging.warning("'sccache' is disabled by '-c'/'--no-cache' option!")
        return

    if RECACHE_SCCACHE:
        set_env("SCCACHE_RECACHE", "1")

    set_env("RUSTC_WRAPPER", SCCACHE_FULLPATH)


def clippy():
    append_rustflags("-Dwarnings")
    set_rustflags()
    set_sccache()

    command = "cargo clippy --workspace --all-features --all-targets"

    command = command.strip()
    logging.info(command)
    os.system(command)


def test(name, miri, jobs):
    if len(name) == 0:
        name = None
        logging.info("Run 'cargo test' for all tests")
    else:
        name = " ".join(list(dict.fromkeys(name)))
        logging.info(f"Run 'cargo test' for tests: {name}")

    if jobs is None:
        jobs = ""
    else:
        jobs = f" -j {jobs[0]}"

    if miri is not None:
        set_env(
            "MIRIFLAGS",
            "-Zmiri-disable-isolation -Zmiri-permissive-provenance",
        )
        if name is None:
            name = ""
        command = f"cargo +nightly miri nextest run{jobs} -F unicode_lines --no-default-features -p {miri} {name}"
    else:
        set_env("RSVIM_LOG", "trace")
        set_sccache()
        set_rustflags()
        if name is None:
            name = "--all"
        command = f"cargo nextest run{jobs} --no-capture {name}"

    command = command.strip()
    logging.info(command)
    os.system(command)


def list_test():
    set_sccache()
    set_rustflags()

    command = "cargo nextest list"

    command = command.strip()
    logging.info(command)
    os.system(command)


def build(release, features, all_features):
    set_sccache()
    set_rustflags()

    feature_flags = ""
    if all_features:
        feature_flags = "--all-features"
    elif len(features) > 0:
        feature_flags = " ".join([f"--features {f}" for feat in features for f in feat])

    def show_feat(ff):
        if len(ff) == 0:
            return "default"
        else:
            return ff

    if release:
        logging.info(
            f"Run 'cargo build --release' with features: {show_feat(feature_flags)}"
        )
        command = f"cargo build --release {feature_flags}"
    else:
        logging.info(
            f"Run 'cargo build --debug' with features: {show_feat(feature_flags)}"
        )
        command = f"cargo build {feature_flags}"

    command = command.strip()
    logging.info(command)
    os.system(command)


def fmt():
    command = "typos"
    logging.info(command)
    os.system(command)

    command = "cargo fmt"
    logging.info(command)
    os.system(command)

    command = "taplo fmt"
    logging.info(command)
    os.system(command)

    command = "prettier --write *.md **/*.ts"
    logging.info(command)
    os.system(command)

    command = "tsc"
    logging.info(command)
    os.system(command)


def doc(watch):
    command = "cargo doc && browser-sync start --ss target/doc -s target/doc --directory --startPath rsvim_core --no-open"
    if watch:
        logging.info("Run 'cargo doc' and refresh it on file changes")
        command = f"cargo watch -s '{command}'"
    else:
        logging.info("Run 'cargo doc' only once")

    command = command.strip()
    logging.info(command)
    os.system(command)


def release(level, execute):
    cwd_path = pathlib.Path.cwd()
    git_root_path = cwd_path / ".git"
    assert git_root_path.is_dir(), "The $CWD/$PWD must be git repo root!"

    command = f"GIT_CLIFF_CONFIG=$PWD/cliff.toml GIT_CLIFF_WORKDIR=$PWD GIT_CLIFF_REPOSITORY=$PWD GIT_CLIFF_OUTPUT=$PWD/CHANGELOG.md cargo release {level}"
    if execute:
        logging.info(f"Execute 'cargo release' with level: {level}")
        command = f"{command} --execute --no-verify"
    else:
        logging.info(f"Dry run 'cargo release' with level: {level}")

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
        help="Rebuild all `sccache` caches",
    )
    parser.add_argument(
        "-c",
        "--no-cache",
        action="store_true",
        help="Build without `sccache`",
    )

    subparsers = parser.add_subparsers(dest="subcommand")

    clippy_subparser = subparsers.add_parser(
        "clippy",
        aliases=["c"],
        help="Run `cargo clippy` with `RUSTFLAGS=-Dwarnings`",
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
    test_subparser.add_argument(
        "-j",
        "--job",
        nargs=1,
        metavar="N",
        help="Run `cargo nextest run` with N threads",
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

    fmt_subparser = subparsers.add_parser(
        "fmt",
        aliases=["f"],
        help="Run multiple formatters and code-generator: `cargo fmt`, `taplo fmt`, `prettier`, `tsc`",
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
    # print(parser)

    if parser.recache:
        RECACHE_SCCACHE = True
    if parser.no_cache:
        NO_SCCACHE = True

    if parser.subcommand == "clippy" or parser.subcommand == "c":
        clippy()
    elif parser.subcommand == "test" or parser.subcommand == "t":
        if parser.list_test:
            list_test()
        else:
            test(parser.name, parser.miri, parser.job)
    elif parser.subcommand == "build" or parser.subcommand == "b":
        build(parser.release, parser.features, parser.all_features)
    elif parser.subcommand == "doc" or parser.subcommand == "d":
        doc(parser.watch)
    elif parser.subcommand == "fmt" or parser.subcommand == "f":
        fmt()
    elif parser.subcommand == "release" or parser.subcommand == "r":
        release(parser.level, parser.execute)
    else:
        logging.error("Missing arguments, use -h/--help for more details.")
