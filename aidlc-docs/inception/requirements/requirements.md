# Requirements — SmartAM 全方位強化

## Intent Analysis

- **User Request**: SmartAMを全方向に強化したい + HTMLメール内リンクバグ修正
- **Request Type**: Enhancement + Bug Fix
- **Scope**: System-wide
- **Complexity**: Complex
- **Approach**: 全並行（ユニット分割して並列開発）
- **Phase 2機能**: 含めない（既存品質向上に集中）

---

## Functional Requirements

### FR-1: HTMLメール内リンクの外部ブラウザ起動（バグ修正）

- **優先度**: Critical
- **現状**: 全HTMLメールでリンクをクリックしても何も起きない
- **期待動作**: リンククリック → OSデフォルトブラウザで開く
- **原因調査**: iframe srcdoc内のクリックハンドラが正しく動作していない（全HTMLメールで再現）

### FR-2: God Component分割

- **優先度**: High
- **対象**:
  - `+page.svelte` (32KB) → 状態管理・ショートカット・メール操作を分離
  - `Settings.svelte` (35KB) → タブごとにサブコンポーネント化
- **目標**: 各コンポーネント10KB以下、単一責任

### FR-3: 構造化エラーハンドリング

- **優先度**: High
- **現状**: Rust側が`Result<T, String>`を多用
- **目標**: カスタムエラー型（enum）導入、フロントエンドでエラー種別に応じたUI表示

### FR-4: Dead Code除去

- **優先度**: Medium
- **対象**: LiteLLM proxy参照（localhost:4000）の残骸コード除去

---

## Non-Functional Requirements

### NFR-1: テストカバレッジ 80%+

- **Rust**: ユニットテスト（ai_client, imap_client, calendar, oauth）
- **Frontend**: コンポーネントテスト（Vitest + Testing Library）
- **E2E**: Playwright（主要ユーザーフロー）
- **パフォーマンスベンチマーク**: メール取得・AI応答時間

### NFR-2: CI/CD

- **GitHub Actions**: lint + test + build（PR時自動実行）
- **Rust**: `cargo clippy` + `cargo test`
- **Frontend**: `svelte-check` + `vitest`

### NFR-3: セキュリティ

- **APIキー**: macOS Keychainに移行（settings.jsonから平文削除）
- **Security Extension**: 全ルール適用（ブロッキング制約）

### NFR-4: パフォーマンス

- **仮想スクロール**: メール一覧に導入（1000件以上でも60fps維持）
- **非同期IMAP**: 同期`imap` crateから非同期実装へ移行
- **起動時間**: 3秒以内維持
- **メモリ**: アイドル時100MB以下維持

### NFR-5: コード品質

- **Lint**: `cargo clippy`（Rust）、`eslint`（Frontend）
- **フォーマット**: `rustfmt`、`prettier`
- **ドキュメント**: 公開関数にrustdoc/JSDocコメント

---

## Extension Configuration

| Extension              | Enabled | Decided At            |
| ---------------------- | ------- | --------------------- |
| Security Baseline      | Yes     | Requirements Analysis |
| Property-Based Testing | No      | Requirements Analysis |

---

## Summary

既存SmartAMの品質を本番レベルに引き上げる改善プロジェクト。新機能追加は行わず、バグ修正・リファクタリング・テスト・セキュリティ・パフォーマンスの5軸で全並行に強化する。
