use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::imap_client::{connect_idle_session, fetch_new_mails_with_session, resolve_folder_pub};
use crate::{AccountConfig, MailSummary};

// --- Domain Entities ---

#[derive(Clone, Serialize)]
pub struct WatcherStatus {
    pub account_index: usize,
    pub folder: String,
    pub state: String,
    pub error: Option<String>,
}

#[derive(Clone, Serialize)]
struct NewMailEvent {
    account_index: usize,
    mails: Vec<MailSummary>,
}

#[derive(Clone, Serialize)]
struct IdleStatusEvent {
    account_index: usize,
    folder: String,
    status: String,
}

// --- Config passed from frontend ---

#[derive(Clone, serde::Deserialize)]
pub struct WatcherAccountConfig {
    pub account: AccountConfig,
    pub account_index: usize,
    pub folders: Vec<String>,
    pub sync_interval_secs: u64,
    pub notification_sound: bool,
}

// --- Global State ---

pub struct IdleWatcher {
    handles: Vec<tauri::async_runtime::JoinHandle<()>>,
    statuses: Arc<Mutex<Vec<WatcherStatus>>>,
    paused: Arc<AtomicBool>,
}

static WATCHER: std::sync::LazyLock<tokio::sync::Mutex<Option<IdleWatcher>>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(None));

// --- Public API ---

pub async fn start(app: AppHandle, configs: Vec<WatcherAccountConfig>) {
    stop_all().await;
    let statuses: Arc<Mutex<Vec<WatcherStatus>>> = Arc::new(Mutex::new(Vec::new()));
    let paused = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();

    for cfg in configs {
        for folder in &cfg.folders {
            let status = WatcherStatus {
                account_index: cfg.account_index,
                folder: folder.clone(),
                state: "connecting".into(),
                error: None,
            };
            statuses.lock().await.push(status);

            let app_clone = app.clone();
            let cfg_clone = cfg.clone();
            let folder_clone = folder.clone();
            let statuses_clone = statuses.clone();
            let paused_clone = paused.clone();

            let handle = tauri::async_runtime::spawn(async move {
                idle_loop(app_clone, cfg_clone, folder_clone, statuses_clone, paused_clone).await;
            });
            handles.push(handle);
        }
    }

    let mut watcher = WATCHER.lock().await;
    *watcher = Some(IdleWatcher { handles, statuses, paused });
}

pub async fn stop_all() {
    let mut watcher = WATCHER.lock().await;
    if let Some(w) = watcher.take() {
        for h in w.handles {
            h.abort();
        }
    }
}

pub async fn reload(app: AppHandle, configs: Vec<WatcherAccountConfig>) {
    start(app, configs).await;
}

pub async fn get_status() -> Vec<WatcherStatus> {
    let watcher = WATCHER.lock().await;
    match &*watcher {
        Some(w) => w.statuses.lock().await.clone(),
        None => Vec::new(),
    }
}

pub async fn set_paused(paused: bool) {
    let watcher = WATCHER.lock().await;
    if let Some(w) = &*watcher {
        w.paused.store(paused, Ordering::Relaxed);
    }
}

pub async fn is_paused() -> bool {
    let watcher = WATCHER.lock().await;
    match &*watcher {
        Some(w) => w.paused.load(Ordering::Relaxed),
        None => false,
    }
}

// --- Internal: IDLE Loop ---

async fn idle_loop(
    app: AppHandle,
    cfg: WatcherAccountConfig,
    folder: String,
    statuses: Arc<Mutex<Vec<WatcherStatus>>>,
    paused: Arc<AtomicBool>,
) {
    let mut last_uid: u32 = 0;
    let imap_folder = resolve_folder_pub(&cfg.account.email, &folder);

    loop {
        update_status(&statuses, cfg.account_index, &folder, "connecting", None).await;
        emit_status(&app, cfg.account_index, &folder, "connecting");

        match connect_and_idle(&cfg.account, &imap_folder, &mut last_uid).await {
            Ok(new_mails) => {
                update_status(&statuses, cfg.account_index, &folder, "idle", None).await;
                emit_status(&app, cfg.account_index, &folder, "connected");

                if !new_mails.is_empty() {
                    on_new_mail(&app, &cfg, &new_mails, &paused);
                    if let Some(max_uid) = new_mails.iter().map(|m| m.uid).max() {
                        last_uid = last_uid.max(max_uid);
                    }
                }
                // IDLE returned normally (29min timeout or EXISTS) — loop continues
            }
            Err(e) => {
                log::debug!("idle_loop error [{}:{}]: {}", cfg.account.email, folder, e);
                // Attempt reconnection with backoff
                if !reconnect_with_backoff(&app, &cfg, &folder, &imap_folder, &statuses, &paused, &mut last_uid).await {
                    // 5 failures → fallback to polling
                    log::info!("IDLE→Polling fallback: {}:{}", cfg.account.email, folder);
                    update_status(&statuses, cfg.account_index, &folder, "polling", None).await;
                    emit_status(&app, cfg.account_index, &folder, "polling");
                    poll_fallback(&app, &cfg, &folder, &imap_folder, &statuses, &paused, &mut last_uid).await;
                }
            }
        }
    }
}

