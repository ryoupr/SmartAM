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
│   │   │   │   ├── AiPanel.svelte
│   │   │   │   ├── CalendarPanel.svelte
│   │   │   │   ├── ComposeModal.svelte
│   │   │   │   ├── EventCard.svelte
│   │   │   │   ├── MailDetail.svelte
│   │   │   │   ├── MailList.svelte
│   │   │   │   ├── Settings.svelte
│   │   │   │   └── Sidebar.svelte
│   │   │   ├── store.ts            # 設定永続化 (Tauri Store)
│   │   │   └── types.ts            # TypeScript 型定義
│   │   ├── app.css                 # グローバルCSS (Catppuccin Mocha)
│   │   └── app.html                # HTML テンプレート
│   ├── src-tauri/                  # Backend (Rust)
│   │   ├── src/
│   │   │   ├── lib.rs              # Tauri コマンド定義・型・invoke_handler
│   │   │   ├── main.rs             # エントリポイント
│   │   │   ├── imap_client.rs      # IMAP 操作
│   │   │   ├── smtp_client.rs      # SMTP 送信
│   │   │   ├── ai_client.rs        # LLM 統合 (OpenAI互換 + Bedrock)
│   │   │   ├── ai_usage.rs         # トークン使用量・コスト追跡
│   │   │   ├── calendar.rs         # カレンダーイベント検出・登録
│   │   │   ├── ics_parser.rs       # ICS ファイルパース
│   │   │   ├── oauth.rs            # Google OAuth 2.0
│   │   │   └── ics_parser.rs       # ICS ファイルパース
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
- **モジュール分割**: Rust 側は機能ごとに 1 ファイル（imap, smtp, ai, calendar, oauth）
