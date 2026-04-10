# AIネイティブメーラー TODO

最終更新: 2026-04-09

---

## 要件定義

- [x] 技術スタック調査（Tauri vs Electron）
- [x] LLM統合方式調査（LiteLLM）
- [x] 要件定義書作成 → `ai-native-mailer.md`
- [x] 未調査項目の解消（Tauri sidecar / Claude Code統合 / Jira OAuth）
- [x] MVPスコープ確定

---

## MVP

### 基本メール機能
- [ ] Tauri v2 + SvelteKit プロジェクト初期化
- [ ] IMAP接続実装（async-imap）
- [ ] SMTP送信実装（lettre）
- [ ] メール一覧・詳細UI
- [ ] スレッド表示
- [ ] 返信・転送・削除

### AI機能
- [ ] LiteLLM Proxy sidecarセットアップ
- [ ] メール要約
- [ ] 返信文自動生成・提案

### カレンダー連携
- [ ] 日程・イベント情報の検出（LLMで抽出）
- [ ] Apple Calendar登録（EventKit）
- [ ] Google Calendar登録（REST API）

---

## Phase 2

- [ ] 優先度・カテゴリ自動分類
- [ ] アクションアイテム抽出 → Jira登録（OAuth 2.0 3LO）

## Phase 4

- [ ] Claude Code統合（`ANTHROPIC_BASE_URL` 経由）
