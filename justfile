set unstable
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

export RUSTFLAGS := if os() == "windows" { "-Dwarnings -Csymbol-mangling-version=v0" } else { "-Dwarnings" }
export RUSTC_WRAPPER := "sccache"

alias c := clippy


[windows]
clippy:
  @echo "RUSTFLAGS='$env:RUSTFLAGS'"
  @echo "RUSTC_WRAPPER='$env:RUSTC_WRAPPER'"
  cargo clippy --workspace --all-targets

[unix]
clippy:
  @echo "RUSTFLAGS='$RUSTFLAGS'"
  @echo "RUSTC_WRAPPER='$RUSTC_WRAPPER'"
  cargo clippy --workspace --all-targets
