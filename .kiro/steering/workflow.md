# 開発ワークフロー

## ブランチ戦略

- `main`: リリース用。タグはここから打つ
- `develop`: 開発用。feature branch のマージ先
- `feature/*`: 機能開発・バグ修正用。`develop` から切る

```
develop → feature/xxx → PR → develop → (リリース時) → main + tag
```

## ブランチ削除ルール

- **feature → develop** の PR マージ時: feature ブランチを削除して OK
- **develop → main** マージ時: **develop を絶対に削除しない**

## Issue 対応フロー

1. `git checkout develop && git pull origin develop`
2. `git checkout -b feature/xxx`（worktree 使用時は `git worktree add`）
3. 実装・検証（`cargo check` + `npm run check`）
4. commit → push → PR（base: `develop`）→ マージ
5. feature ブランチを削除
6. `git checkout develop && git pull origin develop`
