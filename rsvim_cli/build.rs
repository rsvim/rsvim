use git2::Repository;
use rsvim_core::js::JsRuntimeForSnapshot;
use rsvim_core::js::v8_version;
use std::path::Path;

pub const LOG: &str = "[RSVIM]";
// pub const LOG: &str = "cargo:warning=[RSVIM]";

fn version() {
  let profile = std::env::var("PROFILE").unwrap_or("debug".to_string());
  let opt_level = std::env::var("OPT_LEVEL").unwrap_or("0".to_string());
  let debug = std::env::var("DEBUG").unwrap_or("0".to_string());
  println!(
    "{LOG} Raw profile:{:?}, opt_level:{:?}, debug:{:?}",
    profile, opt_level, debug
  );

  let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
  let mut version = env!("CARGO_PKG_VERSION").to_string();

  // profile and git commit
  let is_release_profile = profile == "release"
    && (opt_level == "s" || opt_level == "z")
    && debug != "true";
  if is_release_profile {
    println!("{LOG} Resolved profile:release");
  } else {
    let profile = if profile == "release" {
      "nightly".to_string()
    } else {
      profile
    };
    println!("{LOG} Resolved profile:{:?}", profile);
    let maybe_git_commit = match Repository::open(&workspace_dir) {
      Ok(repo) => {
        let head = repo.head().unwrap();
        let oid = head.target().unwrap();
        let commit = repo.find_commit(oid).unwrap();
        let id = commit.id();
        let id = id.to_string();
        println!("{LOG} Git id:{:?}", id);
        format!("+{}", &id[0..8])
      }
      Err(e) => {
        println!("{LOG} Git error:{:?}", e);
        "".to_string()
      }
    };
    println!(
      "{LOG} Resolved version:{:?}, profile:{:?}, git_commit:{:?}",
      version, profile, maybe_git_commit
    );
    version = format!("{}+{}{}", version, profile, maybe_git_commit)
  }

  // swc version
  let swc = match std::fs::read_to_string(workspace_dir.join("Cargo.toml")) {
    Ok(manifest) => match manifest.parse::<toml::Table>() {
      Ok(parsed_manifest) => {
        let deps = &parsed_manifest["workspace"]["dependencies"];
        let parser = deps["swc_ecma_parser"].as_str();
        println!("{LOG} Swc version, swc_ecma_parser:{:?}", parser);
        format!(", swc_ecma_parser {}", parser.unwrap())
      }
      Err(e) => {
        println!("{LOG} Parse Cargo.toml error:{:?}", e);
        "".to_string()
      }
    },
    Err(e) => {
      println!("{LOG} Read Cargo.toml error:{:?}", e);
      "".to_string()
    }
  };

  // v8 version
  let v8 = format!("v8 {}", v8_version());

  version = format!("{} ({}{})", version, v8, swc);

  let output_path =
    Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_VERSION.TXT");
  println!("{LOG} Writing version into {:?}...", output_path.as_path());

  std::fs::write(output_path.as_path(), version.as_bytes()).unwrap();
}

fn snapshot() {
  let js_runtime = JsRuntimeForSnapshot::new();
  println!("{LOG} Build snapshot for rsvim cli...");
  let snapshot = js_runtime.create_snapshot();
  let snapshot = Box::from(&snapshot);
  let mut vec = Vec::with_capacity(snapshot.len());
  vec.extend_from_slice(&snapshot);

  let output_path =
    Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_SNAPSHOT.BIN");
  println!("{LOG} Writing snapshot into {:?}...", output_path.as_path());
  std::fs::write(output_path.as_path(), vec.into_boxed_slice()).unwrap();
}

fn main() {
  version();
  snapshot();
}
