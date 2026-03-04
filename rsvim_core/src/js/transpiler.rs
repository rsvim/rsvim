//! Js module transpiler.

// pub mod jsx;
// pub mod tsx;
#[cfg(feature = "typescript")]
pub mod typescript;
#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(all(test, feature = "typescript"))]
mod typescript_tests;

#[cfg(feature = "typescript")]
pub use typescript::TypeScript;
#[cfg(feature = "wasm")]
pub use wasm::Wasm;
