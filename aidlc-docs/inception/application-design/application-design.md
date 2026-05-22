# Application Design — SmartAM 全方位強化

## Design Decisions Summary

| 項目 | 決定 |
|------|------|
| Frontend状態管理 | Store module分割 + コンポーネント内$state() |
| Rustエラー型 | モジュール別enum + FrontendError（code/message/retryable） |
| Keychain範囲 | 全認証情報（APIキー + OAuth + IMAPパスワード） |
| 非同期IMAP | async-imap crateに移行 |
| テストFW | Vitest + testing-library/svelte + Playwright |

## Architecture After Refactoring

```
Frontend (SvelteKit/Svelte 5)
├── +page.svelte          ← thin orchestrator (< 200行目標)
├── stores/               ← ドメイン別状態管理 (NEW)
│   ├── mail.ts
│   ├── settings.ts
│   ├── ai.ts
│   └── ui.ts
├── components/           ← UI (各10KB以下目標)
│   ├── Sidebar.svelte
│   ├── MailList.svelte   ← 仮想スクロール対応
│   ├── MailDetail.svelte
│   ├── AiPanel.svelte
│   ├── CalendarPanel.svelte
│   ├── ComposeModal.svelte
│   ├── ShortcutManager.svelte (NEW)
│   ├── ToastNotification.svelte (NEW)
│   └── settings/        ← タブ分割 (NEW)
│       ├── AccountTab.svelte
│       ├── LlmTab.svelte
│       ├── UsageTab.svelte
│       ├── ShortcutTab.svelte
│       ├── NotificationTab.svelte
│       └── CalendarTab.svelte
└── lib/types.ts

Backend (Rust/Tauri v2)
├── lib.rs               ← コマンド定義のみ (縮小)
├── error.rs             ← 統合エラー型 (NEW)
├── error/               ← モジュール別エラー (NEW)
├── keychain.rs          ← macOS Keychain統合 (NEW)
├── imap_client.rs       ← async-imap移行
├── smtp_client.rs
├── ai_client.rs         ← dead code除去
├── ai_usage.rs
├── oauth.rs             ← Keychain統合
├── calendar.rs
└── ics_parser.rs
```

## Key Principles

1. **Single Responsibility**: 各コンポーネント/モジュールは1つの責務
2. **Store-mediated IPC**: invoke()はstore内に閉じ込め、コンポーネントから直接呼ばない
3. **Typed Errors**: 全エラーはFrontendError型でフロントエンドに伝達
4. **Zero Plaintext Secrets**: 認証情報は全てmacOS Keychainに保存
5. **Testable by Design**: store/moduleレベルでmock可能な設計
