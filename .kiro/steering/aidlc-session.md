# AI-DLC セッション中の作業ルール

## 適用条件

AI-DLC ワークフローが開始されたセッション（aidlc-docs/ が存在する状態）では、
本ルールがすべての作業に適用される。

## 原則

**AI-DLC セッション中のすべての作業は、規模に関わらずイテレーションとして記録する。**

## 作業開始前の必須手順

コード修正・feature branch 作成の **前に** 以下を実行する:

1. `aidlc-state.md` に新イテレーションのヘッダを追記
   - Start Date, Scope, Depth（Minimal / Standard / Deep）
2. Depth に応じたステージを列挙

## 作業完了後の必須手順

1. `aidlc-state.md` のステージ進捗を更新（チェックマーク）
2. `audit.md` にログを追記（User Request, Changes, Release）

## Minimal depth の最低限

バグ修正・軽微な改善でも以下は必須:
- aidlc-state.md: イテレーションヘッダ + Stage Progress
- audit.md: User Request + Changes
- Process Deviations: スキップした理由を記録
