# Application Design Plan — Iteration 4

## Plan Steps
- [ ] コンポーネント定義（components.md）
- [ ] コンポーネントメソッド定義（component-methods.md）
- [ ] サービス定義（services.md）
- [ ] コンポーネント依存関係（component-dependency.md）
- [ ] 統合ドキュメント（application-design.md）

## Design Questions (Answered)

### Question 1
新規バックグラウンドサービス（IDLE監視）の配置場所は？
[Answer]: B — 新規 `idle_watcher.rs` モジュールとして分離（責務分離）

### Question 2
フロントエンドからバックグラウンドサービスの状態を確認する手段は？
[Answer]: C — 両方（状態変化はイベント、詳細取得はコマンド）

### Question 3
メニューバー（Tray）アイコンのメニュー項目は？
[Answer]: B — 標準: 「ウィンドウを表示」「新着N件」「通知一時停止」「終了」
