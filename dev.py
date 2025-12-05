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


def run(cmd):
    assert isinstance(cmd, str)
    logging.info(cmd)
    os.system(cmd)


def env(name, value):
    os.environ[name] = value
    logging.info(f"Set {name}={value}")


def sccache():
    if SCCACHE is None:
        logging.warning("'sccache' not found!")
        return
    if NO_CACHE:
        logging.warning("'sccache' is disabled!")
        return
    env("RUSTC_WRAPPER", SCCACHE)


def rustflags():
    flags = ["-Dwarnings"]
    if WINDOWS:
        flags.append("-Csymbol-mangling-version=v0")
    flags = " ".join(flags)
    env("RUSTFLAGS", flags)


def rust_backtrace():
    env("RUST_BACKTRACE", "full")


def miriflags():
    env(
        "MIRIFLAGS",
        '"-Zmiri-backtrace=full -Zmiri-disable-isolation -Zmiri-permissive-provenance"',
    )


def rsvim_log():
    var = os.getenv("RSVIM_LOG")
    if isinstance(var, str):
        env("RSVIM_LOG", var)
    else:
        env("RSVIM_LOG", "trace")


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
        sccache()
        rustflags()
        cmd = "cargo clippy --workspace --all-targets"
        run(cmd)


# test/t
class Test(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "test"
        self._alias = "t"

        self.test_parser = subparsers.add_parser(
            self._name,
            aliases=[self._alias],
            help="cargo test",
        )
        self.test_parser.add_argument(
            "-l", "--list", action="store_true", dest="list_test"
        )
        self.test_parser.add_argument("name", nargs="*", default=[])

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return self._alias

    def run(self, args) -> None:
        if args.list_test:
            self.list()
        else:
            self.test(args.name)

    def test(self, name) -> None:
        sccache()
        rustflags()
        rust_backtrace()
        rsvim_log()
        cmd = "cargo nextest run --no-capture"
        if len(name) == 0:
            cmd = f"{cmd} --all"
        else:
            name = " ".join(list(dict.fromkeys(name)))
            cmd = f"{cmd} {name}"
        run(cmd)

    def list(self) -> None:
        sccache()
        rustflags()
        cmd = "cargo nextest list"
        run(cmd)


# miri
class Miri(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "miri"

        self.test_parser = subparsers.add_parser(
            self._name,
            help="cargo +nightly miri test",
        )
        self.test_parser.add_argument("name", nargs="*", default=[])
        self.test_parser.add_argument("-j", "--job", nargs=1, metavar="N")

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return None

    def run(self, args) -> None:
        rustflags()
        rust_backtrace()
        miriflags()
        if args.job is None:
            job = ""
        else:
            job = f" -j {args.job[0]}"
        cmd = f"cargo +nightly miri nextest run{job} -F unicode_lines --no-default-features -p rsvim_core"
        run(cmd)


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
        sccache()
        rustflags()
        cmd = "cargo build"
        if args.release:
            cmd = f"{cmd} --release"
        elif args.nightly:
            cmd = f"{cmd} --profile nightly"
        if args.verbose:
            cmd = f"{cmd} -vv"
        if isinstance(args.features, list) and len(args.features) > 0:
            feat = ",".join(args.features)
            cmd = f"{cmd} -F {feat}"
        run(cmd)


# fmt/f
class Format(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "fmt"
        self._alias = "f"

        self.fmt_parser = subparsers.add_parser(
            self._name,
            aliases=[self._alias],
            help="code formattors",
        )
        self.fmt_parser.add_argument("-r", "--rust", action="store_true")
        self.fmt_parser.add_argument("-t", "--tsc", action="store_true")

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
            for cmd in [
                self.others,
                self.rustfmt,
                self.tsc,
            ]:
                cmd()

    def others(self):
        typos = "typos"
        taplo = "taplo fmt"
        prettier = "prettier --write *.md ./runtime/*.ts"
        for cmd in [typos, taplo, prettier]:
            run(cmd)

    def rustfmt(self):
        cmd = "cargo +nightly fmt"
        run(cmd)

    def tsc(self):
        cmd = "tsc"
        run(cmd)
        for file in ["00__web.d.ts", "01__rsvim.d.ts"]:
            src = f"types/{file}"
            dest = f".{file}"
            with open(src, "r") as src:
                with open(dest, "w") as dest:
                    dest.write("// @ts-nocheck\n")
                    for line in src.readlines():
                        dest.write(line)
            cmd = f"mv {dest} {src}"
            run(cmd)


# doc
class Document(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "doc"

        self.doc_parser = subparsers.add_parser(
            self._name,
            help="cargo doc",
        )

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return None

    def run(self, args) -> None:
        cmd = "cargo doc && browser-sync start --ss target/doc -s target/doc --directory --startPath rsvim_core --no-open"
        run(cmd)


# release
class Release(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "release"

        self.release_parser = subparsers.add_parser(
            self._name,
            help="cargo release",
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
        return None

    def run(self, args) -> None:
        cwd = pathlib.Path.cwd()
        git_root = cwd / ".git"
        assert git_root.is_dir(), "The $CWD/$PWD must be git repo root!"

        env("GIT_CLIFF_CONFIG", f"{cwd / 'cliff.toml'}")
        env("GIT_CLIFF_WORKDIR", f"{cwd}")
        env("GIT_CLIFF_REPOSITORY", f"{cwd}")
        env("GIT_CLIFF_OUTPUT", f"{cwd / 'CHANGELOG.md'}")
        cmd = f"cargo release {args.level}"
        if args.execute:
            cmd = f"{cmd} --execute --no-verify"
        run(cmd)


# npm
class Npm(Cmd):
    def __init__(self, subparsers) -> None:
        self._name = "npm"

        self.npm_parser = subparsers.add_parser(
            self._name,
            help="npm version",
        )
        self.npm_parser.add_argument(
            "-v", "--version", choices=["major", "minor", "patch"]
        )

    def name(self) -> str:
        return self._name

    def alias(self) -> Optional[str]:
        return None

    def run(self, args) -> None:
        if args.version is not None:
            self.version(args.version)

    def version(self, level) -> None:
        cmd = f"npm version {level} --no-git-tag-version"
        run(cmd)


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
        Document(subparsers),
        Format(subparsers),
        Miri(subparsers),
        Npm(subparsers),
        Release(subparsers),
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
