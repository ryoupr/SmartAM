# 開発ワークフロー

## ブランチ戦略

```
feature/xxx → PR+Copilotレビュー → develop → PR+Copilotレビュー → main + tag
```

- `main`: リリース用。タグはここから打つ
- `develop`: 開発用。feature branch のマージ先
- `feature/*`: 機能開発・バグ修正用。`develop` から切る

## ルール

- **すべてのブランチ間マージは GitHub PR 経由**（ローカル `git merge` 禁止）
- **PR には必ず Copilot レビューをリクエスト**（`--reviewer @copilot`）
- **Copilot 指摘に対応してからマージ**（対応不要なら理由をコメント）
- develop / main ブランチを削除しない

---

## 開発タスクリスト

feature 開発時に以下を順番に実行する。

### Phase 1: 実装

- [ ] `git checkout develop && git pull origin develop`
- [ ] `git checkout -b feature/xxx`
- [ ] 実装
- [ ] `cargo check` + `npm run check` で検証
- [ ] コミット + `git push -u origin feature/xxx`

### Phase 2: PR + Copilot レビュー（feature → develop）

- [ ] `gh pr create --base develop --title "..." --body "..." --reviewer @copilot`
- [ ] Copilot レビュー完了を待つ
- [ ] 指摘事項を確認し、対応が必要なものは修正 → push
- [ ] GitHub 上で PR マージ

### Phase 3: クリーンアップ

- [ ] `git checkout develop && git pull origin develop`
- [ ] `git branch -d feature/xxx`

---

## リリースタスクリスト

リリース時は以下を順番に実行する。詳細コマンドは `release.md` 参照。

### Phase 1: バージョンバンプ

- [ ] `git checkout develop && git pull origin develop`
- [ ] 3ファイルのバージョンを更新（package.json / Cargo.toml / tauri.conf.json）
- [ ] `git commit -m "chore: bump version to X.Y.Z"` + `git push origin develop`

### Phase 2: PR + Copilot レビュー（develop → main）

- [ ] `gh pr create --base main --head develop --title "release: vX.Y.Z" --reviewer @copilot`
- [ ] Copilot レビュー完了を待つ
- [ ] 指摘事項を確認し、対応が必要なものは修正 → push
- [ ] GitHub 上で PR マージ（⚠️ develop を削除しない）
- [ ] `git checkout main && git pull origin main`

### Phase 3: ビルド + リリース

- [ ] `cd app && npx tauri build`
- [ ] `SmartAM.app.tar.gz.sig` が生成されたことを確認
- [ ] latest.json を生成
- [ ] `git tag vX.Y.Z && git push origin vX.Y.Z`
- [ ] `gh release create vX.Y.Z ...` でアセット付き Release 作成

### Phase 4: 同期 + 確認

- [ ] `gh pr create --base develop --head main --title "sync: main → develop after vX.Y.Z"`
- [ ] GitHub 上でマージ（Copilot レビュー不要）
- [ ] `git checkout develop && git pull origin develop`
- [ ] `gh release view vX.Y.Z` で確認
- [ ] `curl latest.json` で自動アップデート配信確認
