# Code Generation Plan — Unit 2+3: Frontend/Backend リファクタリング

## Unit 2: Frontend リファクタリング

### Step 1: Store分割 — stores/mail.ts
- [ ] `app/src/lib/stores/mail.ts` 作成
- メール状態（一覧・選択・フォルダ・検索・同期）を+page.svelteから抽出

### Step 2: Store分割 — stores/settings.ts
- [ ] `app/src/lib/stores/settings.ts` 作成
- 既存store.tsからsettings load/save/getterを移動

### Step 3: Store分割 — stores/ui.ts
- [ ] `app/src/lib/stores/ui.ts` 作成
- トースト・モーダル・コンポーズ状態

### Step 4: Store分割 — stores/ai.ts
- [ ] `app/src/lib/stores/ai.ts` 作成
- AI機能状態（パネル開閉は既にMailDetail内なので最小限）

### Step 5: ShortcutManager抽出
- [ ] `app/src/lib/components/ShortcutManager.svelte` 作成
- +page.svelteからキーボードショートカットロジックを抽出

### Step 6: ToastNotification抽出
- [ ] `app/src/lib/components/ToastNotification.svelte` 作成

### Step 7: Settings.svelte タブ分割
- [ ] `app/src/lib/components/settings/AccountTab.svelte` 作成
- [ ] `app/src/lib/components/settings/LlmTab.svelte` 作成
- [ ] `app/src/lib/components/settings/UsageTab.svelte` 作成
- [ ] `app/src/lib/components/settings/ShortcutTab.svelte` 作成
- [ ] `app/src/lib/components/settings/NotificationTab.svelte` 作成
- [ ] `app/src/lib/components/settings/CalendarTab.svelte` 作成
- [ ] `Settings.svelte` をタブルーター化（各タブコンポーネントを読み込むだけ）

### Step 8: +page.svelte スリム化
- [ ] storeインポートに切り替え、直接ロジックを削除
- [ ] 目標: 200行以下

---

## Unit 3: Backend リファクタリング

### Step 9: エラー型定義
- [ ] `app/src-tauri/src/error.rs` 作成（FrontendError + モジュール別enum）

### Step 10: imap_client.rsにエラー型適用
- [ ] `Result<T, String>` → `Result<T, ImapError>` に変更

### Step 11: ai_client.rsにエラー型適用 + Dead Code除去
- [ ] `Result<T, String>` → `Result<T, AiError>` に変更
- [ ] localhost:4000参照を除去

### Step 12: lib.rsのコマンドでFrontendError変換
- [ ] 各コマンドの戻り値を`Result<T, FrontendError>`に統一

### Step 13: store.ts getLlmConfig修正
- [ ] localhost:4000参照を除去し、直接API呼び出しに修正

---

## Files Modified/Created Summary
- **Created (Unit 2)**: stores/mail.ts, stores/settings.ts, stores/ui.ts, stores/ai.ts, ShortcutManager.svelte, ToastNotification.svelte, settings/6タブ
- **Modified (Unit 2)**: +page.svelte, Settings.svelte, store.ts
- **Created (Unit 3)**: error.rs
- **Modified (Unit 3)**: lib.rs, imap_client.rs, ai_client.rs, store.ts
