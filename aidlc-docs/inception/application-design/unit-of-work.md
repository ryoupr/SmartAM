# Unit of Work

## Decomposition Strategy
- **Type**: Monolith (論理モジュール分割)
- **Release Strategy**: Unit 1を先行リリース → 残りは後続リリース

---

## Unit Definitions

### Unit 1: バグ修正（HTMLリンク）
- **Priority**: Critical — 先行リリース
- **Scope**: MailDetail.svelte iframe内リンククリック問題の修正
- **Deliverables**:
  - 原因特定・修正
  - 修正の動作確認テスト
- **Estimated Size**: Small

### Unit 2: Frontend リファクタリング
- **Priority**: High
- **Scope**: God Component分割 + Store分割
- **Deliverables**:
  - +page.svelte → thin orchestrator + stores/分割
  - Settings.svelte → 6タブコンポーネント分割
  - ShortcutManager, ToastNotification, UpdateBanner, MailSync 抽出
- **Estimated Size**: Large

### Unit 3: Backend リファクタリング
- **Priority**: High
- **Scope**: エラーハンドリング構造化 + Dead Code除去
- **Deliverables**:
  - error.rs + モジュール別エラー型（thiserror）
  - FrontendError シリアライズ型
  - lib.rs縮小（コマンド定義のみ）
  - LiteLLM残骸コード除去
- **Estimated Size**: Medium

### Unit 4: セキュリティ（Keychain移行）
- **Priority**: High
- **Scope**: 全認証情報のmacOS Keychain移行
- **Deliverables**:
  - keychain.rs モジュール
  - settings.json → Keychain マイグレーション
  - OAuth/IMAP/LLM認証情報のKeychain保存・取得
- **Dependencies**: Unit 3（エラー型を使用）
- **Estimated Size**: Medium

### Unit 5: パフォーマンス
- **Priority**: Medium
- **Scope**: 仮想スクロール + async-imap移行
- **Deliverables**:
  - MailList.svelte 仮想スクロール実装
  - imap_client.rs async-imap移行
  - パフォーマンスベンチマーク
- **Dependencies**: Unit 2（MailList改修）, Unit 3（エラー型）
- **Estimated Size**: Large

### Unit 6: テスト基盤 + CI/CD
- **Priority**: Medium
- **Scope**: テストフレームワーク導入 + GitHub Actions
- **Deliverables**:
  - Rust: cargo test基盤 + ユニットテスト
  - Frontend: Vitest + testing-library/svelte + Playwright
  - GitHub Actions: lint + test + build
  - カバレッジ80%+
- **Dependencies**: Unit 2, 3（リファクタ後のコードに対してテスト）
- **Estimated Size**: Large
