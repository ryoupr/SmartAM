# CLAUDE.md

This file provides guidance to AI coding assistants (Kiro, Claude Code, etc.) when working with code in this repository.

## Project Overview

SmartAM is an AI-native desktop email client built with **Tauri v2** (Rust backend + SvelteKit frontend). It integrates LLM capabilities for mail summarization, reply drafting, translation, and calendar event detection. Target platform is macOS.

Requirements doc: `ai-native-mailer.md` | Progress tracking: `TODO.md` | UI wireframes: `screens.drawio`

## Tech Stack

- **Frontend**: SvelteKit (Svelte 5 with runes mode) + TypeScript, static adapter for Tauri
- **Backend**: Rust (Tauri v2 core)
- **Mail**: `async-imap` (IMAP), `lettre` (SMTP), `mailparse` (parsing)
- **AI**: OpenAI-compatible `/v1/chat/completions` endpoint (LiteLLM proxy planned as sidecar)
- **Tooling**: mise (`rust=latest`, `node=22.12`)
- **Design**: Catppuccin Mocha dark theme (CSS custom properties in `app/src/app.css`)

## Setup

```bash
cp .env.example .env
# Edit .env with your Google OAuth credentials
```

Required environment variables (see `.env.example`):
- `GOOGLE_OAUTH_CLIENT_ID` — Google OAuth 2.0 client ID
- `GOOGLE_OAUTH_CLIENT_SECRET` — Google OAuth 2.0 client secret

## Development Commands

All commands run from `app/` directory:

```bash
# Frontend dev server (Vite on :5173)
cd app && npm run dev

# Tauri desktop app (starts Vite + Rust build)
cd app && npx tauri dev

# Type check
cd app && npm run check

# Rust build only
cd app/src-tauri && cargo build

# Rust check (fast feedback)
cd app/src-tauri && cargo check

# Production build
cd app && npx tauri build
```

## Architecture

### Frontend (`app/src/`)

- `routes/+page.svelte` — Single-page app entry; orchestrates all state (mails, settings, compose, polling)
- `routes/+layout.ts` — SSR disabled (`export const ssr = false`)
- `lib/store.ts` — Settings persistence via `@tauri-apps/plugin-store` (writes `settings.json`); exports `AppSettings`, `loadSettings`, `saveSettings`, config helpers (`getImapConfig`, `getSmtpConfig`, `getLlmConfig`)
- `lib/types.ts` — TypeScript interfaces mirroring Rust structs (`MailSummary`, `MailDetail`, `Attachment`, etc.)
- `lib/components/` — UI components: `Sidebar`, `MailList`, `MailDetail`, `ComposeModal`, `Settings`, `AiPanel`, `CalendarPanel`

Frontend calls Rust via `invoke()` from `@tauri-apps/api/core`. All Tauri commands are defined in `lib.rs`.

### Backend (`app/src-tauri/src/`)

- `lib.rs` — Tauri command definitions and shared types (`AccountConfig`, `SmtpConfig`, `LlmConfig`, `MailSummary`, `MailDetail`, etc.). All `#[tauri::command]` functions registered in `invoke_handler`.
- `imap_client.rs` — IMAP operations: connect, fetch list/detail, folders, thread search, archive, delete, star, attachment download. Each operation opens a new TLS connection.
- `smtp_client.rs` — SMTP send (plain text and with attachments) via STARTTLS.
- `ai_client.rs` — LLM integration: summarize, draft_nuances (returns JSON array of nuance suggestions), draft_reply, translate. All use a shared `chat()` function hitting `/v1/chat/completions`.
- `ai_usage.rs` — AI token usage tracking and cost estimation per model. In-memory aggregation with per-model pricing lookup.
- `calendar.rs` — Calendar event detection via LLM + Apple Calendar registration via `osascript` (AppleScript).
- `oauth.rs` — Google OAuth 2.0 flow (loopback redirect). Client ID/Secret loaded from environment variables at runtime.
- `trace.rs` — File-based logging to `/var/log/smartam/trace.log` (fallback: `~/.smartam/logs/trace.log`).

### Tauri Plugins

Registered in `lib.rs` setup: `tauri-plugin-log`, `tauri-plugin-store`, `tauri-plugin-notification`, `tauri-plugin-dialog`.

### LLM Provider Routing

`store.ts::getLlmConfig()` maps provider selection to `base_url` + model string:
- Ollama: direct to `http://localhost:11434` with `ollama/` prefix
- Others (OpenAI, Anthropic, Bedrock, Gemini): through LiteLLM proxy at `http://localhost:4000` with provider prefix

