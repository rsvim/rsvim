//! Js module transpiler.

// pub mod jsx;
// pub mod tsx;
pub mod typescript;
pub mod wasm;

#[cfg(test)]
mod typescript_tests;

pub use typescript::TypeScript;
