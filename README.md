# ✉ SmartAM

AIをネイティブに統合したデスクトップメールクライアント。

![Catppuccin Mocha](https://img.shields.io/badge/theme-Catppuccin%20Mocha-1e1e2e?style=flat&labelColor=cba6f7&color=1e1e2e)
![Platform](https://img.shields.io/badge/platform-macOS-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## SmartAM とは

SmartAM は、メール処理を AI で効率化するデスクトップメーラーです。メールの要約、返信文の自動生成、翻訳、カレンダー登録をワンクリックで実行できます。

AI はボタンを押した時だけ動作するため、トークンを無駄に消費しません。

## 主な機能

### 📬 メール
- Gmail OAuth 2.0 による安全な認証
- 複数アカウント対応（アカウントごとに署名・通知・カレンダーを設定）
- アーカイブ・スター・削除（即座に反映、バックグラウンドで同期）
- 添付ファイルプレビュー（画像・PDF・動画・テキスト・Markdown）

### 🤖 AI
- **📝 要約** — メールの要点を5行以内に凝縮
- **✍ 返信下書き** — AIがニュアンスを提案 → 選択して返信文を生成
- **🌐 翻訳** — メール本文をその場で翻訳（HTMLレイアウト維持）
- **📅 カレンダー** — 日程を自動検出、編集してからカレンダーに登録

### ⌨ キーボードショートカット
Gmail ライクなショートカットで素早く操作。設定画面でカスタマイズ可能。

| キー | 操作 | キー | 操作 |
|------|------|------|------|
| `j` / `k` | 次 / 前のメール | `r` | 返信 |
| `a` | アーカイブ | `f` | 転送 |
| `s` | スター切替 | `c` | 新規作成 |
| `y` | AI要約 | `d` | AI下書き |
| `t` | AI翻訳 | `l` | AIカレンダー |
| `g→i` | 受信トレイへ | `/` | 検索 |

### 🎨 対応 LLM プロバイダー
- Ollama（ローカル実行）
- OpenAI / Anthropic / Google Gemini
- AWS Bedrock（API Key / IAM 認証）

## インストール

### 必要なもの
- macOS
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) 22+
- Google OAuth 2.0 クライアント ID（[Google Cloud Console](https://console.cloud.google.com/apis/credentials) で作成）

### 手順

```bash
# 1. リポジトリをクローン
git clone https://github.com/Ryou6152/SmartAM.git
cd SmartAM

# 2. OAuth 認証情報を設定
cp .env.example .env
# .env を編集して Client ID / Secret を入力

# 3. 依存関係をインストール
cd app && npm install

# 4. 起動
npx tauri dev
```

初回起動後、設定画面（⚙）からメールアカウントを追加してください。

### プロダクションビルド

```bash
cd app && npx tauri build
```

`app/src-tauri/target/release/bundle/` にアプリバンドルが生成されます。

## 使い方

1. **アカウント追加** — 設定 → 「Google でログイン」または手動で IMAP/SMTP を設定
2. **メール閲覧** — サイドバーでアカウント・フォルダを切り替え
3. **AI 機能** — メール詳細画面のボタン（📝 ✍ 🌐 📅）をクリック
4. **ショートカット** — `?` キーで一覧表示

## 設定

設定は全てアプリ内の設定画面（⚙）から変更できます。

| タブ | 内容 |
|------|------|
| メールアカウント | アカウントの追加・削除 |
| LLMプロバイダー | AI プロバイダーの選択・API キー設定 |
| AI 利用状況 | トークン使用量・コスト確認・月額上限設定 |
| キーボードショートカット | キーバインドのカスタマイズ |
| 署名 | アカウントごとの署名テキスト |
| カレンダー連携 | Apple Calendar / Google Calendar の選択 |
| 通知 | 新着通知・サウンド・同期間隔 |

---

## For Developers

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | SvelteKit (Svelte 5 runes) + TypeScript |
| Backend | Rust (Tauri v2) |
| Mail | `async-imap`, `lettre`, `mailparse` |
| AI | OpenAI-compatible `/v1/chat/completions` + Bedrock Converse API |
| Calendar | Apple Calendar (AppleScript), Google Calendar (REST API) |

### Project Structure

```
SmartAM/
├── app/src/
│   ├── routes/+page.svelte        # SPA エントリ（状態管理・ショートカット）
│   └── lib/
│       ├── components/             # Sidebar, MailList, MailDetail, Settings, etc.
│       ├── store.ts                # 設定の永続化 (Tauri Store → settings.json)
│       └── types.ts                # TypeScript 型定義
├── app/src-tauri/src/
│   ├── lib.rs                      # Tauri コマンド定義
│   ├── imap_client.rs              # IMAP 操作（接続プール・フォルダ解決）
│   ├── smtp_client.rs              # SMTP 送信
│   ├── ai_client.rs                # LLM 統合（OpenAI互換 + Bedrock Converse）
│   ├── ai_usage.rs                 # トークン使用量追跡・コスト計算
│   ├── calendar.rs                 # カレンダーイベント検出・登録
│   └── oauth.rs                    # Google OAuth 2.0 (loopback redirect)
├── ai-native-mailer.md             # 要件定義書
├── screens.drawio                  # UI ワイヤーフレーム (24ページ)
└── CLAUDE.md                       # AI アシスタント向けコンテキスト
```

### Development Commands

```bash
cd app
npm run dev              # Vite dev server (:5173)
npx tauri dev            # Tauri desktop app (Vite + Rust)
npm run check            # TypeScript type check
cd src-tauri && cargo check   # Rust check (fast feedback)
```

### Design Docs

- `ai-native-mailer.md` — 全機能要件・技術スタック・開発フェーズ
- `screens.drawio` — 24ページの UI ワイヤーフレーム（draw.io で開く）
- `CLAUDE.md` — AI コーディングアシスタント向けプロジェクトコンテキスト

## License

[MIT](LICENSE)
