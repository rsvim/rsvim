#!/usr/bin/env python3

# Formatted with ruff.

from abc import abstractmethod
from typing import Protocol
from typing import Optional
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
SKIP_SCCACHE = False

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

    if SKIP_SCCACHE:
        logging.warning("'sccache' is skipped!")
        return

    if RECACHE_SCCACHE:
        set_env("SCCACHE_RECACHE", "1")

    set_env("RUSTC_WRAPPER", SCCACHE_FULLPATH)


def subcommand_clippy():
    append_rustflags("-Dwarnings")
    if WINDOWS:
        append_rustflags("-Csymbol-mangling-version=v0")

    set_rustflags()
    set_sccache()

    command = "cargo clippy --workspace --all-targets"

    command = command.strip()
    logging.info(command)
    os.system(command)


def subcommand_test(name, miri, jobs):
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

    if WINDOWS:
        append_rustflags("-Csymbol-mangling-version=v0")
    set_env("RUST_BACKTRACE", "full")
    if miri:
        set_env(
            "MIRIFLAGS",
            "-Zmiri-backtrace=full -Zmiri-disable-isolation -Zmiri-permissive-provenance",
        )
        if name is None:
            name = ""
        command = f"cargo +nightly miri nextest run{jobs} -F unicode_lines --no-default-features -p rsvim_core {name}"
    else:
        rsvim_log = os.getenv("RSVIM_LOG")
        if isinstance(rsvim_log, str):
            set_env("RSVIM_LOG", rsvim_log)
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


def subcommand_list_test():
    set_sccache()
    set_rustflags()

    command = "cargo nextest list"

    command = command.strip()
    logging.info(command)
    os.system(command)


def subcommand_build(profile, verbose, features):
    set_sccache()
    set_rustflags()

    if profile == "release":
        command = "cargo build --release"
    elif profile == "nightly":
        command = "cargo build --profile nightly"
    else:
        command = "cargo build"

    if verbose:
        command = f"{command} -vv"

    if isinstance(features, list) and len(features) > 0:
        features = ",".join(features)
        command = f"{command} --features {features}"

    command = command.strip()
    logging.info(command)
    os.system(command)


def typos_formatter():
    command = "typos"
    logging.info(command)
    os.system(command)


def rust_formatter():
    command = "cargo +nightly fmt"
    logging.info(command)
    os.system(command)


def taplo_formatter():
    command = "taplo fmt"
    logging.info(command)
    os.system(command)


def prettier_formatter():
    command = "prettier --write *.md ./runtime/*.ts"
    logging.info(command)
    os.system(command)


def tsc_formatter():
    command = "tsc"
    logging.info(command)
    os.system(command)
    for filename in ["00__web.d.ts", "01__rsvim.d.ts"]:
        src_file = f"types/{filename}"
        dest_file = f".{filename}"
        with open(src_file, "r") as src:
            with open(dest_file, "w") as dest:
                dest.write("// @ts-nocheck\n")
                for line in src.readlines():
                    dest.write(line)
        command = f"mv {dest_file} {src_file}"
        logging.info(command)
        os.system(command)


def subcommand_fmt(only):
    formatters = {
        "typos": typos_formatter,
        "rust": rust_formatter,
        "prettier": prettier_formatter,
        "taplo": taplo_formatter,
        "tsc": tsc_formatter,
    }

    if isinstance(only, str) and only in formatters:
        formatters[only]()
    else:
        for formatter in formatters.values():
            formatter()


def subcommand_doc(watch):
    command = "cargo doc && browser-sync start --ss target/doc -s target/doc --directory --startPath rsvim_core --no-open"
    if watch:
        logging.info("Run 'cargo doc' and refresh it on file changes")
        command = f"cargo watch -s '{command}'"
    else:
        logging.info("Run 'cargo doc' only once")

    command = command.strip()
    logging.info(command)
    os.system(command)


def subcommand_release(level, execute):
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


def subcommand_npm_version(level):
    command = f"npm version {level} --no-git-tag-version"
    logging.info(command)
    os.system(command)


