//! WASM transpiler.

// pub struct Wasm;
//
// impl Wasm {
//   // Converts a wasm binary into an ES module template.
//   pub fn parse(source: &str) -> String {
//     format!(
//       "
//         const wasmCode = new Uint8Array({:?});
//         const wasmModule = new WebAssembly.Module(wasmCode);
//         const wasmInstance = new WebAssembly.Instance(wasmModule);
//         export default wasmInstance.exports;
//         ",
//       source.as_bytes()
//     )
//   }
// }
