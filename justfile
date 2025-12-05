set unstable
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

export RUST_BACKTRACE := "full"
export RUSTFLAGS := if os() == "windows" { "-Dwarnings -Csymbol-mangling-version=v0" } else { "-Dwarnings" }
export RSVIM_LOG := "trace"

alias c := clippy
alias t := test

[windows]
_sccache:
  @$env:RUSTC_WRAPPER="sccache.exe"
  @echo "set RUSTC_WRAPPER='$env:RUSTC_WRAPPER'"

[unix]
_sccache:
  @export RUSTC_WRAPPER={{which("sccache")}}
  @echo "set RUSTC_WRAPPER='$RUSTC_WRAPPER'"

[windows]
_sccache_nocache:
  @echo "set RUSTC_WRAPPER='$env:RUSTC_WRAPPER'"

[unix]
_sccache_nocache:
  @export RUSTC_WRAPPER={{which("sccache")}}
  @echo "set RUSTC_WRAPPER='$RUSTC_WRAPPER'"

[windows]
_rustflags:
  @echo "set RUSTFLAGS='$env:RUSTFLAGS'"

[unix]
_rustflags:
  @echo "set RUSTFLAGS='$RUSTFLAGS'"

[windows]
_rust_backtrace:
  @echo "set RUST_BACKTRACE='$env:RUST_BACKTRACE'"

[unix]
_rust_backtrace:
  @echo "set RUST_BACKTRACE='$RUST_BACKTRACE'"

[windows]
_rsvim_log:
  @echo "set RSVIM_LOG='$env:RSVIM_LOG'"

[unix]
_rsvim_log:
  @echo "set RSVIM_LOG='$RSVIM_LOG'"

_clippy: _sccache _rustflags
  cargo clippy --workspace --all-targets

_clippy_nocache: _sccache_nocache _rustflags
  cargo clippy --workspace --all-targets

clippy nocache="":
  if ("{{nocache}}" -eq "nc" -or "{{nocache}}" -eq "nocache") { just _clippy_nocache } else { just _clippy }


_list_tests: _sccache _rustflags
  cargo nextest list

_test +name: _sccache _rustflags _rust_backtrace _rsvim_log
  cargo nextest run --no-capture {{name}}

_test_nocache +name: _sccache_nocache _rustflags _rust_backtrace _rsvim_log
  cargo nextest run --no-capture {{name}}

test nocache="" *name="--all":
  if ("{{nocache}}" -eq "nc" -or "{{nocache}}" -eq "nocache") { just _test_nocache {{name}} } else { just _test {{name}} }