class SubCommand(Protocol):
    @abstractmethod
    def run(self, args) -> None:
        raise Exception("Not implemented!")

    @abstractmethod
    def name(self) -> str:
        raise Exception("Not implemented!")

    @abstractmethod
    def alias(self) -> Optional[str]:
        raise Exception("Not implemented!")


# clippy/c
class ClippyCommand(SubCommand):
    def __init__(self, subparsers) -> None:
        self._name = "clippy"
        self._alias = "c"

        self.clippy_parser = subparsers.add_parser(
            self._name,
            aliases=[self._alias],
            help="Run `cargo clippy` with `RUSTFLAGS=-Dwarnings`",
        )

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return self._alias

    def run(self, args) -> None:
        append_rustflags("-Dwarnings")
        if WINDOWS:
            append_rustflags("-Csymbol-mangling-version=v0")

        set_rustflags()
        set_sccache()

        command = "cargo clippy --workspace --all-targets"

        command = command.strip()
        logging.info(command)
        os.system(command)


if __name__ == "__main__":
    logging.basicConfig(format="%(levelname)s: %(message)s", level=logging.INFO)

    parser = argparse.ArgumentParser(description="Rsvim development toolkit")
    parser.add_argument(
        "--recache",
        action="store_true",
        help="Rebuild all `sccache` caches",
    )
    parser.add_argument(
        "--skip-cache",
        action="store_true",
        help="Build without `sccache`",
    )

    subparsers = parser.add_subparsers(dest="subcommand")

    commands = {"clippy"}

    clippy_subparser = subparsers.add_parser(
        "clippy",
        aliases=["c"],
        help="Run `cargo clippy` with `RUSTFLAGS=-Dwarnings`",
    )

    test_subparser = subparsers.add_parser(
        "test",
        aliases=["t"],
        help="Run `cargo test` with by default `RSVIM_LOG=trace` on all test cases",
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
        action="store_true",
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
        "-v", "--verbose", action="store_true", help="Build with '--verbose' option"
    )
    build_subparser.add_argument(
        "-F", "--features", action="append", help="Build with '--features' option"
    )
    build_subparser.add_argument(
        "-r", "--release", action="store_true", help="Build release target"
    )
    build_subparser.add_argument(
        "-n",
        "--nightly",
        action="store_true",
        help="Build nightly target",
    )

    doc_subparser = subparsers.add_parser(
        "doc",
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
        help="Run multiple formatters, checkers and code generators, by default run all tools",
    )
    fmt_subparser.add_argument(
        "--rust",
        action="store_true",
        help="Only run `cargo +nightly fmt`, by default is false",
    )
    fmt_subparser.add_argument(
        "--tsc",
        action="store_true",
        help="Only run `tsc`, by default is false",
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

    npm_subparser = subparsers.add_parser(
        "npm",
        help="Run `npm` commands for bump version.",
    )
    npm_subparser.add_argument(
        "-v",
        "--version",
        choices=["major", "minor", "patch"],
        help="Npm --version [LEVEL]",
    )

    parser = parser.parse_args()
    # print(parser)

    if parser.recache:
        RECACHE_SCCACHE = True
    if parser.skip_cache:
        SKIP_SCCACHE = True

    if parser.subcommand == "clippy" or parser.subcommand == "c":
        subcommand_clippy()
    elif parser.subcommand == "test" or parser.subcommand == "t":
        if parser.list_test:
            subcommand_list_test()
        else:
            subcommand_test(parser.name, parser.miri, parser.job)
    elif parser.subcommand == "build" or parser.subcommand == "b":
        profile = "debug"
        if parser.release:
            profile = "release"
        elif parser.nightly:
            profile = "nightly"
        subcommand_build(profile, parser.verbose, parser.features)
    elif parser.subcommand == "doc":
        subcommand_doc(parser.watch)
    elif parser.subcommand == "fmt":
        if parser.tsc:
            subcommand_fmt("tsc")
        elif parser.rust:
            subcommand_fmt("rust")
        else:
            subcommand_fmt(None)
    elif parser.subcommand == "release" or parser.subcommand == "r":
        subcommand_release(parser.level, parser.execute)
    elif parser.subcommand == "npm":
        subcommand_npm_version(parser.version)
    else:
        logging.error("Missing arguments, use -h/--help for more details.")
