# Application Design — SmartAM v0.4.x

## Design Decisions Summary

| 項目 | 決定 |
|------|------|
| Frontend状態管理 | Store module分割 + コンポーネント内$state() |
| Rustエラー型 | モジュール別enum + FrontendError（code/message/retryable） |
| Keychain範囲 | 全認証情報（APIキー + OAuth + IMAPパスワード） |
| 非同期IMAP | async-imap crateに移行済み |
| テストFW | Vitest + testing-library/svelte |
| デザインシステム | 智ロゴ + デザイントークン + 線アイコン + テーマ切替 |
| UIボタン | 絵文字全廃止 → Button.svelte + Icon.svelte |
| テーマ | Light (デフォルト) / Dark 切替（CSS変数 + data-theme属性） |

## Brand & Design System (v0.4.0)

### ブランドマーク
- **智(CHI)**: Noto Sans JP Black 900、アウトライン + 赤バー (#C8281E)
- **ワードマーク**: Smart + 赤AM
- **favicon**: 32×32 塗り版（クリーム背景 + 赤バー）

### デザイントークン (app.css)
- **Dark** (:root): Catppuccin Mocha ベース、`--red: #C8281E`（ブランド赤）
- **Light** ([data-theme="light"]): Cream/Paper/Bone/Ink 系
- **AI機能色**: green(要約), yellow(下書き), blue(翻訳/カレンダー), mauve(再生成)
- **Spacing**: 4px grid (space-1〜space-8)
- **Radius**: xs(3px), sm(6px), md(10px), lg(16px)
- **Shadow**: subtle, card, floating

### テーマ切替
- `AppSettings.theme: 'light' | 'dark'`
- `+layout.svelte` で起動時に `document.documentElement.setAttribute('data-theme', theme)` 適用
- 設定 → 表示設定で即時切替

## Architecture

```
Frontend (SvelteKit/Svelte 5)
├── +page.svelte          ← orchestrator
├── +layout.svelte        ← テーマ適用
├── app.css               ← デザイントークン（Dark/Light）
├── app.html              ← Google Fonts読み込み
├── stores/               ← ドメイン別状態管理
│   ├── mail.ts
│   ├── settings.ts
│   └── ui.ts
├── components/           ← UI
│   ├── Button.svelte     ← 汎用ボタン（バリアント対応）
│   ├── Icon.svelte       ← 12種SVG線アイコン
│   ├── Sidebar.svelte    ← 智マーク + ワードマーク
│   ├── MailList.svelte   ← 仮想スクロール + ヘッダー
│   ├── MailDetail.svelte ← ツールバー（auto-collapse）
│   ├── AiPanel.svelte
│   ├── CalendarPanel.svelte
│   ├── ComposeModal.svelte
│   ├── ToastNotification.svelte ← slide-in + ink背景
│   ├── Settings.svelte   ← 動的バージョン表示
│   └── settings/         ← タブ分割
└── lib/
    ├── store.ts          ← 設定永続化
    └── types.ts

Backend (Rust/Tauri v2)
├── lib.rs               ← コマンド定義
├── imap_client.rs       ← async-imap（接続プール + キャッシュ）
├── smtp_client.rs
├── ai_client.rs
├── ai_usage.rs
├── oauth.rs
├── calendar.rs
└── ics_parser.rs
```

## Key Principles

1. **Single Responsibility**: 各コンポーネント/モジュールは1つの責務
2. **Design Token Driven**: 色・余白・角丸は全てCSS変数経由
3. **No Emoji in UI**: ボタン・ヘッダーは全て線アイコン（メール本文内は許可）
4. **Theme Aware**: 全コンポーネントがLight/Dark両対応
5. **Auto-collapse**: ペイン幅≤600pxでボタンがicon-onlyに縮退
