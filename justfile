set unstable
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

export MIRIFLAGS := "-Zmiri-backtrace=full -Zmiri-disable-isolation -Zmiri-permissive-provenance"
export RUSTC_WRAPPER := if os() == "windows" { "sccache.exe" } else { which("sccache") }
export RUST_BACKTRACE := "full"
export RUSTFLAGS := if os() == "windows" { "-Dwarnings -Csymbol-mangling-version=v0" } else { "-Dwarnings" }
export RSVIM_LOG := "trace"

alias c := clippy
alias t := test
alias l := list_tests

[windows]
_sccache:
  @echo "set RUSTC_WRAPPER=${env:RUSTC_WRAPPER}"

[unix]
_sccache:
  @echo "set RUSTC_WRAPPER='$RUSTC_WRAPPER'"

[windows]
_rustflags:
  @echo "set RUSTFLAGS=${env:RUSTFLAGS}"

[unix]
_rustflags:
  @echo "set RUSTFLAGS='$RUSTFLAGS'"

[windows]
_miriflags:
  @echo "set MIRIFLAGS=${env:MIRIFLAGS}"

[unix]
_miriflags:
  @echo "set MIRIFLAGS='$MIRIFLAGS'"

[windows]
_rust_backtrace:
  @echo "set RUST_BACKTRACE=${env:RUST_BACKTRACE}"

[unix]
_rust_backtrace:
  @echo "set RUST_BACKTRACE='$RUST_BACKTRACE'"

[windows]
_rsvim_log:
  @echo "set RSVIM_LOG=${env:RSVIM_LOG}"

[unix]
_rsvim_log:
  @echo "set RSVIM_LOG='$RSVIM_LOG'"

clippy: _sccache _rustflags
  cargo clippy --workspace --all-targets

list_tests: _sccache _rustflags
  cargo nextest list

test *name="--all": _sccache _rustflags _rust_backtrace _rsvim_log
  cargo nextest run --no-capture {{name}}

miri job="job=num-cpus": _sccache _rustflags _miriflags _rust_backtrace
  cargo +nightly miri nextest run {{replace_regex(job, 'job=|j=', '-j ')}} -F unicode_lines --no-default-features -p rsvim_core
