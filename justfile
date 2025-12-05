set unstable
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

export RUSTFLAGS := if os() == "windows" { "-Dwarnings -Csymbol-mangling-version=v0" } else { "-Dwarnings" }

alias c := clippy

[windows]
_sccache:
  @$env:RUSTC_WRAPPER="sccache.exe"
  @echo "set RUSTC_WRAPPER='$env:RUSTC_WRAPPER'"

[unix]
_sccache:
  @export RUSTC_WRAPPER={{which("sccache")}}
  @echo "set RUSTC_WRAPPER='$RUSTC_WRAPPER'"

[windows]
_rustflags:
  @echo "set RUSTFLAGS='$env:RUSTFLAGS'"

[unix]
_rustflags:
  @echo "set RUSTFLAGS='$RUSTFLAGS'"

_clippy: _sccache _rustflags
  cargo clippy --workspace --all-targets

_clippy_nocache: _rustflags
  cargo clippy --workspace --all-targets

clippy nocache="":
  if ("{{nocache}}" -eq "nc" -or "{{nocache}}" -eq "nocache") { just _clippy_nocache } else { just _clippy }
