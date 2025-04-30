//! RSVIM global memory allocator.

#[cfg(all(
  target_family = "unix",
  not(target_os = "macos"),
  not(target_os = "emscripten"),
  feature = "jemalloc"
))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(all(target_family = "unix", target_os = "macos", feature = "jemalloc"))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