async fn connect_and_idle(
    config: &AccountConfig,
    imap_folder: &str,
    last_uid: &mut u32,
) -> Result<Vec<MailSummary>, String> {
    let mut session = connect_idle_session(config).await?;
    let mbox = session.select(imap_folder).await.map_err(|e| format!("SELECT失敗: {e}"))?;

    // Set baseline UID on first connect (don't notify for existing mails)
    if *last_uid == 0 {
        *last_uid = mbox.exists;
    }

    // Issue IDLE
    let mut idle_handle = session.idle();
    idle_handle.init().await.map_err(|e| format!("IDLE init失敗: {e}"))?;

    let (idle_future, _stop) = idle_handle.wait_with_timeout(Duration::from_secs(29 * 60));
    let reason = idle_future.await.map_err(|e| format!("IDLE wait失敗: {e}"))?;

    // Get session back
    let mut session = idle_handle.done().await.map_err(|e| format!("IDLE done失敗: {e}"))?;

    // Check for new mails if server sent data
    let new_mails = match reason {
        async_imap::extensions::idle::IdleResponse::NewData(_) => {
            fetch_new_mails_with_session(&mut session, imap_folder, *last_uid).await.unwrap_or_default()
        }
        _ => Vec::new(), // Timeout or ManualInterrupt — just re-issue IDLE
    };

    let _ = session.logout().await;
    Ok(new_mails)
}

// --- Internal: Reconnection ---

const BACKOFF_SECS: [u64; 5] = [5, 10, 30, 60, 300];

async fn reconnect_with_backoff(
    app: &AppHandle,
    cfg: &WatcherAccountConfig,
    folder: &str,
    imap_folder: &str,
    statuses: &Arc<Mutex<Vec<WatcherStatus>>>,
    paused: &Arc<AtomicBool>,
    last_uid: &mut u32,
) -> bool {
    for (attempt, &delay) in BACKOFF_SECS.iter().enumerate() {
        let msg = format!("attempt {}/{}", attempt + 1, BACKOFF_SECS.len());
        update_status(statuses, cfg.account_index, folder, "reconnecting", Some(msg)).await;
        emit_status(app, cfg.account_index, folder, "reconnecting");

        sleep(Duration::from_secs(delay)).await;

        match connect_and_idle(&cfg.account, imap_folder, last_uid).await {
            Ok(new_mails) => {
                if !new_mails.is_empty() {
                    on_new_mail(app, cfg, &new_mails, paused);
                    if let Some(max_uid) = new_mails.iter().map(|m| m.uid).max() {
                        *last_uid = (*last_uid).max(max_uid);
                    }
                }
                update_status(statuses, cfg.account_index, folder, "idle", None).await;
                emit_status(app, cfg.account_index, folder, "connected");
                return true;
            }
            Err(e) => {
                log::debug!("reconnect attempt {} failed: {}", attempt + 1, e);
            }
        }
    }
    false
}

// --- Internal: Polling Fallback ---

async fn poll_fallback(
    app: &AppHandle,
    cfg: &WatcherAccountConfig,
    folder: &str,
    imap_folder: &str,
    statuses: &Arc<Mutex<Vec<WatcherStatus>>>,
    paused: &Arc<AtomicBool>,
    last_uid: &mut u32,
) {
    let interval = Duration::from_secs(cfg.sync_interval_secs);
    let mut poll_count: u32 = 0;

    loop {
        sleep(interval).await;
        poll_count += 1;

        // Try fetching new mails via regular connection
        match crate::imap_client::fetch_new_mails(&cfg.account, folder, *last_uid).await {
            Ok(mails) => {
                if !mails.is_empty() {
                    on_new_mail(app, cfg, &mails, paused);
                    if let Some(max_uid) = mails.iter().map(|m| m.uid).max() {
                        *last_uid = (*last_uid).max(max_uid);
                    }
                }
            }
            Err(e) => {
                log::debug!("poll_fallback error: {}", e);
            }
        }

        // Every 10 polls, try to restore IDLE
        if poll_count % 10 == 0 {
            log::debug!("poll_fallback: attempting IDLE restore for {}:{}", cfg.account.email, folder);
            if connect_and_idle(&cfg.account, imap_folder, last_uid).await.is_ok() {
                log::info!("Polling→IDLE restored: {}:{}", cfg.account.email, folder);
                update_status(statuses, cfg.account_index, folder, "idle", None).await;
                emit_status(app, cfg.account_index, folder, "connected");
                return; // Exit polling, caller's loop will resume IDLE
            }
        }
    }
}

// --- Internal: Notification ---

fn on_new_mail(
    app: &AppHandle,
    cfg: &WatcherAccountConfig,
    mails: &[MailSummary],
    paused: &Arc<AtomicBool>,
) {
    log::debug!("new mail: account={} count={}", cfg.account_index, mails.len());

    // Always emit event to frontend (even when paused)
    let _ = app.emit("new-mail", NewMailEvent {
        account_index: cfg.account_index,
        mails: mails.to_vec(),
    });

    // Skip notification if paused
    if paused.load(Ordering::Relaxed) {
        return;
    }

    // Send macOS notification
    let latest = &mails[0];
    let body = if mails.len() == 1 {
        format!("{}: {}", latest.from, latest.subject)
    } else {
        format!("{}: {} 他{}件", latest.from, latest.subject, mails.len() - 1)
    };

    if let Err(e) = app.notification()
        .builder()
        .title("SmartAM")
        .body(&body)
        .show()
    {
        log::error!("notification send failed: {}", e);
    }
}

// --- Internal: Helpers ---

async fn update_status(
    statuses: &Arc<Mutex<Vec<WatcherStatus>>>,
    account_index: usize,
    folder: &str,
    state: &str,
    error: Option<String>,
) {
    let mut list = statuses.lock().await;
    if let Some(s) = list.iter_mut().find(|s| s.account_index == account_index && s.folder == folder) {
        s.state = state.to_string();
        s.error = error;
    }
}

fn emit_status(app: &AppHandle, account_index: usize, folder: &str, status: &str) {
    let _ = app.emit("idle-status-changed", IdleStatusEvent {
        account_index,
        folder: folder.to_string(),
        status: status.to_string(),
    });
}
