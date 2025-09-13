//! Js module transpiler.

// pub mod jsx;
// pub mod tsx;
pub mod typescript;
// pub mod wasm;

pub use typescript::TypeScript;

#[cfg(test)]
mod typescript_tests;
