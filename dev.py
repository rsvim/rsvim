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

SCCACHE = shutil.which("sccache")
NO_CACHE = False

RUSTFLAGS = []


def set_env(name, value):
    logging.info(f"Set env: {name}={value}")
    os.environ[name] = value


def run(cmd):
    assert isinstance(cmd, list)
    cmd = " ".join(cmd)
    logging.info(cmd)
    os.system(cmd)


def env(name, value):
    os.environ[name] = value
    logging.info(f"Set {name}={value}")


def sccache():
    if SCCACHE is None:
        logging.warning("'sccache' not found!")
        return None
    if NO_CACHE:
        logging.warning("'sccache' is disabled!")
        return None
    return env("RUSTC_WRAPPER", f'"{SCCACHE}"')


def rustflags(flags):
    assert isinstance(flags, list)
    if len(flags) == 0:
        flags = ["-Dwarnings"]
    if WINDOWS:
        flags.append("-Csymbol-mangling-version=v0")
    flags = " ".join(flags)
    return env("RUSTFLAGS", f'"{flags}"')


def append_rustflags(opt):
    global RUSTFLAGS
    RUSTFLAGS.append(opt)


def set_rustflags():
    global RUSTFLAGS
    if len(RUSTFLAGS) > 0:
        rustflags = " ".join([f for f in RUSTFLAGS])
        set_env("RUSTFLAGS", rustflags)


def set_sccache():
    if SCCACHE is None:
        logging.warning("'sccache' not found!")
        return

    if NO_CACHE:
        logging.warning("'sccache' is disabled!")
        return

    set_env("RUSTC_WRAPPER", SCCACHE)


