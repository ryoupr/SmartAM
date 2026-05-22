# Components

## Frontend Components

### Design System (v0.4.0 リブランド)
| Component | Purpose | Props |
|-----------|---------|-------|
| `Button.svelte` | 汎用ボタン（線アイコン + ラベル + kbd） | label, icon, kbd, variant, active, disabled, title, onclick |
| `Icon.svelte` | SVG線アイコン（12種 + 翻訳テキスト） | name, size |

**Button バリアント**: primary, danger, starred, ai-summary, ai-draft, ai-translate, ai-calendar, ai-regen, ghost

**Icon 名**: reply, forward, archive, trash, star, summary, draft, translate, calendar, regen, send, attach

### Core Layout
| Component | Purpose | 備考 |
|-----------|---------|------|
| `Sidebar.svelte` | サイドバー（智マーク + ワードマーク、フォルダ、LLM badge） | 絵文字廃止済み |
| `MailList.svelte` | メール一覧（仮想スクロール、380px幅） | ヘッダー: タイトル+カウント+更新ボタン |
| `MailDetail.svelte` | メール詳細 + ツールバー（Button.svelte使用） | Auto-collapse対応（≤600px → icon-only） |
| `AiPanel.svelte` | AI機能パネル（要約・下書き・翻訳） | Icon.svelte使用 |
| `CalendarPanel.svelte` | カレンダーイベント検出・登録 | |
| `ComposeModal.svelte` | メール作成（Icon.svelte使用） | |
| `EventCard.svelte` | ICSイベント表示・承諾/辞退 | |

### Utility
| Component | Purpose |
|-----------|---------|
| `ToastNotification.svelte` | トースト通知（ink背景、slide-in、左border色） |
| `ShortcutManager.svelte` | キーボードショートカット制御 |
| `UpdateBanner.svelte` | アプリ更新通知バナー |
| `ConfirmDeleteDialog.svelte` | 削除確認ダイアログ |
| `Settings.svelte` | 設定モーダル（智マーク、動的バージョン表示） |

### Settings Tabs
| Component | Purpose |
|-----------|---------|
| `settings/AccountTab.svelte` | メールアカウント管理 |
| `settings/LlmTab.svelte` | LLMプロバイダー設定 |
| `settings/UsageTab.svelte` | AI利用状況・コスト |
| `settings/ShortcutTab.svelte` | ショートカットカスタマイズ |

### State Management
| Module | Purpose |
|--------|---------|
| `stores/mail.ts` | メール状態（一覧・選択・フォルダ） |
| `stores/settings.ts` | アプリ設定（loadSettings/saveSettings ラッパー） |
| `stores/ui.ts` | UI状態（トースト・モーダル） |

---

## Backend Components (Rust)

### Core
| Module | Purpose |
|--------|---------|
| `lib.rs` | Tauriコマンド定義 |
| `imap_client.rs` | async-imap（接続プール・キャッシュ・全IMAP操作） |
| `smtp_client.rs` | SMTP送信 |
| `ai_client.rs` | LLM統合（OpenAI互換 + Bedrock Converse） |
| `ai_usage.rs` | トークン使用量追跡・コスト計算 |
| `oauth.rs` | Google OAuth 2.0 |
| `calendar.rs` | カレンダーイベント検出・登録 |
| `ics_parser.rs` | ICSファイルパース |
