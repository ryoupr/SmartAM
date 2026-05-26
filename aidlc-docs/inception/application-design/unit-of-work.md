# Units of Work — Iteration 4

## 分割方針
モノリスデスクトップアプリ内のモジュール分割。各ユニットは独立してコード生成・テスト可能だが、同一バイナリにコンパイルされる。

## Unit 1: IdleWatcher（Rustバックグラウンドサービス）
**スコープ**: IMAP IDLE接続維持 + フォールバックポーリング + 通知発火
**ファイル**: `app/src-tauri/src/idle_watcher.rs`
**責務**:
- アカウントごとのIDLE接続管理（起動/停止/再起動）
- RFC 2177準拠のIDLEループ（29分再発行）
- 接続断検知→exponential backoff再接続
- 再接続失敗→ポーリングフォールバック
- 新着検知→`tauri_plugin_notification`で通知送信
- 新着検知→`app.emit("new-mail")` でFEに通知
- OAuthトークンリフレッシュ（Rust側で自動実行）
**依存**: imap_client.rs（接続確立）, tauri_plugin_notification, AppHandle

## Unit 2: TrayManager + ウィンドウライフサイクル
**スコープ**: メニューバー常駐 + Dockアイコン非表示 + ウィンドウ制御
**ファイル**: `app/src-tauri/src/tray.rs`, `app/src-tauri/src/lib.rs`（setup変更）
**責務**:
- Trayアイコン + メニュー初期化
- メニュー項目: ウィンドウ表示 / 新着N件 / 通知一時停止 / 終了
- `RunEvent::ExitRequested` → `prevent_exit()`
- ウィンドウ閉じ→非表示（destroyしない）
- Dockアイコン非表示設定
- バッジ件数の動的更新
**依存**: AppHandle, IdleWatcher（件数取得）

## Unit 3: Frontend統合 + 設定UI
**スコープ**: FEポーリング削除、イベントリスナー追加、通知フォルダ設定UI
**ファイル**: `app/src/routes/+page.svelte`, `app/src/lib/store.ts`, `app/src/lib/components/settings/AccountTab.svelte`
**責務**:
- 既存FEポーリング（`startPolling`/`fetchNewMails`）をRust側に委譲
- `new-mail` イベントリスナーでUI更新（メール一覧に新着追加）
- `idle-status-changed` イベントで接続状態表示
- アカウント設定に `notificationFolders` 追加
- 通知対象フォルダ選択UI（チェックボックス）
- 設定変更時に `restart_idle_watcher` コマンド呼び出し
**依存**: Tauriイベント, Tauriコマンド

## 実行順序
1. **Unit 1** (IdleWatcher) — 他ユニットの基盤
2. **Unit 2** (TrayManager) — Unit 1のバッジ更新に依存
3. **Unit 3** (Frontend統合) — Unit 1, 2が動作する前提でFE側を接続
