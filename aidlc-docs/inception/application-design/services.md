# Services — Iteration 4

## サービスオーケストレーション

### MailNotificationService（IdleWatcher内で実現）

**責務**: アカウント設定に基づき、適切な監視方式を選択・実行

**オーケストレーションフロー**:
1. アプリ起動時: `setup()` 内で `IdleWatcher::start()` を呼び出し
2. 各アカウントについて:
   - `notifications: true` → IDLE接続を試行
   - IDLE成功 → IDLEループ維持
   - IDLE失敗 → ポーリングフォールバック
3. 新着検知時:
   - `tauri_plugin_notification` で macOS通知送信
   - `app.emit("new-mail", payload)` でFEに通知
   - TrayManager経由でバッジ更新
4. 設定変更時: FEから `restart_idle_watcher` コマンドで再起動

### WindowLifecycleService（lib.rs内で実現）

**責務**: ウィンドウ閉じてもプロセス維持 + Tray経由の再表示

**フロー**:
1. `RunEvent::ExitRequested` → `api.prevent_exit()`
2. ウィンドウ閉じ → 非表示（destroy しない）
3. Tray「ウィンドウを表示」→ ウィンドウ再表示
4. Tray「終了」→ `app.exit(0)`
