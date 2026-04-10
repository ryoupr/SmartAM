// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  // Load .env from project root (walk up from exe dir)
  dotenvy::from_filename_override(
    std::env::current_exe()
      .ok()
      .and_then(|p| p.ancestors().take(10).find_map(|d| {
        let env = d.join(".env");
        env.exists().then_some(env)
      }))
      .unwrap_or_else(|| std::path::PathBuf::from(".env")),
  ).ok();
  app_lib::run();
}
