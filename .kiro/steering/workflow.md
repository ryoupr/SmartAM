# 開発ワークフロー

## ブランチ戦略

```
feature/xxx → PR+AIレビュー → develop → PR → main + tag
```

- `main`: リリース用。タグはここから打つ
- `develop`: 開発用。feature branch のマージ先
- `feature/*`: 機能開発・バグ修正用。`develop` から切る

## ルール

- **すべてのブランチ間マージは GitHub PR 経由**（ローカル `git merge` 禁止）
- **PR には必ず AI サブエージェントによるセキュリティレビューを実施**
- **レビュー指摘と対応ログは PR コメントに残す**（対応不要なら理由をコメント）
- develop / main ブランチを削除しない

## レビュープロセス

PR作成後、以下の手順でレビューを実施する:

1. サブエージェント（`code-reviewer` ロール）にPR差分を渡してセキュリティレビュー
2. Major以上の指摘があれば修正してpush
3. 指摘と対応のログをPRコメントに記録
4. 全Major指摘を解消後にマージ

---

## 開発タスクリスト

feature 開発時に以下を順番に実行する。

### Phase 1: 実装

- [ ] `git checkout develop && git pull origin develop`
- [ ] `git checkout -b feature/xxx`
- [ ] 実装
- [ ] `cargo check` + `npm run check` で検証
- [ ] コミット + `git push -u origin feature/xxx`

### Phase 2: PR + AI レビュー（feature → develop）

- [ ] `gh pr create --base develop --title "..." --body "..."`
- [ ] AI サブエージェントでセキュリティレビュー実施
- [ ] Major 指摘に対応 → push
- [ ] PR コメントにレビューログ記録
- [ ] GitHub 上で PR マージ

### Phase 3: クリーンアップ

- [ ] `git checkout develop && git pull origin develop`
- [ ] `git branch -d feature/xxx`

---

## リリースタスクリスト

リリース時は以下を順番に実行する。詳細コマンドは `release.md` 参照。
Kiro に `リリース` と伝えれば自動実行可能。

### Phase 1: バージョンバンプ

- [ ] `git checkout develop && git pull origin develop`
- [ ] 3ファイルのバージョンを更新（package.json / Cargo.toml / tauri.conf.json）
- [ ] `git commit -m "chore: bump version to X.Y.Z"` + `git push origin develop`

### Phase 2: PR（develop → main）

- [ ] `gh pr create --base main --head develop --title "release: vX.Y.Z"`
- [ ] PR コメントに「feature PR でレビュー済み」を記録
- [ ] GitHub 上で PR マージ（⚠️ develop を削除しない）
- [ ] `git checkout main && git pull origin main`

### Phase 3: ビルド + リリース

- [ ] `cd app && npx tauri build`（※mise壊れ時は下記トラブルシューティング参照）
- [ ] `SmartAM.app.tar.gz.sig` が生成されたことを確認
- [ ] latest.json を生成
- [ ] `git tag vX.Y.Z && git push origin vX.Y.Z`
- [ ] `gh release create vX.Y.Z ...` でアセット付き Release 作成

### Phase 4: 同期 + 確認

- [ ] `gh pr create --base develop --head main --title "sync: main → develop after vX.Y.Z"`
- [ ] GitHub 上でマージ
- [ ] `git checkout develop && git pull origin develop`
- [ ] `gh release view vX.Y.Z` で確認
- [ ] `curl latest.json` で自動アップデート配信確認
