use std::sync::OnceLock;

pub fn next_global_usize() -> usize {
  static GLOBAL: OnceLock<usize> = OnceLock::new();
  let result: &usize = GLOBAL.get_or_init(|| 0usize);
  GLOBAL.set(result + 1);
  *result
}
