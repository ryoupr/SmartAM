# プロジェクト概要

SmartAM は AI をネイティブに統合した macOS デスクトップメールクライアント。

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | SvelteKit (Svelte 5 runes) + TypeScript |
| Backend | Rust (Tauri v2) |
| Mail | `async-imap`, `lettre`, `mailparse` |
| AI | OpenAI-compatible `/v1/chat/completions` + Bedrock Converse API |
| Calendar | Apple Calendar (AppleScript), Google Calendar (REST API) |
| Updater | `tauri-plugin-updater` + GitHub Releases (`latest.json`) |

## アーキテクチャ

- Frontend: SPA (`+page.svelte` にほぼ全ロジック集約)
- Backend: Tauri コマンド (`lib.rs`) → 各モジュール (`imap_client.rs`, `ai_client.rs` 等)
- 設定: `@tauri-apps/plugin-store` → `settings.json`（ローカル永続化）

## ビルド・開発コマンド

```bash
cd app
npm run dev              # Vite dev server (:5173)
npx tauri dev            # Tauri desktop app (Vite + Rust)
npm run check            # TypeScript type check
cd src-tauri && cargo check   # Rust check
npx tauri build          # プロダクションビルド（.env に署名鍵が必要）
```
