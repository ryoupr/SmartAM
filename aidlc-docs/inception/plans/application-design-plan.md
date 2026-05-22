# Application Design Plan

## Plan Checklist

- [ ] コンポーネント分割設計（Frontend）
- [ ] コンポーネント分割設計（Backend）
- [ ] サービスレイヤー設計
- [ ] コンポーネント依存関係定義
- [ ] 設計整合性検証

---

## Design Questions

### Q1: Frontend状態管理パターン

+page.svelteから状態管理を分離する方式：

A) Svelte 5 runes + context API（コンポーネントツリーで共有）
B) 専用のstate store module（store.tsを拡張してドメイン別に分割）
C) 両方組み合わせ（グローバル状態はstore、ローカル状態はcontext）
X) Other (please describe after [Answer]: tag below)

[Answer]: ---

### Q2: Rustエラー型の粒度

A) 1つの統合エラーenum（`AppError`に全バリアント）
B) モジュール別エラーenum（`ImapError`, `AiError`, `CalendarError`等）+ `thiserror`
C) B + フロントエンド向けのシリアライズ可能エラー型を別途定義
X) Other (please describe after [Answer]: tag below)

[Answer]: ---

### Q3: Keychain統合の範囲

A) APIキーのみ（LLMプロバイダーのapi_key）
B) APIキー + OAuthトークン（access_token, refresh_token）
C) 全認証情報（APIキー + OAuthトークン + IMAPパスワード）
X) Other (please describe after [Answer]: tag below)

[Answer]: ---

### Q4: 非同期IMAP実装方式

A) `async-imap` crateに移行（tokioネイティブ）
B) 現行`imap` crateを維持し`spawn_blocking`で包む（現状の改善版）
C) `imap-next` crate（新しい非同期IMAP実装）
X) Other (please describe after [Answer]: tag below)

[Answer]: ---

### Q5: テストフレームワーク選定（Frontend）

A) Vitest + @testing-library/svelte（コンポーネントテスト）
B) Vitest + Playwright（E2Eのみ、コンポーネントテストなし）
C) Vitest + @testing-library/svelte + Playwright（両方）
X) Other (please describe after [Answer]: tag below)

[Answer]: ---
