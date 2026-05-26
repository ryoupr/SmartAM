# リリースフロー

## 前提条件

`.env` に以下が設定されていること（`mise.toml` の `[env] _.file = ".env"` で自動読み込み）:

- `TAURI_SIGNING_PRIVATE_KEY` — base64 エンコードされた秘密鍵
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — 秘密鍵のパスワード

## バージョン管理

以下の3ファイルを同時に更新する（1つでも漏れるとビルドバージョンが不整合になる）:

- `app/package.json`
- `app/src-tauri/Cargo.toml`
- `app/src-tauri/tauri.conf.json`

コミットメッセージ: `chore: bump version to X.Y.Z`

## 手順

### 1. バージョンバンプ + develop → main マージ

```bash
# develop で3ファイルのバージョンを更新
git checkout develop && git pull origin develop
# ... 3ファイル編集 ...
git add -A && git commit -m "chore: bump version to X.Y.Z"
git push origin develop

# main にマージ（⚠️ develop を削除しない）
git checkout main && git pull origin main
git merge develop --no-edit && git push origin main
```

### 2. ビルド（署名付き）

```bash
cd app && npx tauri build
```

成功時の出力に以下が含まれることを確認:
```
Finished 1 updater signature at:
    .../SmartAM.app.tar.gz.sig
```

⚠️ これが出ない場合、`TAURI_SIGNING_PRIVATE_KEY` が未設定。`.env` と `mise env | grep TAURI` を確認。

生成物:
- `target/release/bundle/dmg/SmartAM_X.Y.Z_aarch64.dmg`
- `target/release/bundle/macos/SmartAM.app.tar.gz`
- `target/release/bundle/macos/SmartAM.app.tar.gz.sig`

### 3. latest.json 生成

```bash
cd /path/to/SmartAM
python3 -c "
import json, datetime
sig = open('app/src-tauri/target/release/bundle/macos/SmartAM.app.tar.gz.sig').read().strip()
json.dump({
  'version': 'X.Y.Z',
  'notes': 'リリースノート',
  'pub_date': datetime.datetime.now(datetime.UTC).strftime('%Y-%m-%dT%H:%M:%SZ'),
  'platforms': {'darwin-aarch64': {
    'signature': sig,
    'url': 'https://github.com/ryoupr/SmartAM/releases/download/vX.Y.Z/SmartAM.app.tar.gz'
  }}
}, open('/tmp/latest.json','w'), indent=2, ensure_ascii=False)
"
```

### 4. タグ + GitHub Release

```bash
git tag vX.Y.Z && git push origin vX.Y.Z

gh release create vX.Y.Z \
  app/src-tauri/target/release/bundle/dmg/SmartAM_X.Y.Z_aarch64.dmg \
  app/src-tauri/target/release/bundle/macos/SmartAM.app.tar.gz \
  app/src-tauri/target/release/bundle/macos/SmartAM.app.tar.gz.sig \
  /tmp/latest.json \
  --title "vX.Y.Z" --target main --notes "リリースノート"
```

### 5. develop を main に同期

```bash
git checkout develop && git merge main --no-edit && git push origin develop
```

### 6. 確認

```bash
gh release view vX.Y.Z
curl -sL https://github.com/ryoupr/SmartAM/releases/latest/download/latest.json | python3 -m json.tool
```

## よくあるトラブル

| 症状 | 原因 | 対処 |
|------|------|------|
| `A public key has been found, but no private key` | `TAURI_SIGNING_PRIVATE_KEY` 未設定 | `.env` を確認、`mise env \| grep TAURI` で検証 |
| `--private-key cannot be used with --private-key-path` | `TAURI_SIGNING_PRIVATE_KEY_PATH` が残っている | `unset TAURI_SIGNING_PRIVATE_KEY_PATH` |
| アプリのアップデートボタンが反応しない | `latest.json` の signature が不正、または `downloadAndInstall` の例外 | `latest.json` の内容を curl で確認、アプリログを確認 |
