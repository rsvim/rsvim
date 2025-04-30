//! Common utils for rsvim executables.

#[cfg(all(
  not(target_os = "windows"),
  not(target_os = "openbsd"),
  not(target_os = "freebsd"),
  any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "powerpc64"
  ),
  feature = "jemalloc"
))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
