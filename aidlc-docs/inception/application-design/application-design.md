# Application Design — Iteration 4: 新規メール受信通知

## 設計概要

既存のフロントエンド主導ポーリングを、Rustバックグラウンドサービス（IMAP IDLE）に置換する。

## アーキテクチャ変更

```
Before:  [+page.svelte] --poll--> [imap_client.rs] --IMAP--> [Server]
After:   [idle_watcher.rs] --IDLE--> [Server]
              ↓ emit("new-mail")
         [+page.svelte] (UI更新のみ)
              ↓
         [tauri_plugin_notification] → macOS通知
```

## 新規コンポーネント

| コンポーネント | ファイル | 責務 |
|---|---|---|
| IdleWatcher | `src-tauri/src/idle_watcher.rs` | IMAP IDLE維持、新着検知、フォールバックポーリング |
| TrayManager | `src-tauri/src/tray.rs` | メニューバーアイコン、メニュー管理 |

## 通信設計

- **Rust → Frontend**: Tauriイベント（`new-mail`, `idle-status-changed`）
- **Frontend → Rust**: Tauriコマンド（`get_idle_status`, `restart_idle_watcher`, `set_notification_pause`）
- **Rust → macOS**: `tauri_plugin_notification` 直接呼び出し

## ウィンドウライフサイクル

- ウィンドウ閉じ → 非表示（プロセス維持）
- Dockアイコン非表示
- メニューバーアイコンから再表示/終了

## 設定モデル拡張

```typescript
// store.ts AccountConfig に追加
notificationFolders: string[]  // デフォルト: ["INBOX"]
```

## 詳細ドキュメント

- [components.md](./components.md) — コンポーネント定義
- [component-methods.md](./component-methods.md) — メソッドシグネチャ
- [services.md](./services.md) — サービスオーケストレーション
- [component-dependency.md](./component-dependency.md) — 依存関係
