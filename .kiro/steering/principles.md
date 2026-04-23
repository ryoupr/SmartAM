# コーディング原則

- 最小限のコードのみ書く
- テストコードは明示的に要求されない限り書かない
- 憶測で修正しない — ログや実測で確認してから直す

## UI

- Catppuccin Mocha パレット
- Material Design 的なコンポーネント設計
- 日本語 UI ラベル、Rust 側のエラーメッセージも日本語
- Svelte 5 runes: `$state()`, `$derived()`, `$props()` を使用（Svelte 4 stores は使わない）

## セキュリティ

- `.env` は `.gitignore` 済み — 絶対にコミットしない
- `TAURI_SIGNING_PRIVATE_KEY` / OAuth credentials はローカルの `.env` で管理
- API keys (LLM providers 等) は Tauri Store (`settings.json`) でユーザーローカルに保存
