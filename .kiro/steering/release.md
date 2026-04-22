# リリースワークフロー

## バージョン管理

以下の 3 ファイルを同時に更新する:

- `app/package.json`
- `app/src-tauri/Cargo.toml`
- `app/src-tauri/tauri.conf.json`

コミットメッセージ: `chore: bump version to {version}`

## 前提条件

- Ed25519 署名キー（`updater-key.pem`）が手元にあること
- `gh` CLI がインストール済みであること

## リリースフロー

### 1. develop を最新化

```bash
git checkout develop && git pull origin develop
```

### 2. 未マージの PR をすべてマージ

```bash
gh pr list --state open --json number,title
gh pr merge {番号} --merge
```

### 3. バージョンバンプ（3 ファイル同時更新）

```bash
# app/package.json, app/src-tauri/Cargo.toml, app/src-tauri/tauri.conf.json
# の "version" を更新
git add -A && git commit -m "chore: bump version to {version}"
git push origin develop
```

### 4. develop → main マージ

```bash
git checkout main && git pull origin main
git merge develop -m "release: merge develop into main for v{version}"
git push origin main
```

⚠️ **main マージ後に develop ブランチを絶対に削除しないこと**

### 5. タグ作成

```bash
git tag v{version} && git push origin v{version}
```
### 6. 署名付きビルド

```bash
# 署名キーを環境変数に設定
export TAURI_SIGNING_PRIVATE_KEY=$(cat /path/to/updater-key.pem)
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""

# プロダクションビルド
cd app && npx tauri build
```

ビルド成果物（`app/src-tauri/target/release/bundle/`）:

| ファイル | 用途 |
|---|---|
| `macos/SmartAM.app` | アプリ本体 |
| `dmg/SmartAM_{version}_aarch64.dmg` | 新規インストール用 DMG |
| `macos/SmartAM.app.tar.gz` | アップデーター用アーカイブ |
| `macos/SmartAM.app.tar.gz.sig` | アーカイブの Ed25519 署名 |

### 7. GitHub Release 作成 & アセットアップロード

```bash
cd app/src-tauri/target/release/bundle

# Release 作成（DMG を添付）
gh release create v{version} \
  dmg/SmartAM_{version}_aarch64.dmg \
  --title "v{version}" \
  --notes "## Changes
- ...（変更内容を記載）"

# アップデーター用アセットを追加アップロード
gh release upload v{version} \
  macos/SmartAM.app.tar.gz \
  macos/SmartAM.app.tar.gz.sig
```

### 8. latest.json を作成してアップロード

```bash
# latest.json を生成
cat > /tmp/latest.json << 'EOF'
{
  "version": "{version}",
  "platforms": {
    "darwin-aarch64": {
      "url": "https://github.com/ryoupr/SmartAM/releases/download/v{version}/SmartAM.app.tar.gz",
      "signature": "$(cat macos/SmartAM.app.tar.gz.sig)"
    }
  }
}
EOF

# アップロード
gh release upload v{version} /tmp/latest.json
```

### 9. 確認

```bash
gh release view v{version}
```

### 10. develop に戻る

```bash
git checkout develop
```

## ブランチ削除ルール

- **feature → develop** の PR マージ時: feature ブランチを削除して OK
- **develop → main** のマージ時: **develop ブランチを絶対に削除しない**

## 署名キーの管理

- 秘密鍵 `updater-key.pem` は `.gitignore` で除外済み
- 紛失するとアップデート署名ができなくなる — 安全な場所にバックアップすること
- 公開鍵は `tauri.conf.json` の `plugins.updater.pubkey` に埋め込み済み

## アップデーターの動作

1. アプリ起動時に `https://github.com/ryoupr/SmartAM/releases/latest/download/latest.json` を取得
2. `latest.json` のバージョンと現在のバージョンを比較
3. 新バージョンがあれば画面上部に通知バーを表示
4. ユーザーが「アップデート」をクリック → ダウンロード → 署名検証 → インストール → 再起動
