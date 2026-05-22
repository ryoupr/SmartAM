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

## ログ

- `log` クレートのマクロのみ使用（独自ログ関数は作らない）
- `tauri-plugin-log` がバックエンド（ファイル出力 + コンソール）
- 設定画面からランタイムでレベル変更可能（`log::set_max_level`）
- レベル基準:
  - `error!` — アプリが正常動作できない
  - `warn!` — 問題だが動作は継続する
  - `info!` — ユーザー操作の結果、起動/終了
  - `debug!` — 処理の開始/完了、引数の値
  - `trace!` — 大量データ、ループ内の値
- モジュールパスは自動付与されるため手動タグ不要
