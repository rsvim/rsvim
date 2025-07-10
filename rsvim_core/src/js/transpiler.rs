//! Js module transpiler.

// Re-export
pub use typescript::TypeScript;

// pub mod jsx;
// pub mod tsx;
pub mod typescript;
// pub mod wasm;

#[cfg(test)]
mod typescript_tests;
