# Product

SmartAM は macOS 向けの AI ネイティブデスクトップメールクライアント。
メールの要約・返信下書き・翻訳・カレンダー登録を LLM でワンクリック実行できる。

## コアバリュー

- AI はオンデマンド（ボタン押下時のみ動作、トークン節約）
- Gmail ライクなキーボードショートカット
- 複数アカウント対応（OAuth + 手動 IMAP/SMTP）
- ローカル LLM（Ollama）からクラウド（OpenAI, Anthropic, Bedrock, Gemini）まで対応
- トークン使用量・コスト可視化（チャート表示）
- システムトレイ常駐 + アイドル時自動切断
- macOS Keychain によるセキュアな認証情報管理
- 自動アップデート（GitHub Releases + tauri-plugin-updater）

## ターゲット

- macOS (Apple Silicon) ユーザー
- 日本語 UI

## 現在のバージョン

- v0.4.15

## 要件・設計ドキュメント

- `ai-native-mailer.md` — 全機能要件
- `TODO.md` — 進捗トラッキング
