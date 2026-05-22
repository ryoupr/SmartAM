# Code Generation Plan — Unit 1: HTMLリンクバグ修正

## Context
- **Unit**: Unit 1 (バグ修正)
- **Requirement**: FR-1 — HTMLメール内リンクをクリックしたらOSデフォルトブラウザで開く
- **Current State**: 全HTMLメールでリンクが反応しない
- **Target File**: `app/src/lib/components/MailDetail.svelte`

## Root Cause Analysis

iframe内のクリックハンドラscript（`buildSrcdoc`で注入）が正しく動作していない。
考えられる原因:
1. メールHTMLが完全なドキュメント構造（`<html><head>...</head><body>...</body></html>`）を持ち、srcdocの外側構造と入れ子になる
2. メールHTML内のCSPメタタグが自前のscriptをブロックする可能性
3. `sanitizeHtml`がメール内の`<script>`を除去する際に、自前scriptの`<\/script>`エスケープと干渉する可能性

## Plan Steps

- [x] Step 1: `sanitizeHtml`を改善 — メールHTMLから`<html>`, `<head>`, `<body>`タグ（開始・終了両方）を除去し、body内コンテンツのみ抽出する
- [x] Step 2: メール内のCSP `<meta>`タグも除去する（自前CSPと競合防止）
- [x] Step 3: `buildSrcdoc`のクリックハンドラを堅牢化 — `DOMContentLoaded`後に登録するよう変更
- [x] Step 4: プレーンテキスト表示のリンク（`{@html linkifyText(...)}`）にもクリックハンドラを追加（念のため）
- [x] Step 5: 動作確認用のコメントを追加

## Files to Modify
- `app/src/lib/components/MailDetail.svelte` — sanitizeHtml, buildSrcdoc関数の修正
