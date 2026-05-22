# Components

## Frontend Components (After Refactoring)

### Core Layout
| Component | Purpose | 現状 |
|-----------|---------|------|
| `App.svelte` | アプリルート（レイアウト制御のみ） | 新規（+page.svelteから分離） |
| `Sidebar.svelte` | サイドバー（アカウント・フォルダ） | 既存維持 |
| `MailList.svelte` | メール一覧（仮想スクロール対応） | 既存改修 |
| `MailDetail.svelte` | メール詳細表示 | 既存維持 |

### State Management (新規)
| Module | Purpose |
|--------|---------|
| `stores/mail.ts` | メール状態（一覧・選択・フォルダ・検索） |
| `stores/settings.ts` | アプリ設定（既存store.tsから分離） |
| `stores/ai.ts` | AI機能状態（パネル開閉・処理中フラグ） |
| `stores/ui.ts` | UI状態（トースト・モーダル・ショートカット） |

### Extracted from +page.svelte (新規)
| Component | Purpose | 抽出元 |
|-----------|---------|--------|
| `ShortcutManager.svelte` | キーボードショートカット制御 | +page.svelte |
| `ToastNotification.svelte` | トースト通知表示 | +page.svelte |
| `UpdateBanner.svelte` | アプリ更新通知 | +page.svelte |
| `MailSync.svelte` | メール同期制御（ポーリング・プリフェッチ） | +page.svelte |

### Extracted from Settings.svelte (新規)
| Component | Purpose | 抽出元 |
|-----------|---------|--------|
| `settings/AccountTab.svelte` | アカウント管理タブ | Settings.svelte |
| `settings/LlmTab.svelte` | LLMプロバイダー設定タブ | Settings.svelte |
| `settings/UsageTab.svelte` | AI利用状況タブ | Settings.svelte |
| `settings/ShortcutTab.svelte` | ショートカット設定タブ | Settings.svelte |
| `settings/NotificationTab.svelte` | 通知設定タブ | Settings.svelte |
| `settings/CalendarTab.svelte` | カレンダー連携タブ | Settings.svelte |

---

## Backend Components (After Refactoring)

### Error Handling (新規)
| Module | Purpose |
|--------|---------|
| `error.rs` | 統合エラー型定義（`AppError` enum） |
| `error/imap.rs` | IMAP固有エラー（`ImapError`） |
| `error/ai.rs` | AI API固有エラー（`AiError`） |
| `error/calendar.rs` | カレンダー固有エラー（`CalendarError`） |
| `error/auth.rs` | 認証固有エラー（`AuthError`） |

### Security (新規)
| Module | Purpose |
|--------|---------|
| `keychain.rs` | macOS Keychain統合（全認証情報の保存・取得） |

### Existing (改修)
| Module | 改修内容 |
|--------|---------|
| `imap_client.rs` | async-imap移行、エラー型適用 |
| `ai_client.rs` | エラー型適用、dead code除去 |
| `oauth.rs` | Keychain統合 |
| `lib.rs` | コマンド定義のみに縮小、FEエラー型変換 |
