use std::sync::OnceLock;

pub fn next() -> usize {
  static GLOBAL: OnceLock<usize> = OnceLock::new();
  let result: &usize = GLOBAL.get_or_init(|| 0usize);
  GLOBAL.set(result + 1);
  *result
}
