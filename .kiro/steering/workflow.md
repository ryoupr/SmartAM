# 開発ワークフロー

## ブランチ戦略

- `main`: リリース用。タグはここから打つ
- `develop`: 開発用。feature branch のマージ先
- `feature/*`: 機能開発・バグ修正用。`develop` から切る

```
feature/xxx → PR → develop → PR → main + tag
```

## マージルール（厳守）

- **すべてのブランチ間マージは GitHub PR 経由で行う**
- ローカルでの `git merge` は禁止（fast-forward 同期含む）
- PR 作成には `gh pr create` を使用する
- **PR 作成後、Copilot のコードレビューを必ずリクエストする**
- Copilot レビュー結果の指摘に対応してからマージする（対応不要と判断したものはコメントで理由を残す）

| マージ方向 | 方法 | base | head |
|-----------|------|------|------|
| feature → develop | `gh pr create --base develop --reviewer @copilot` | develop | feature/xxx |
| develop → main | `gh pr create --base main --head develop --reviewer @copilot` | main | develop |
| main → develop (リリース後同期) | `gh pr create --base develop --head main` | develop | main |

※ main → develop の同期 PR は自動マージで OK（Copilot レビュー不要）

## ブランチ削除ルール

- **feature → develop** の PR マージ時: feature ブランチを削除して OK
- **develop → main** マージ時: **develop を絶対に削除しない**
- **main → develop** 同期 PR マージ時: main は削除しない（当然）

## Issue 対応フロー

1. `git checkout develop && git pull origin develop`
2. `git checkout -b feature/xxx`（worktree 使用時は `git worktree add`）
3. 実装・検証（`cargo check` + `npm run check`）
4. `git push -u origin feature/xxx`
5. `gh pr create --base develop --title "..." --body "..." --reviewer @copilot`
6. Copilot レビュー結果を確認し、対応が必要な指摘は修正 → push
7. PR マージ後: `git checkout develop && git pull origin develop`
8. feature ブランチを削除: `git branch -d feature/xxx`

## ⚠ 必須ルール

- **git 操作（commit, merge, push, tag）を行う前に、必ず本ファイルと `release.md` を参照すること**
- ローカルで `main` や `develop` に直接マージしない — 必ず PR 経由
- リリース時は `release.md` の手順を厳密に順守する（署名付きビルド + アセット添付）
