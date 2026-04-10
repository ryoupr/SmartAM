fn main() {
  // Embed OAuth credentials at compile time from .env (if present)
  if let Ok(env_path) = std::fs::canonicalize("../../.env") {
    println!("cargo:rerun-if-changed={}", env_path.display());
    if let Ok(contents) = std::fs::read_to_string(&env_path) {
      for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if let Some((key, val)) = line.split_once('=') {
          println!("cargo:rustc-env={}={}", key.trim(), val.trim());
        }
      }
    }
  }
  tauri_build::build()
}
