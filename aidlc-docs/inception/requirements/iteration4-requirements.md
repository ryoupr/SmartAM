# Iteration 4: 新規メール受信通知 — 要件定義

## Intent Analysis

- **User Request**: 新規メール受信時にmacOS通知を表示する機能の実装
- **Request Type**: Enhancement（既存機能の本格化）
- **Scope**: Multiple Components（Rust backend + Frontend設定UI + Tauri設定）
- **Complexity**: Moderate（IMAP IDLE + バックグラウンド常駐 + フォールバック）

## Functional Requirements

### FR-1: IMAP IDLEによるリアルタイム新着検知
- 通知ONのアカウントに対してIMAP IDLE接続を維持
- 対象フォルダ: デフォルトはINBOXのみ
- 設定画面で通知対象フォルダを追加選択可能
- RFC 2177準拠: 29分ごとにIDLE再発行
- 新着検知時にmacOS通知を発火

### FR-2: フォールバックポーリング
- IDLE接続失敗・切断時は自動再接続を試行
- 再接続失敗時はポーリングにフォールバック
- ポーリング間隔: 現在の設定値を流用（デフォルト5分、ユーザー変更可能）
- ネットワーク復帰時にIDLE接続を再確立

### FR-3: バックグラウンド常駐
- ウィンドウ閉じてもプロセス維持
- メニューバー常駐アイコン表示
- Dockアイコン非表示（完全バックグラウンドアプリ化）
- メニューバーアイコンからウィンドウ再表示可能

### FR-4: 通知表示
- 内容: 送信者 + 件名
- 複数件同時: 「{送信者}: {件名} 他N件」形式
- サウンド: アカウント設定の `notificationSound` に従う

### FR-5: アカウント別通知設定
- 既存の `notifications` フラグを流用（アカウント個別ON/OFF）
- デフォルト: 通知ON
- 通知対象フォルダの選択UI（デフォルト: INBOX）

## Non-Functional Requirements

### NFR-1: 信頼性
- IDLE接続断→自動再接続（exponential backoff）
- フォールバックポーリングで通知漏れを防止

### NFR-2: リソース効率
- IDLE接続はアカウント×対象フォルダ数の接続を維持
- Rust側バックグラウンドスレッドで実行（WebView非依存）

### NFR-3: セキュリティ
- OAuth トークンリフレッシュをRust側で自動実行
- Keychain経由の認証情報管理（既存実装を流用）

## Technical Decisions

- ポーリングロジックをフロントエンド（+page.svelte）からRustバックエンドに移動
- `async-imap` crateのIDLE機能を使用
- `tauri_plugin_notification` をRust側から直接呼び出し
- Tauri `RunEvent::ExitRequested` + `prevent_exit()` でプロセス維持
- Tauri tray plugin でメニューバーアイコン実装

### NFR-4: ロギング（可観測性）
- 既存の `log` crate + `tauri_plugin_log` 基盤を流用
- ログレベル（既存慣例に準拠）:
  - `debug`: IDLE接続/切断/再接続、IDLE再発行、新着検知、通知発火、ポーリング実行、トークンリフレッシュ
  - `info`: フォールバック切替（重要な状態変化のみ）
  - `error`: 接続失敗、通知送信失敗
- フロントエンドへのイベント通知は既存 `trace(TAG, msg)` パターンを踏襲

## Out of Scope

- IMAP IDLE非対応サーバーの検出・警告（将来対応）
- 通知クリック時のメール直接表示（将来対応）
- 本文プレビュー表示