### Data Flow

Settings stored via Tauri store plugin → `settings.json`. No local database; mail is fetched live from IMAP on each view. Polling interval configurable (default 5 min).

## Security

- **No secrets in source code.** OAuth credentials are loaded from environment variables at runtime.

## Git Workflow

- デフォルト branch: `main`（リリース用）、`develop`（開発用）
- feature branch は必ず `develop` から切る
- PR のターゲットは `develop`
- リリース時のみ `develop` → `main` にマージし、タグを打つ

```
develop → feature/xxx → PR → develop → (リリース時) → main + tag
```

### 手順

1. `git checkout develop && git pull origin develop`
2. `git checkout -b feature/xxx`
3. 実装・検証（`cargo check` + `npm run check`）
4. commit → push → PR（base: `develop`）→ マージ
5. リリース時: `develop` → `main` にマージ、バージョンバンプ、タグ作成

### リリースフロー

1. **バージョンバンプ** — 3ファイルを同時に更新してコミット:
   - `app/package.json` / `app/src-tauri/Cargo.toml` / `app/src-tauri/tauri.conf.json`
   - コミット: `chore: bump version to X.Y.Z`

2. **develop → main マージ**（⚠️ マージ後に develop を削除しない）:
   ```bash
   git checkout main && git pull origin main
   git merge develop --no-edit && git push origin main
   ```

3. **ビルド**（`.env` に `TAURI_SIGNING_PRIVATE_KEY` と `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` が必要）:
   ```bash
   cd app && npx tauri build
   ```
   成功すると以下が生成される:
   - `target/release/bundle/dmg/SmartAM_X.Y.Z_aarch64.dmg`
   - `target/release/bundle/macos/SmartAM.app.tar.gz`（updater 用）
   - `target/release/bundle/macos/SmartAM.app.tar.gz.sig`（署名）

   ⚠️ `Finished 1 updater signature` が出ない場合、署名鍵が未設定。`.env` を確認。

4. **latest.json 生成**:
   ```bash
   cd /path/to/SmartAM
   python3 -c "
   import json, datetime
   sig = open('app/src-tauri/target/release/bundle/macos/SmartAM.app.tar.gz.sig').read().strip()
   json.dump({
     'version': 'X.Y.Z',
     'notes': 'リリースノート',
     'pub_date': datetime.datetime.now(datetime.UTC).strftime('%Y-%m-%dT%H:%M:%SZ'),
     'platforms': {'darwin-aarch64': {
       'signature': sig,
       'url': 'https://github.com/ryoupr/SmartAM/releases/download/vX.Y.Z/SmartAM.app.tar.gz'
     }}
   }, open('/tmp/latest.json','w'), indent=2, ensure_ascii=False)
   "
   ```

5. **タグ + GitHub Release**:
   ```bash
   git tag vX.Y.Z && git push origin vX.Y.Z
   gh release create vX.Y.Z \
     app/src-tauri/target/release/bundle/dmg/SmartAM_X.Y.Z_aarch64.dmg \
     app/src-tauri/target/release/bundle/macos/SmartAM.app.tar.gz \
     app/src-tauri/target/release/bundle/macos/SmartAM.app.tar.gz.sig \
     /tmp/latest.json \
     --title "vX.Y.Z" --target main --notes "リリースノート"
   ```

6. **develop を main に同期**:
   ```bash
   git checkout develop && git merge main --no-edit && git push origin develop
   ```

7. **確認**: `gh release view vX.Y.Z`

### 署名鍵

- `.env` に `TAURI_SIGNING_PRIVATE_KEY`（base64）と `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` を設定
- `mise.toml` の `[env] _.file = ".env"` で自動読み込み
- `.env` は `.gitignore` 済み — 絶対にコミットしない

### バージョン管理

バージョンは以下の3ファイルを同時に更新する:
- `app/package.json`
- `app/src-tauri/Cargo.toml`
- `app/src-tauri/tauri.conf.json`
- `.env` files are gitignored. Use `.env.example` as a template.

## UI/Design Conventions

- Catppuccin Mocha palette: `--base:#1e1e2e`, `--green:#a6e3a1`, `--red:#f38ba8`, `--yellow:#f9e2af`, `--blue:#89b4fa`, `--mauve:#cba6f7`
- AI feature buttons have color-coded active states (green=summary, yellow=draft, blue=translate/calendar)
- Japanese UI labels throughout; error messages from Rust are also in Japanese
- Svelte 5 runes: use `$state()`, `$derived()`, `$props()` — not Svelte 4 stores
