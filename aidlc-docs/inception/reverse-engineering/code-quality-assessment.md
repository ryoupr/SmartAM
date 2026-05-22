# Code Quality Assessment (v0.4.1 時点)

## Test Coverage
- **Unit Tests**: 3ファイル（store.test.ts, config.test.ts, ui.test.ts）
- **テストFW**: Vitest + @testing-library/svelte
- **モック**: Tauri API モック（test/__mocks__/tauri.ts, tauri-store.ts）
- **カバレッジ**: 低（Store/設定周りのみ、コンポーネントテスト未着手）

## Code Quality Indicators
- **svelte-check**: エラー 0、警告 28（a11y + state_referenced_locally）
- **cargo check**: 警告 2（dead_code: ai_usage.rs の未使用フィールド）
- **Code Style**: 一貫（Rust: 標準フォーマット、Svelte: インデント統一）
- **デザイン一貫性**: ✅ 絵文字全廃止、Button/Icon コンポーネント統一

## Technical Debt

### 解決済み ✅
- ~~God Component問題~~: Store分割 + ShortcutManager/Toast/UpdateBanner 抽出済み
- ~~同期IMAP~~: async-imap 移行完了
- ~~テスト不在~~: テスト基盤構築済み（Vitest + モック）
- ~~セキュリティ~~: Keychain 統合済み
- ~~仮想スクロール~~: MailList.svelte 実装済み
- ~~CI/CD~~: 基盤構築済み

### 残存（中優先度）
1. **テストカバレッジ不足**: コンポーネントテスト・E2Eテスト未着手
2. **a11y警告**: iframe title属性、click handler にキーボードイベント未付与（28件）
3. **型定義の重複**: Rust側とTypeScript側で同じ型を手動で二重定義
4. **エラーハンドリング**: 一部 `Result<T, String>` が残存（構造化エラー型への移行途中）
5. **+page.svelte**: まだ 18KB（目標は薄いオーケストレーター）

### 残存（低優先度）
6. **i18n**: UI文字列がハードコード（日本語のみ）
7. **cargo clippy 警告**: ai_usage.rs の未使用フィールド
8. **Settings.svelte state_referenced_locally 警告**: Svelte 5 runes パターンの軽微な問題

## Patterns

### Good Patterns ✅
- オンデマンドAI実行（トークン節約）
- Svelte 5 runes による反応的状態管理
- デザイントークン駆動（CSS変数で Light/Dark 統一）
- Button/Icon コンポーネントによるUI一貫性
- Auto-collapse レスポンシブ（ResizeObserver）
- async-imap 接続プール + LRU キャッシュ
- テーマ切替（data-theme 属性）
- 動的バージョン表示（getVersion API）

### Remaining Anti-patterns
- **+page.svelte がまだ大きい**: 状態管理の一部がまだ残存
- **一部 Magic numbers**: タイムアウト値等がハードコード
