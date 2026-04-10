use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn log_dir() -> PathBuf {
    PathBuf::from("/var/log/smartam")
}

fn log_file() -> PathBuf {
    log_dir().join("trace.log")
}

pub fn init() {
    let dir = log_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        // Fallback to home dir if /var/log not writable
        let fallback = dirs::home_dir().unwrap_or_default().join(".smartam/logs");
        let _ = fs::create_dir_all(&fallback);
        eprintln!("Cannot create /var/log/smartam ({}), using {:?}", e, fallback);
    }
}

pub fn trace(tag: &str, msg: &str) {
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!("[{ts}] [{tag}] {msg}\n");

    // Try /var/log/smartam first, fallback to ~/.smartam/logs
    let paths = [
        log_file(),
        dirs::home_dir().unwrap_or_default().join(".smartam/logs/trace.log"),
    ];

    for path in &paths {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = f.write_all(line.as_bytes());
            let _ = f.flush();
            return;
        }
    }
    eprintln!("{}", line.trim());
}
