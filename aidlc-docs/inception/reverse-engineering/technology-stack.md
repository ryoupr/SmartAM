# Technology Stack

## Programming Languages
- Rust 1.77+ — バックエンド（メール通信・AI API・カレンダー）
- TypeScript 6.0 — フロントエンド型システム
- Svelte 5 — UIコンポーネント（runes構文）

## Frameworks
- Tauri v2 (2.10.3) — デスクトップアプリフレームワーク
- SvelteKit (2.57) — フロントエンドアプリフレームワーク
- Vite (8.0) — ビルドツール・開発サーバー

## Infrastructure
- macOS WebKit (WKWebView) — レンダリングエンジン
- Tauri Updater Plugin — 自動更新
- DMG配布 — インストーラー

## Build Tools
- Cargo — Rustビルド
- npm — フロントエンドパッケージ管理
- Tauri CLI (2.10) — アプリバンドル生成

## Testing Tools
- なし（未導入）
- 利用可能: `svelte-check`（型チェック）、`cargo check`（Rustチェック）
