//! Javascript react (jsx) transpiler.

// static PRAGMA_REGEX: OnceLock<Regex> = OnceLock::new();
//
// fn init_pragma_regex() -> Regex {
//   Regex::new(r"@jsx\s+([^\s]+)").unwrap()
// }
//
// pub struct Jsx;
//
// impl Jsx {
//   /// Compiles JSX code into JavaScript.
//   pub fn compile(filename: Option<&str>, source: &str) -> AnyResult<String> {
//     let globals = Globals::default();
//     let cm: Lrc<SourceMap> = Default::default();
//     let handler = Handler::with_tty_emitter(ColorConfig::Never, true, false, Some(cm.clone()));
//
//     let filename = match filename {
//       Some(filename) => FileName::Custom(filename.into()),
//       None => FileName::Anon,
//     };
//
//     let fm = cm.new_source_file(filename.into(), source.to_string());
//
//     // NOTE: We're using a TypeScript lexer to parse JSX because it's a super-set
//     // of JavaScript and we also want to support .tsx files.
//
//     let lexer = Lexer::new(
//       Syntax::Typescript(TsSyntax {
//         tsx: true,
//         decorators: true,
//         no_early_errors: true,
//         ..Default::default()
//       }),
//       Default::default(),
//       StringInput::from(&*fm),
//       None,
//     );
//
//     let mut parser = Parser::new_from(lexer);
//
//     let module = match parser
//       .parse_module()
//       .map_err(|e| e.into_diagnostic(&handler).emit())
//     {
//       Ok(module) => Program::Module(module),
//       Err(_) => bail!("JSX compilation failed."),
//     };
//
//     // This is where we're gonna store the JavaScript output.
//     let mut buffer = vec![];
//
//     // Look for the JSX pragma in the source code.
//     // https://www.gatsbyjs.com/blog/2019-08-02-what-is-jsx-pragma/
//
//     let pragma = PRAGMA_REGEX
//       .get_or_init(init_pragma_regex)
//       .find_iter(source)
//       .next()
//       .map(|m| m.as_str().to_string().replace("@jsx ", ""));
//
//     GLOBALS.set(&globals, || {
//       // Apply SWC transforms to given code.
//       let module = module.apply(&mut react::<SingleThreadedComments>(
//         cm.clone(),
//         None,
//         Options {
//           pragma,
//           ..Default::default()
//         },
//         Mark::new(),
//         Mark::new(),
//       ));
//
//       {
//         let mut emitter = Emitter {
//           cfg: swc_ecma_codegen::Config::default(),
//           cm: cm.clone(),
//           comments: None,
//           wr: JsWriter::new(cm, "\n", &mut buffer, None),
//         };
//
//         emitter.emit_module(&module).unwrap();
//       }
//     });
//
//     Ok(String::from_utf8_lossy(&buffer).to_string())
//   }
// }
