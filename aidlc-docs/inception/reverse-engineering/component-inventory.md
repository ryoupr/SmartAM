# Component Inventory

## Application Packages
- `app/src-tauri/` — Rustバックエンド（Tauri v2コア）
- `app/src/` — SvelteKitフロントエンド

## Infrastructure Packages
- なし（デスクトップアプリのため）

## Shared Packages
- `app/src/lib/types.ts` — 共有型定義
- `app/src/lib/store.ts` — 設定管理ユーティリティ
- `app/src/lib/stores/` — ドメイン別状態管理（mail.ts, settings.ts, ui.ts）

## Test Packages
- `app/src/test/` — テストヘルパー・モック
- `app/src/lib/store.test.ts` — Store単体テスト
- `app/src/lib/config.test.ts` — 設定テスト
- `app/src/lib/stores/ui.test.ts` — UIストアテスト

## UI Component Packages
- `app/src/lib/components/` — UIコンポーネント（15ファイル）
  - `Button.svelte` — 汎用ボタン（バリアント対応）
  - `Icon.svelte` — 12種SVG線アイコン + 翻訳テキストアイコン
  - `Sidebar.svelte` — サイドバー（智マーク + ワードマーク）
  - `MailList.svelte` — メール一覧（仮想スクロール）
  - `MailDetail.svelte` — メール詳細 + ツールバー
  - `AiPanel.svelte` — AI機能パネル
  - `CalendarPanel.svelte` — カレンダー登録
  - `ComposeModal.svelte` — メール作成
  - `Settings.svelte` — 設定モーダル
  - `ShortcutManager.svelte` — キーボードショートカット
  - `ToastNotification.svelte` — トースト通知
  - `UpdateBanner.svelte` — アプリ更新通知
  - `ConfirmDeleteDialog.svelte` — 削除確認
  - `EventCard.svelte` — カレンダーイベント表示
  - `settings/` — 設定タブ（AccountTab, LlmTab, UsageTab, ShortcutTab）

## Total Count
- **Total Packages**: 2（Frontend + Backend）
- **Application**: 2
- **Infrastructure**: 0
- **Shared**: 3（types.ts, store.ts, stores/）
- **Test**: 4（テストファイル）
- **UI Components**: 15
