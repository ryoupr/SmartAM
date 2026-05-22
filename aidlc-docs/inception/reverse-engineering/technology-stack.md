# Technology Stack

## Programming Languages
- Rust 1.77+ — バックエンド（メール通信・AI API・カレンダー）
- TypeScript 6.0 — フロントエンド型システム
- Svelte 5 — UIコンポーネント（runes構文）

## Frameworks
- Tauri v2 (2.10.3) — デスクトップアプリフレームワーク
- SvelteKit (2.57) — フロントエンドアプリフレームワーク
- Vite (8.0) — ビルドツール・開発サーバー

## Design System
- **ブランド**: 智(CHI)ロゴ + 赤バー (#C8281E)
- **テーマ**: ダーク (Catppuccin Mocha) / ライト (Cream/Paper/Bone)
- **フォント**: Inter (UI), Noto Sans JP (日本語), JetBrains Mono (コード)
- **アイコン**: 1.5px ストローク SVG 線アイコン（Icon.svelte、12種）
- **CSS変数**: デザイントークン（色・spacing・radius・shadow）
- **テーマ切替**: `[data-theme="light"|"dark"]` 属性

## Infrastructure
- macOS WebKit (WKWebView) — レンダリングエンジン
- Tauri Updater Plugin — 自動更新（latest.json + 署名）
- DMG配布 — インストーラー
- Google Fonts — Web フォント配信

## Build Tools
- Cargo — Rustビルド
- npm — フロントエンドパッケージ管理
- Tauri CLI (2.10) — アプリバンドル生成・アイコン生成
- mise — ランタイムバージョン管理

## Testing Tools
- Vitest — フロントエンド単体テスト
- @testing-library/svelte — コンポーネントテスト
- svelte-check — TypeScript + Svelte 型チェック
- cargo check / cargo clippy — Rust 静的解析
