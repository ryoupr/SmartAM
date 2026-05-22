# Requirements Verification Questions

## Intent Analysis

- **User Request**: SmartAMを全方向に強化したい + HTMLメール内リンクが外部ブラウザで開かないバグ修正
- **Request Type**: Enhancement + Bug Fix
- **Scope**: System-wide
- **Complexity**: Complex（全方位改善）

---

## Q1: 改善の優先順位

「全方向に強化」とのことですが、限られたリソースで最大効果を出すために優先順位を教えてください。

A) バグ修正（リンク問題等）→ コード品質（テスト・リファクタ）→ 新機能
B) バグ修正 → 新機能（Phase 2機能追加）→ コード品質
C) コード品質（テスト・リファクタ・CI）→ バグ修正 → 新機能
D) 全て同時並行で進める（ユニット分割して並列開発）
X) Other (please describe after [Answer]: tag below)

[Answer]: D

---

## Q2: Phase 2機能のスコープ

要件定義書のPhase 2機能をこの改善に含めますか？

A) 含める（優先度分類・Jira連携・Reply All・UI改善全て）
B) 一部含める（UI改善・Reply Allのみ。Jira連携は後回し）
C) 含めない（既存機能の品質向上とバグ修正に集中）
X) Other (please describe after [Answer]: tag below)

[Answer]: C

---

## Q3: コード品質の目標レベル

A) 最低限（主要パスのユニットテスト + CI基盤構築）
B) 標準（テストカバレッジ60%+ 、リファクタリング、CI/CD、lint）
C) 高品質（テストカバレッジ80%+、E2Eテスト、セキュリティ監査、パフォーマンスベンチマーク）
X) Other (please describe after [Answer]: tag below)

[Answer]: C

---

## Q4: God Component問題への対処

+page.svelte(32KB)とSettings.svelte(35KB)が巨大です。リファクタリングしますか？

A) する（コンポーネント分割・状態管理の整理）
B) 部分的にする（最も問題のある箇所のみ）
C) しない（動いているものは触らない）
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Q5: セキュリティ改善

APIキーが現在settings.jsonに平文保存されています。

A) OSキーチェーン（macOS Keychain）に移行する
B) 現状維持（個人利用なので許容）
C) 暗号化して保存（独自実装）
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Q6: パフォーマンス改善

A) 仮想スクロール導入（大量メール対応）+ 非同期IMAP移行
B) 仮想スクロールのみ
C) 現状で十分（体感的に問題ない）
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Q7: 既知バグ — HTMLメール内リンク

リンクが開かない問題について追加情報：

A) 全てのHTMLメールで発生する
B) 特定のメール（ニュースレター等、複雑なHTML）でのみ発生する
C) 不明（再現条件を特定できていない）
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Q8: Security Extension

セキュリティ拡張ルールをこのプロジェクトに適用しますか？

A) Yes — 全SECURITYルールをブロッキング制約として適用（本番品質アプリ向け推奨）
B) No — SECURITYルールをスキップ（PoC・プロトタイプ向け）
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Q9: Property-Based Testing Extension

プロパティベーステスト（PBT）ルールを適用しますか？

A) Yes — 全PBTルールを適用（ビジネスロジック・データ変換・シリアライゼーションがあるプロジェクト向け）
B) Partial — 純粋関数とシリアライゼーションのラウンドトリップのみ適用
C) No — PBTルールをスキップ（シンプルなCRUD・UI中心のプロジェクト向け）
X) Other (please describe after [Answer]: tag below)

[Answer]: C
