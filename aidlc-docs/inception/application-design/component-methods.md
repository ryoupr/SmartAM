# Component Methods — Iteration 4

## IdleWatcher (`idle_watcher.rs`)

```rust
/// IDLEウォッチャーを起動（全アカウント分）
pub fn start(app_handle: AppHandle, accounts: Vec<AccountConfig>, folders: Vec<Vec<String>>)

/// 特定アカウントのIDLE監視を停止
pub fn stop_account(account_index: usize)

/// 全アカウントのIDLE監視を停止
pub fn stop_all()

/// 現在の接続状態を取得
pub fn get_status() -> Vec<WatcherStatus>

/// 設定変更時にウォッチャーを再起動
pub fn reload(accounts: Vec<AccountConfig>, folders: Vec<Vec<String>>)
```

### 内部メソッド
```rust
/// 単一アカウント・フォルダのIDLEループ
async fn idle_loop(app: AppHandle, config: AccountConfig, folder: String, index: usize)

/// IDLE接続確立
async fn connect_idle(config: &AccountConfig, folder: &str) -> Result<ImapSession>

/// フォールバックポーリング
async fn poll_fallback(app: AppHandle, config: AccountConfig, folder: String, interval: Duration)

/// 新着検知時の処理（通知発火 + イベント送信）
fn on_new_mail(app: &AppHandle, account_index: usize, mails: Vec<MailSummary>)
```

## TrayManager (`tray.rs`)

```rust
/// Trayアイコンとメニューを初期化
pub fn setup(app: &App) -> Result<()>

/// メニューの新着件数を更新
pub fn update_badge(app: &AppHandle, count: usize)

/// 通知一時停止の切替
pub fn toggle_pause(app: &AppHandle) -> bool
```

## 新規Tauriコマンド (`lib.rs`)

```rust
#[tauri::command]
fn get_idle_status() -> Vec<WatcherStatus>

#[tauri::command]
fn restart_idle_watcher(accounts: Vec<AccountConfig>, folders: Vec<Vec<String>>)

#[tauri::command]
fn set_notification_pause(paused: bool)
```

## Tauriイベント（Rust → Frontend）

| イベント名 | ペイロード | 発火タイミング |
|---|---|---|
| `new-mail` | `{ accountIndex, mails: MailSummary[] }` | 新着メール検知時 |
| `idle-status-changed` | `{ accountIndex, status: "connected" \| "reconnecting" \| "polling" }` | 接続状態変化時 |
