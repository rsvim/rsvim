#!/usr/bin/env python3

# Formatted with black/isort.

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

SCCACHE_FULLPATH = shutil.which("sccache")
RECACHE_SCCACHE = False

NO_LLD_LINKER = False
LLD_NAME = None
if WINDOWS:
    LLD_NAME = "lld-link"
elif MACOS:
    LLD_NAME = "ld64.lld"
else:
    LLD_NAME = "ld.lld"
LLD_FULLPATH = shutil.which(LLD_NAME)

RUSTFLAGS = []


def set_env(command, name, value, is_string=False):
    assert isinstance(command, str)
    if WINDOWS:
        logging.info(f"Set env {name}={value}")
        os.environ[name] = value
    else:
        if is_string is True:
            command = f'{command} {name}="{value}"'
        else:
            command = f"{command} {name}={value}"
    return command.strip()


def append_rustflags(opt):
    global RUSTFLAGS
    RUSTFLAGS.append(opt)


def append_lld_rustflags():
    if NO_LLD_LINKER:
        return

    if LLD_FULLPATH is None:
        logging.warning(f"'lld' ({LLD_NAME}) not found!")
        return

    append_rustflags("-Clink-arg=-fuse-ld=lld")


def set_rustflags(command):
    global RUSTFLAGS
    rustflags = " ".join([f for f in RUSTFLAGS])
    command = set_env(command, "RUSTFLAGS", rustflags, is_string=True)
    return command.strip()


def set_sccache(command):
    if SCCACHE_FULLPATH is None:
        logging.warning("'sccache' not found!")
        return command

    if RECACHE_SCCACHE:
        command = set_env(command, "SCCACHE_RECACHE", "1")

    command = set_env(command, "RUSTC_WRAPPER", "sccache")
    return command.strip()


def clippy(watch):
    append_rustflags("-Dwarnings")
    append_lld_rustflags()

    command = set_rustflags("")
    command = set_sccache(command)

    if watch:
        logging.info("Run 'clippy' as a service and watching file changes")
        command = f"{command} bacon -j clippy-all --headless --all-features"
    else:
        logging.info("Run 'clippy' only once")
        command = f"{command} cargo clippy --workspace --all-features --all-targets"

    command = command.strip()
    logging.info(command)
    os.system(command)


def test(name, miri, jobs):
    append_lld_rustflags()

    if len(name) == 0:
        name = None
        logging.info("Run 'test' for all cases")
    else:
        name = " ".join(list(dict.fromkeys(name)))
        logging.info(f"Run 'test' for '{name}'")

    if jobs is None:
        jobs = ""
    else:
        jobs = f" -j {jobs[0]}"

    if miri is not None:
        command = set_env(
            "",
            "MIRIFLAGS",
            "-Zmiri-disable-isolation -Zmiri-permissive-provenance",
            is_string=True,
        )
        command = set_sccache(command)
        command = set_rustflags(command)
        if name is None:
            name = ""
        command = f"{command} cargo +nightly miri nextest run{jobs} -F unicode_lines --no-default-features -p {miri} {name}"
    else:
        command = set_env("", "RSVIM_LOG", "trace")
        command = set_sccache(command)
        command = set_rustflags(command)
        if name is None:
            name = "--all"
        command = f"{command} cargo nextest run{jobs} --no-capture {name}"

    command = command.strip()
    logging.info(command)
    os.system(command)


def list_test():
    append_lld_rustflags()

    command = set_sccache("")
    command = set_rustflags(command)

    command = f"{command} cargo nextest list"

    command = command.strip()
    logging.info(command)
    os.system(command)


def build(release, features, all_features):
    append_lld_rustflags()

    command = set_sccache("")
    command = set_rustflags(command)

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


def fmt():
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
        help="Rebuild all `sccache` caches",
    )
    parser.add_argument(
        "-l",
        "--no-lld",
        dest="no_lld",
        action="store_true",
        help="Build without `lld` linker",
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
    if parser.no_lld:
        NO_LLD_LINKER = True

    if parser.subcommand == "clippy" or parser.subcommand == "c":
        clippy(parser.watch)
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
