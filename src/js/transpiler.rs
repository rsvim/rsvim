//! Js module transpiler.

use anyhow::bail;
use anyhow::Result;
use regex::Regex;
use std::sync::OnceLock;
use swc_common::comments::SingleThreadedComments;
use swc_common::errors::ColorConfig;
use swc_common::errors::Handler;
use swc_common::sync::Lrc;
use swc_common::FileName;
use swc_common::Globals;
use swc_common::Mark;
use swc_common::SourceMap;
use swc_common::GLOBALS;
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::Parser;
use swc_ecma_parser::StringInput;
use swc_ecma_parser::Syntax;
use swc_ecma_parser::TsSyntax;
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_transforms_base::hygiene::hygiene;
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_react::react;
use swc_ecma_transforms_react::Options;
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;

static PRAGMA_REGEX: OnceLock<Regex> = OnceLock::new();

fn init_pragma_regex() -> Regex {
  Regex::new(r"@jsx\s+([^\s]+)").unwrap()
}

pub struct TypeScript;

impl TypeScript {
  /// Compiles TypeScript code into JavaScript.
  pub fn compile(filename: Option<&str>, source: &str) -> Result<String> {
    let globals = Globals::default();
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let filename = match filename {
      Some(filename) => FileName::Custom(filename.into()),
      None => FileName::Anon,
    };

    let fm = cm.new_source_file(filename.into(), source.into());

    // Initialize the TypeScript lexer.
    let lexer = Lexer::new(
      Syntax::Typescript(TsSyntax {
        tsx: true,
        decorators: true,
        no_early_errors: true,
        ..Default::default()
      }),
      Default::default(),
      StringInput::from(&*fm),
      None,
    );

    let mut parser = Parser::new_from(lexer);

    let program = match parser
      .parse_program()
      .map_err(|e| e.into_diagnostic(&handler).emit())
    {
      Ok(module) => module,
      Err(_) => bail!("TypeScript compilation failed."),
    };

    // This is where we're gonna store the JavaScript output.
    let mut buffer = vec![];

    GLOBALS.set(&globals, || {
      // Apply the rest SWC transforms to generated code.
      let program = program
        .fold_with(&mut resolver(Mark::new(), Mark::new(), true))
        .fold_with(&mut strip(Mark::new(), Mark::new()))
        .fold_with(&mut hygiene())
        .fold_with(&mut fixer(None));

      {
        let mut emitter = Emitter {
          cfg: swc_ecma_codegen::Config::default(),
          cm: cm.clone(),
          comments: None,
          wr: JsWriter::new(cm, "\n", &mut buffer, None),
        };

        emitter.emit_program(&program).unwrap();
      }
    });

    Ok(String::from_utf8_lossy(&buffer).to_string())
  }
}

pub struct Jsx;

impl Jsx {
  /// Compiles JSX code into JavaScript.
  pub fn compile(filename: Option<&str>, source: &str) -> Result<String> {
    let globals = Globals::default();
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let filename = match filename {
      Some(filename) => FileName::Custom(filename.into()),
      None => FileName::Anon,
    };

    let fm = cm.new_source_file(filename.into(), source.into());

    // NOTE: We're using a TypeScript lexer to parse JSX because it's a super-set
    // of JavaScript and we also want to support .tsx files.

    let lexer = Lexer::new(
      Syntax::Typescript(TsSyntax {
        tsx: true,
        decorators: true,
        no_early_errors: true,
        ..Default::default()
      }),
      Default::default(),
      StringInput::from(&*fm),
      None,
    );

    let mut parser = Parser::new_from(lexer);

    let module = match parser
      .parse_module()
      .map_err(|e| e.into_diagnostic(&handler).emit())
    {
      Ok(module) => module,
      Err(_) => bail!("JSX compilation failed."),
    };

    // This is where we're gonna store the JavaScript output.
    let mut buffer = vec![];

    // Look for the JSX pragma in the source code.
    // https://www.gatsbyjs.com/blog/2019-08-02-what-is-jsx-pragma/

    let pragma = PRAGMA_REGEX
      .get_or_init(init_pragma_regex)
      .find_iter(source)
      .next()
      .map(|m| m.as_str().to_string().replace("@jsx ", ""));

    GLOBALS.set(&globals, || {
      // Apply SWC transforms to given code.
      let module = module.fold_with(&mut react::<SingleThreadedComments>(
        cm.clone(),
        None,
        Options {
          pragma,
          ..Default::default()
        },
        Mark::new(),
        Mark::new(),
      ));

      {
        let mut emitter = Emitter {
          cfg: swc_ecma_codegen::Config::default(),
          cm: cm.clone(),
          comments: None,
          wr: JsWriter::new(cm, "\n", &mut buffer, None),
        };

        emitter.emit_module(&module).unwrap();
      }
    });

    Ok(String::from_utf8_lossy(&buffer).to_string())
  }
}

pub struct Wasm;

impl Wasm {
  // Converts a wasm binary into an ES module template.
  pub fn parse(source: &str) -> String {
    format!(
      "
        const wasmCode = new Uint8Array({:?});
        const wasmModule = new WebAssembly.Module(wasmCode);
        const wasmInstance = new WebAssembly.Instance(wasmModule);
        export default wasmInstance.exports;
        ",
      source.as_bytes()
    )
  }
}
