# Components — Iteration 4

## 新規コンポーネント

### 1. IdleWatcher (`idle_watcher.rs`)
**責務**: IMAP IDLE接続の維持・新着検知・フォールバックポーリング
- アカウントごとにIDLE接続を管理
- 新着検知時にTauriイベントを発火
- 接続断時の自動再接続 + ポーリングフォールバック
- RFC 2177準拠（29分ごとIDLE再発行）

### 2. TrayManager (`tray.rs`)
**責務**: メニューバーアイコンとメニューの管理
- Trayアイコン表示
- メニュー項目の動的更新（新着件数）
- 通知一時停止状態の管理
- ウィンドウ表示/非表示の制御

### 3. NotificationFolderSettings (Frontend: 設定UI拡張)
**責務**: 通知対象フォルダの選択UI
- AccountTab.svelte内の通知設定セクションに追加
- フォルダ一覧取得 + チェックボックス選択

## 既存コンポーネント変更

### 4. lib.rs (変更)
- `RunEvent::ExitRequested` で `prevent_exit()` 追加
- IdleWatcher起動をsetup内に追加
- TrayManager初期化を追加
- 新規Tauriコマンド追加

### 5. store.ts / AccountTab.svelte (変更)
- アカウント設定に `notificationFolders: string[]` 追加
- 通知対象フォルダ選択UIの追加

### 6. +page.svelte (変更)
- フロントエンド側ポーリング削除（Rust側に移行）
- Tauriイベントリスナー追加（新着通知受信）
