use std::sync::LazyLock;

pub fn is_github_actions() -> bool {
  static GITHUB_ACTIONS: LazyLock<bool> =
    LazyLock::new(|| std::env::var("GITHUB_ACTIONS").is_ok());

  *GITHUB_ACTIONS
}
