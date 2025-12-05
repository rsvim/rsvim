set unstable
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

skip-cache := "0"

alias c := clippy


[windows]
_sccache_env:
  if ( "{{skip-cache}}" -eq "0" ) { $env:RUSTC_WRAPPER="sccache.exe" }

[unix]
_sccache_env:
  if "{{skip-cache}}" != "0"; then export RUSTC_WRAPPER="{{which('sccache')}}"; fi

[windows]
_clippy_env:
  $env:RUSTFLAGS="-Dwarnings -Csymbol-mangling-version=v0"

[unix]
_clippy_env:
  export RUSTFLAGS="-Dwarnings -Csymbol-mangling-version=v0"

clippy: _clippy_env _sccache_env
  @just _clippy_env
  @just _sccache_env
  cargo clippy --workspace --all-targets
