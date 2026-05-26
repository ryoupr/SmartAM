# Domain Entities — Unit 1: IdleWatcher

## WatcherState (enum)
```rust
pub enum WatcherState {
    Connecting,
    Idle,
    Reconnecting { attempt: u8 },
    Polling,
    Stopped,
}
```

## WatcherStatus (公開API用)
```rust
#[derive(Serialize, Clone)]
pub struct WatcherStatus {
    pub account_index: usize,
    pub folder: String,
    pub state: String,        // "idle" | "reconnecting" | "polling" | "stopped"
    pub last_check: Option<String>,  // ISO 8601
    pub error: Option<String>,
}
```

## WatcherConfig (内部設定)
```rust
struct WatcherConfig {
    pub account: AccountConfig,
    pub account_index: usize,
    pub folders: Vec<String>,
    pub sync_interval: Duration,
    pub notification_sound: bool,
    pub paused: bool,
}
```

## IdleWatcher (グローバル状態)
```rust
pub struct IdleWatcher {
    handles: Vec<JoinHandle<()>>,   // spawn済みタスク
    statuses: Arc<Mutex<Vec<WatcherStatus>>>,
    paused: Arc<AtomicBool>,
}
```

## Tauriイベントペイロード
```rust
#[derive(Serialize, Clone)]
struct NewMailEvent {
    account_index: usize,
    mails: Vec<MailSummary>,
}

#[derive(Serialize, Clone)]
struct IdleStatusEvent {
    account_index: usize,
    folder: String,
    status: String,  // "connected" | "reconnecting" | "polling" | "stopped"
}
```