class Cmd(Protocol):
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
class Clippy(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "clippy"
        self._alias = "c"

        self.clippy_parser = subparsers.add_parser(
            self._name,
            aliases=[self._alias],
            help="cargo clippy",
        )

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return self._alias

    def run(self, args) -> None:
        cmd = [sccache(), rustflags([]), "cargo clippy --workspace --all-targets"]
        run(cmd)


# test/t
class Test(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "test"
        self._alias = "t"

        self.test_parser = subparsers.add_parser(
            self._name,
            aliases=[self._alias],
            help="Run `cargo test`, by default on all test cases",
        )
        self.test_parser.add_argument(
            "-l",
            "--list",
            action="store_true",
            dest="list_test",
            help="Only list all test cases, instead of run them",
        )
        self.test_parser.add_argument(
            "name",
            nargs="*",
            default=[],
            help="Only run these tests, by default is empty (e.g. run all test cases)",
        )
        self.test_parser.add_argument(
            "--miri",
            action="store_true",
            help="Run `cargo +nightly miri test`",
        )
        self.test_parser.add_argument(
            "-j",
            "--job",
            nargs=1,
            metavar="N",
            help="Run with N threads, by default 1",
        )

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return self._alias

    def run(self, args) -> None:
        if args.list_test:
            self.list()
            return

        if args.job is None:
            jobs = ""
        else:
            jobs = f" -j {args.job[0]}"

        if args.miri:
            self.miri(jobs)
        else:
            self.test(args.name, jobs)

    def test(self, name, jobs) -> None:
        if len(name) == 0:
            name = "--all"
            logging.info("Run 'cargo test' for all tests")
        else:
            name = " ".join(list(dict.fromkeys(name)))
            logging.info(f"Run 'cargo test' for tests: {name}")

        if WINDOWS:
            append_rustflags("-Csymbol-mangling-version=v0")
        set_env("RUST_BACKTRACE", "full")

        rsvim_log = os.getenv("RSVIM_LOG")
        if isinstance(rsvim_log, str):
            set_env("RSVIM_LOG", rsvim_log)
        else:
            set_env("RSVIM_LOG", "trace")
        set_sccache()
        set_rustflags()
        command = f"cargo nextest run{jobs} --no-capture {name}"

        command = command.strip()
        logging.info(command)
        os.system(command)

    def miri(self, jobs) -> None:
        if WINDOWS:
            append_rustflags("-Csymbol-mangling-version=v0")

        set_env("RUST_BACKTRACE", "full")
        set_env(
            "MIRIFLAGS",
            "-Zmiri-backtrace=full -Zmiri-disable-isolation -Zmiri-permissive-provenance",
        )
        command = f"cargo +nightly miri nextest run{jobs} -F unicode_lines --no-default-features -p rsvim_core"

        command = command.strip()
        logging.info(command)
        os.system(command)

    def list(self) -> None:
        set_sccache()
        set_rustflags()

        command = "cargo nextest list"

        command = command.strip()
        logging.info(command)
        os.system(command)


# build/b
class Build(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "build"
        self._alias = "b"

        self.build_parser = subparsers.add_parser(
            self._name,
            aliases=[self._alias],
            help="cargo build",
        )
        self.build_parser.add_argument("-v", "--verbose", action="store_true")
        self.build_parser.add_argument("-F", "--features", action="append")
        self.build_parser.add_argument("-r", "--release", action="store_true")
        self.build_parser.add_argument("-n", "--nightly", action="store_true")

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return self._alias

    def run(self, args) -> None:
        cmd = [sccache(), rustflags([]), "cargo build"]
        if args.release:
            cmd.append("--release")
        elif args.nightly:
            cmd.extend(["--profile", "nightly"])
        if args.verbose:
            cmd.append("-vv")
        if isinstance(args.features, list) and len(args.features) > 0:
            cmd.extend(["-F", ",".join(args.features)])
        run(cmd)


# fmt/f
class FormatCommand(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "fmt"
        self._alias = "f"

        self.fmt_parser = subparsers.add_parser(
            self._name,
            aliases=[self._alias],
            help="Run multiple code formattors/checkers/generators, by default run them all",
        )
        self.fmt_parser.add_argument(
            "--rust",
            action="store_true",
            help="Only run `cargo +nightly fmt`, by default is false",
        )
        self.fmt_parser.add_argument(
            "--tsc",
            action="store_true",
            help="Only run `tsc`, by default is false",
        )

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return self._alias

    def run(self, args) -> None:
        if args.tsc:
            self.tsc()
        elif args.rust:
            self.rustfmt()
        else:
            for f in [
                self.typos,
                self.rustfmt,
                self.taplo,
                self.prettier,
                self.tsc,
            ]:
                f()

    def typos(self):
        command = "typos"
        logging.info(command)
        os.system(command)

    def rustfmt(self):
        command = "cargo +nightly fmt"
        logging.info(command)
        os.system(command)

    def taplo(self):
        command = "taplo fmt"
        logging.info(command)
        os.system(command)

    def prettier(self):
        command = "prettier --write *.md ./runtime/*.ts"
        logging.info(command)
        os.system(command)

    def tsc(self):
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


# doc
class DocumentCommand(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "doc"

        self.doc_parser = subparsers.add_parser(
            self._name,
            help="Start `cargo doc` on `http://localhost:3000/rsvim`",
        )
        self.doc_parser.add_argument(
            "-w",
            "--watch",
            action="store_true",
            help="Running cargo doc and watching file changes, by default is false",
        )

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return None

    def run(self, args) -> None:
        command = "cargo doc && browser-sync start --ss target/doc -s target/doc --directory --startPath rsvim_core --no-open"
        if args.watch:
            logging.info("Run 'cargo doc' and refresh it on file changes")
            command = f"cargo watch -s '{command}'"
        else:
            logging.info("Run 'cargo doc' only once")

        command = command.strip()
        logging.info(command)
        os.system(command)


# release/r
class ReleaseCommand(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "release"
        self._alias = "r"

        self.release_parser = subparsers.add_parser(
            self._name,
            aliases=[self._alias],
            help="Run `cargo release` to publish crates",
        )
        self.release_parser.add_argument(
            "level",
            choices=["alpha", "beta", "rc", "major", "minor", "patch"],
            help="Release [LEVEL]",
        )
        self.release_parser.add_argument(
            "-e",
            "--execute",
            action="store_true",
            help="Execute `cargo release` with `--no-verify`",
        )

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return self._alias

    def run(self, args) -> None:
        cwd = pathlib.Path.cwd()
        git_root = cwd / ".git"
        assert git_root.is_dir(), "The $CWD/$PWD must be git repo root!"

        command = f"GIT_CLIFF_CONFIG=$PWD/cliff.toml GIT_CLIFF_WORKDIR=$PWD GIT_CLIFF_REPOSITORY=$PWD GIT_CLIFF_OUTPUT=$PWD/CHANGELOG.md cargo release {args.level}"
        if args.execute:
            logging.info(f"Execute 'cargo release' with level: {args.level}")
            command = f"{command} --execute --no-verify"
        else:
            logging.info(f"Dry run 'cargo release' with level: {args.level}")

        command = command.strip()
        logging.info(command)
        os.system(command)


# npm
class NpmCommand(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "npm"

        self.npm_parser = subparsers.add_parser(
            self._name,
            help="Run `npm` with multiple sub commands.",
        )
        self.npm_parser.add_argument(
            "-v",
            "--version",
            choices=["major", "minor", "patch"],
            help="Run `npm version` with [LEVEL]",
        )

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return None

    def run(self, args) -> None:
        if args.version is not None:
            self.version(args.version)

    def version(self, level) -> None:
        command = f"npm version {level} --no-git-tag-version"
        logging.info(command)
        os.system(command)


if __name__ == "__main__":
    logging.basicConfig(format="%(levelname)s: %(message)s", level=logging.INFO)

    parser = argparse.ArgumentParser(description="development toolkit")
    parser.add_argument(
        "-n",
        "--no-cache",
        action="store_true",
        help="disable sccache",
    )

    subparsers = parser.add_subparsers(dest="subcommand")

    commands = [
        Build(subparsers),
        Clippy(subparsers),
        DocumentCommand(subparsers),
        FormatCommand(subparsers),
        NpmCommand(subparsers),
        ReleaseCommand(subparsers),
        Test(subparsers),
    ]

    parsed_args = parser.parse_args()
    logging.info(f"args:{parsed_args}")

    if parsed_args.no_cache:
        NO_CACHE = True

    for command in commands:
        sub = parsed_args.subcommand
        if sub is not None and (command.name() == sub or command.alias() == sub):
            command.run(parsed_args)
            exit(0)

    logging.error("missing arguments, use -h/--help for more details.")
