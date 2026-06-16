# Tech Stack & Build

## ランタイム要件

- macOS (aarch64)
- Rust stable (1.77.2+)
- Node.js 22+ (mise で管理)

## Frontend

| 項目 | 技術 |
|------|------|
| フレームワーク | SvelteKit + Svelte 5 (runes mode) |
| 言語 | TypeScript 6 |
| ビルド | Vite 8 |
| アダプタ | `@sveltejs/adapter-static`（Tauri 向け SSG） |
| 状態管理 | Svelte 5 runes (`$state`, `$derived`, `$props`) |
| 設定永続化 | `@tauri-apps/plugin-store` → `settings.json` |
| チャート | `chart.js` + `svelte-chartjs` |
| テスト | `vitest` + `@testing-library/svelte` + `jsdom` |

## Backend (Rust)

| 項目 | クレート |
|------|---------|
| デスクトップ | Tauri v2 (`tauri 2.10`) |
| IMAP | `async-imap 0.11` + `async-native-tls` (runtime-tokio) |
| SMTP | `lettre 0.11` |
| メールパース | `mailparse 0.15` |
| HTTP | `reqwest 0.12` |
| 非同期 | `tokio` (rt-multi-thread, macros, net, time) |
| シリアライズ | `serde` + `serde_json` |
| カレンダー | `ical 0.11` + AppleScript (`osascript`) |
| OAuth | カスタム実装 (loopback redirect) |
| ユーティリティ | `chrono`, `regex`, `base64`, `dirs`, `url`, `rand` |
| 通知 | `notify-rust` |
| 環境変数 | `dotenvy` |

## Tauri プラグイン

`tauri-plugin-log`, `tauri-plugin-store`, `tauri-plugin-notification`, `tauri-plugin-dialog`, `tauri-plugin-updater`, `tauri-plugin-process`

## 開発コマンド

```bash
cd app

# 開発
npm run dev              # Vite dev server (:5173)
npx tauri dev            # Tauri アプリ起動（Vite + Rust）

# 検証
npm run check            # Svelte + TypeScript 型チェック
cd src-tauri && cargo check   # Rust 型チェック（高速）
npm run test             # vitest (単発実行)

# ビルド
npx tauri build          # プロダクションビルド（署名鍵が必要）
```

## LLM プロバイダー構成

- Ollama: `http://localhost:11434` に直接接続
- OpenAI / Anthropic / Bedrock / Gemini: LiteLLM proxy (`http://localhost:4000`) 経由
- API は全て OpenAI 互換 `/v1/chat/completions` + Bedrock Converse API
