# Code Generation Plan — Unit 1: IdleWatcher

## Part 1: Planning

### 実装ステップ

- [ ] 1. `app/src-tauri/src/idle_watcher.rs` 新規作成
  - [ ] 1.1 WatcherState enum, WatcherStatus struct 定義
  - [ ] 1.2 IdleWatcher struct（グローバル状態管理）
  - [ ] 1.3 `start()` — 全アカウント分のIDLEタスクをspawn
  - [ ] 1.4 `idle_loop()` — 単一フォルダのIDLEループ（29分再発行）
  - [ ] 1.5 `reconnect_with_backoff()` — 5s→10s→30s→60s→300s、5回失敗でフォールバック
  - [ ] 1.6 `poll_fallback()` — ポーリングループ + 10回ごとIDLE復帰試行
  - [ ] 1.7 `on_new_mail()` — 通知送信 + イベント発火 + バッジ更新
  - [ ] 1.8 `stop_all()` / `reload()` — 設定変更時の再起動
  - [ ] 1.9 `get_status()` — 現在の接続状態一覧取得
  - [ ] 1.10 `set_paused()` — 通知一時停止

- [ ] 2. `app/src-tauri/src/lib.rs` 変更
  - [ ] 2.1 `mod idle_watcher;` 追加
  - [ ] 2.2 setup内でIdleWatcher起動
  - [ ] 2.3 新規Tauriコマンド追加: `get_idle_status`, `restart_idle_watcher`, `set_notification_pause`
  - [ ] 2.4 invoke_handlerに新コマンド登録

- [ ] 3. `app/src-tauri/Cargo.toml` 変更
  - [ ] 3.1 `tauri-plugin-notification` のRust API利用に必要な追加features確認

- [ ] 4. ビルド検証
  - [ ] 4.1 `cargo check` パス確認

## 技術的考慮事項

### async-imap IDLE API
`async-imap` 0.11 の `Session::idle()` は `IdleHandle` を返し、`.wait_with_timeout()` で応答待機可能。

### 接続分離
IDLE用接続は既存の接続プール（POOL）とは別に管理。IDLEセッションはSELECT後にIDLEコマンドを発行するため、他の操作と共有不可。

### 通知送信（Rust側）
`tauri_plugin_notification::NotificationExt` trait を使い `app.notification().builder().title().body().show()` で送信。

### OAuthトークンリフレッシュ
既存の `oauth.rs` の `google_oauth_refresh` ロジックを内部関数として再利用。
