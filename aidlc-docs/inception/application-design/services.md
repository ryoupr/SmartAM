# Services

## Service Layer Overview

SmartAMはデスクトップアプリのため、従来のマイクロサービスではなく「Tauriコマンド = サービスエンドポイント」として機能する。

## Service Definitions

### Mail Service (imap_client.rs + smtp_client.rs)
- **Responsibility**: メールの取得・送信・操作
- **Orchestration**: lib.rsのTauriコマンドから呼び出し
- **Error Handling**: `ImapError` / `SmtpError` → `FrontendError`変換

### AI Service (ai_client.rs + ai_usage.rs)
- **Responsibility**: LLM API呼び出し + トークン使用量追跡
- **Orchestration**: 要約・下書き・翻訳・カレンダー検出の各コマンド
- **Error Handling**: `AiError` → `FrontendError`変換
- **Budget Check**: 呼び出し前に月額上限チェック

### Auth Service (oauth.rs + keychain.rs)
- **Responsibility**: 認証情報の管理・トークンリフレッシュ
- **Orchestration**: 
  - 起動時: Keychainから認証情報ロード
  - API呼び出し前: トークン有効期限チェック → 必要ならリフレッシュ
  - 設定保存時: Keychainに認証情報保存
- **Error Handling**: `AuthError` → `FrontendError`変換

### Calendar Service (calendar.rs + ics_parser.rs)
- **Responsibility**: カレンダーイベント検出・登録・重複チェック
- **Orchestration**: AI検出 → ユーザー確認 → 登録
- **Error Handling**: `CalendarError` → `FrontendError`変換

## Frontend Service Layer (stores/)

```
+page.svelte (thin orchestrator)
    ├── stores/mail.ts      → invoke('fetch_mails', ...)
    ├── stores/settings.ts  → invoke('save_settings', ...) + Keychain
    ├── stores/ai.ts        → invoke('ai_summarize', ...)
    └── stores/ui.ts        → local state only
```

**原則**: コンポーネントはstoreのアクションを呼ぶだけ。invokeの直接呼び出しはstore内に閉じ込める。
