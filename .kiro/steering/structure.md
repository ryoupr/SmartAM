# Project Structure

```
SmartAM/
├── app/                            # Tauri アプリケーション
│   ├── src/                        # Frontend (SvelteKit)
│   │   ├── routes/
│   │   │   ├── +page.svelte        # SPA エントリ（全状態管理・ショートカット）
│   │   │   ├── +layout.svelte      # レイアウト
│   │   │   └── +layout.ts          # SSR 無効化 (ssr = false)
│   │   ├── lib/
│   │   │   ├── components/         # UI コンポーネント
│   │   │   │   ├── settings/       # 設定タブ群
│   │   │   │   │   ├── AccountTab.svelte
│   │   │   │   │   ├── LlmTab.svelte
│   │   │   │   │   ├── NotificationTab.svelte
│   │   │   │   │   ├── ShortcutTab.svelte
│   │   │   │   │   └── UsageTab.svelte
│   │   │   │   ├── AiPanel.svelte
│   │   │   │   ├── Button.svelte
│   │   │   │   ├── CalendarPanel.svelte
│   │   │   │   ├── ComposeModal.svelte
│   │   │   │   ├── ConfirmDeleteDialog.svelte
│   │   │   │   ├── EventCard.svelte
│   │   │   │   ├── Icon.svelte
│   │   │   │   ├── MailDetail.svelte
│   │   │   │   ├── MailList.svelte
│   │   │   │   ├── Settings.svelte
│   │   │   │   ├── ShortcutManager.svelte
│   │   │   │   ├── Sidebar.svelte
│   │   │   │   ├── ToastNotification.svelte
│   │   │   │   └── UpdateBanner.svelte
│   │   │   ├── stores/             # 状態管理（Svelte 5 runes ベース）
│   │   │   │   ├── mail.ts         # メール状態
│   │   │   │   ├── settings.ts     # 設定状態
│   │   │   │   └── ui.ts           # UI 状態
│   │   │   ├── mailActions.ts      # メール操作アクション
│   │   │   ├── mailHandlers.ts     # メールイベントハンドラ
│   │   │   ├── mailSync.ts         # メール同期ロジック
│   │   │   ├── store.ts            # 設定永続化 (Tauri Store)
│   │   │   └── types.ts            # TypeScript 型定義
│   │   ├── test/                   # テストユーティリティ
│   │   │   └── __mocks__/          # Tauri モック
│   │   ├── app.css                 # グローバルCSS (Catppuccin Mocha)
│   │   ├── app.d.ts               # グローバル型定義
│   │   └── app.html                # HTML テンプレート
│   ├── src-tauri/                  # Backend (Rust)
│   │   ├── src/
│   │   │   ├── lib.rs              # Tauri コマンド定義・型・invoke_handler
│   │   │   ├── main.rs             # エントリポイント
│   │   │   ├── ai_client.rs        # LLM 統合 (OpenAI互換 + Bedrock)
│   │   │   ├── ai_usage.rs         # トークン使用量・コスト追跡
│   │   │   ├── calendar.rs         # カレンダーイベント検出・登録
│   │   │   ├── error.rs            # エラー型定義
│   │   │   ├── ics_parser.rs       # ICS ファイルパース
│   │   │   ├── idle_watcher.rs     # アイドル検出（自動切断）
│   │   │   ├── imap_client.rs      # IMAP 操作（async-imap）
│   │   │   ├── keychain.rs         # macOS Keychain 連携
│   │   │   ├── oauth.rs            # Google OAuth 2.0
│   │   │   ├── smtp_client.rs      # SMTP 送信
│   │   │   └── tray.rs             # システムトレイ
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json         # Tauri 設定
│   └── package.json
├── .env.example                    # 環境変数テンプレート
├── ai-native-mailer.md             # 要件定義書
├── TODO.md                         # 進捗管理
├── CLAUDE.md                       # AI アシスタント向けコンテキスト
└── .kiro/steering/                 # ステアリングファイル
```

## アーキテクチャパターン

- **SPA 構成**: `+page.svelte` に全画面ロジックを集約（ルーティングなし）
- **Tauri IPC**: Frontend → `invoke()` → Rust コマンド（`lib.rs` で定義）
- **データフロー**: ローカル DB なし。メールは IMAP から都度取得。設定のみ `settings.json` に永続化
- **モジュール分割**: Rust 側は機能ごとに 1 ファイル（imap, smtp, ai, calendar, oauth, idle_watcher, keychain, tray, error）
- **状態管理**: `lib/stores/` に分離（mail, settings, ui）。メール操作ロジックは `mailActions.ts`, `mailHandlers.ts`, `mailSync.ts` に分割
- **設定画面**: `components/settings/` にタブ単位でコンポーネント分割
