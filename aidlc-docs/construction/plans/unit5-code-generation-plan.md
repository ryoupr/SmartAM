# Code Generation Plan — Unit 5: パフォーマンス

## Part A: 仮想スクロール（MailList.svelte）

- [x] Step 1: MailList.svelteに仮想スクロール実装

## Part B: async-imap移行（imap_client.rs）— 次イテレーションに延期

- [ ] Step 2: Cargo.tomlの依存関係変更 — **延期**（テスト基盤構築後に実施）
- [ ] Step 3: imap_client.rs全面書き換え — **延期**
- [ ] Step 4: lib.rsのコマンド呼び出し修正 — **延期**

**延期理由**: 32KBの全面書き換えはテスト不在の状態では回帰リスクが高い。テストコード記述後に実施。

- [x] Step 5: ビルド確認
