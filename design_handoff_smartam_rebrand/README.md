# Handoff: SmartAM Rebrand — Logo, Design System, App UI

このパッケージは SmartAM のブランドリニューアル一式のデザインリファレンスです。  
ロゴ「智」（アウトライン + 赤バー）を起点に、デザイントークン、コンポーネント、アプリ UI モック、バナーまでをカバーしています。

---

## ⚠ About the Design Files

このバンドルに含まれる HTML ファイルは **デザインリファレンス（プロトタイプ）** です。
本番コードとしてそのまま投入するものではありません。

タスクは、これらの HTML デザインを **SmartAM の既存コードベース** （SvelteKit + Svelte 5 runes + Tauri v2、`app/` 配下）の中で再現することです。既存の構成・コンポーネント・状態管理の流儀に合わせて再実装してください。

具体的な置き換え対象：

- `app/src/lib/components/*.svelte` の既存 UI 部品
- `app/src/lib/assets/favicon.svg`（初期 Svelte テンプレ）
- `app/src-tauri/icons/` 配下の各サイズアイコン
- `app/src/app.css` の CSS 変数（Catppuccin Mocha のみ）

---

## Fidelity

**High-fidelity (hifi)** です。配色・タイポグラフィ・余白・状態を含む詳細な仕様を提供しています。  
ピクセルパーフェクトに再現してください。

---

## Files

| ファイル | 内容 |
|---|---|
| `01-design-system.html` | デザイントークン、タイポグラフィ、コンポーネント、ライティング規約 |
| `02-logo-deliverables.html` | 智ロゴ最終形：アプリアイコン全サイズ、ワードマーク、設置ガイド、PNG ダウンロードボタン |
| `03-app-ui-sample.html` | 3 ペインメールクライアントの完全モック（Light Theme） |
| `04-button-proposal.html` | 12 個の SVG 線アイコン + ラベル付きボタン仕様 |
| `05-banners.html` | 8 種類のバナーレイアウト（GitHub README / X header / Square social / Email signature 等） |
| `assets/smartam-logo.svg` | 主ロゴ（アウトライン形態、320×320） |
| `assets/smartam-logo-filled.svg` | 塗り形態（小サイズ用） |
| `assets/favicon.svg` | favicon（32×32、塗り + クリーム背景） |

---

## Design Tokens

### Brand · Light (Web / Docs / Marketing)

| トークン | 値 | 役割 |
|---|---|---|
| `--cream` | `#F6EFE4` | キャンバス背景 |
| `--paper` | `#FAF5E7` | サブパネル / カード |
| `--bone` | `#ECE5D5` | ホバー / プレス背景 |
| `--paperWh` | `#FFFAF0` | インプット / 浮き要素 |
| `--ink` | `#1A1410` | 主テキスト / ロゴ |
| `--ink-80` | `#1A1410CC` | 本文 |
| `--ink-60` | `#1A141099` | 二次テキスト |
| `--ink-40` | `#1A141066` | 三次 / ヒント |
| `--ink-20` | `#1A141033` | ボーダー強 |
| `--line` | `rgba(26,20,16,0.10)` | ボーダー弱 |
| `--line-2` | `rgba(26,20,16,0.18)` | ボタンボーダー |
| `--red` | `#C8281E` | ブランド赤 / プライマリ |
| `--red-deep` | `#A02019` | プライマリ hover |

### App · Dark (Catppuccin Mocha 継承)

既存 `app/src/app.css` を踏襲。`--red` のみブランド赤 `#C8281E` で上書き。Catppuccin の `#F38BA8` は破壊操作（削除）用に保持。

```
--base: #1E1E2E    --mantle: #181825    --crust: #11111B
--surface0: #313244 --surface1: #45475A
--text: #CDD6F4    --subtext: #A6ADC8    --overlay: #6C7086
```

### AI 機能色（Light）

ライト背景でも視認性を持たせるため、Catppuccin のパステルから中濃度版に再調整：

| 機能 | カラー | 背景 (tint) | 用途 |
|---|---|---|---|
| 要約 | `#2F7A3B` | `#E3F1DE` | Summary panel |
| 下書き | `#A06F0A` | `#FAECCA` | Draft / Star |
| 翻訳 | `#1D5FB3` | `#DDE9F9` | Translate |
| カレンダー | `#1D5FB3` | `#DDE9F9` | Calendar |
| 再生成 | `#7C3EB5` | `#ECDCF9` | Regen ボタン |

### AI 機能色（Dark）

既存 Catppuccin Mocha のままに保持：

```
--green:  #A6E3A1  --yellow: #F9E2AF
--blue:   #89B4FA  --mauve:  #CBA6F7
```

### Spacing Scale (4px grid)

```
space-1   4px      space-2   8px       space-3   12px
space-4  16px      space-6  24px       space-8   32px
space-12 48px      space-16 64px
```

### Border Radius

```
xs        3px   (small chips)
sm        6px   (buttons / inputs)
md       10px   (cards)
lg       16px   (modals)
squircle 22.37% (macOS app icon)
```

### Shadows

```css
subtle:   0 1px 3px  rgba(26,20,16,0.08)
card:     0 4px 12px rgba(26,20,16,0.10)
floating: 0 16px 36px rgba(26,20,16,0.18)
```

### Typography

- **Latin**: Inter (400/500/600/700/800)
- **日本語**: Noto Sans JP (400/500/700/900)
- **Mono**: JetBrains Mono (400/500/600)

| Token | Size | Weight | Letter Spacing | 用途 |
|---|---|---|---|---|
| Display | 56–64px | 800 | -0.03em | バナー H1 |
| H1 | 38–48px | 800 | -0.025em | ページタイトル |
| H2 | 26–32px | 700 | -0.02em | セクション |
| H3 | 20–24px | 700 | -0.015em | サブセクション |
| Title | 15–18px | 600 | -0.01em | 件名 / カードタイトル |
| Body | 13–14px | 400 | 0 | 本文 |
| Small | 11–12px | 400–500 | 0 | メタ情報 |
| Mono | 10–12px | 500 | 0.05–0.15em | タグ / コード |

---

## Logo Usage

### Primary mark (智 · Outline + Red Bar)

- **Character**: 智 (CHI · ちえ／さとし) — wisdom / smart
- **Typeface**: Noto Sans JP · Black 900
- **Letter-spacing**: -0.05em
- **Stroke**: font-size の 2.5%、color `--ink` (#1A1410)
- **Bar**: 智の字面幅の 90%、高さ 10% of font-size、`--red` (#C8281E)
- **Clear space**: 4ch / 全周
- **最小サイズ**: 16px（≤32px は塗り版に切替必須）

### Do

- アウトラインの太さ・字間・バー比率を維持
- ダーク背景上ではストロークを `--text` (#CDD6F4) に変更可

### Don't

- 塗り潰しを標準形として使わない（≤32px のみ）
- 赤を背景色にしない（バーが消える）
- 傾けたり変形しない（回転・スキュー禁止）
- 色を勝手に変更しない

### Wordmark lockup

```
[智 + bar]  Smart<span style="color:#C8281E">AM</span>
```

- 「AM」のみ赤色で強調
- 智マーク高さ : ワードマーク高さ = 1.2 : 1
- 横並び gap: ワードマーク x-height の 0.7 倍

---

## Screens / Views

### Screen 1: Mail Client (Light Theme)

参照: `03-app-ui-sample.html`

#### Layout

- 固定 1440 × 856（ウィンドウ枠 12px ラジアス、`shadow-floating`）
- グリッド: `36px (titlebar) / 1fr (panes)`
- ペインカラム: `240px (sidebar) | 380px (mail list) | 1fr (detail)`
- min-width: 1504px、横スクロール許容

#### Title bar (36px)

- 背景 `--paper`、下境界 `--line`
- 左：macOS Traffic lights (赤/黄/緑、12×12、6px gap)
- 中央：智 mini-mark (16px) + bar + "SmartAM" + 「— 受信トレイ」

#### Sidebar (240px)

1. **Head mark**: 智 outline + bar + "SmartAM" wordmark + "v0.1.0"
2. **Account chips** (2 個): `--paperWh` 背景 + 同期状態ドット (`--ai-green` / `--ink-40`)
3. **Section: FOLDERS** — `JetBrains Mono` 9px / 0.15em letter-spacing
4. **Nav items**: 7px 10px padding、5px radius
   - Default: `--ink-80` text、hover `--bone` 背景
   - Active: `--ink` 塗り背景 + `--cream` text + 赤カウント
5. **Section: LABELS** — 色付きドット（赤=緊急、青=仕事、緑=個人）
6. **LLM badge** (footer): `--paperWh` 背景 + ボーダー、モデル名 + 同期ドット (`--ai-blue`)

#### Mail list (380px)

- **Header**: タイトル "受信トレイ" (17px / 700)、カウント "24 件 · 新着 5"、↻ 更新ボタン
- **Search**: `--paperWh` 背景 + ボーダー、focus `--red` ボーダー、/ kbd shortcut hint
- **Mail item rows**:
  - 12px 16px padding、左 3px 境界
  - Unread: 左 border `--red`、件名 weight 500
  - Active: `--paperWh` 背景 + ink 内側 border + 赤左 border
  - Hover: `--paper` 背景
  - Row 内: From (13px/600 truncate) + Star + Time (mono 10px) / Subject / Preview (2行 clamp) / Badges
  - Badges: 添付 (`--ai-yellow-bg`)、要約済み (`--ai-green-bg`)

#### Mail detail (1fr)

- **Toolbar** (12px 24px padding、`--paper` 背景、下境界 `--line`):
  - 返信 / 転送 / divider / アーカイブ / 削除 / スター / divider / 要約 / 下書き / 翻訳 / カレンダー / spacer / 1/24 カウンタ
  - すべてラベル + 線アイコンボタン（詳細は Component 仕様参照）
- **Mail head**: アバター 46px (グラデ `linear-gradient(135deg, #c8281e, #7c3eb5)`)、件名 H1、From/To/メタ、スターアイコン
- **AI Summary panel**:
  - 背景 `--ai-green-bg`、ボーダー `--ai-green`、10px radius
  - Head: 緑アイコン + "AI 要約" + model 名 + 再生成ボタン (`--ai-mauve`)
  - Bullet list: `━` プレフィックス（mono 10px、緑）
  - Actions: コピー / Markdown でコピー / 閉じる（mono、`--paperWh` チップ）
- **Body text**: max-width 680px、14.5px / line-height 1.8
  - `<strong>` は yellow tint 背景でハイライト
  - 署名: 上罫線 dashed、`--ink-60` 12px
- **Attachments**: `--paperWh` カード、PDF (`--ai-blue` chip)、FIG (`--ai-mauve` chip)、サイズ表示
- **Toast** (右下、`absolute` 配置):
  - `--ink` 黒背景 + `--cream` text
  - 左 4px border `--ai-blue`
  - "1 件アーカイブしました" + "取り消す" ボタン

#### Page Header (window 上部)

- 6px 赤上罫線
- 智 mark (56px) + "SmartAM · App UI Sample"
- 右に "DESIGN SYSTEM APPLIED · LIGHT THEME" (mono)

---

## Components Spec

### Buttons (詳細: `04-button-proposal.html`)

**重要**: 既存仕様で UI ボタンに使われている絵文字 (📝 ✍ 🌐 📅 など) は **すべて廃止**。1.5px ストローク線アイコン + 日本語ラベルに置き換える。絵文字はメール本文と通知本文では引き続き使用可。

#### Default Button

```
font: Inter 13px / 500 / -0.005em
padding: 8px 12px
border-radius: 6px
border: 1.5px solid --line-2
background: --paperWh
color: --ink
gap: 8px (icon ↔ label ↔ kbd)
hover: background --bone, border --ink-40
```

#### Variants

| Class | Border | Color | Active |
|---|---|---|---|
| `.btn` | `--line-2` | `--ink` | — |
| `.btn.primary` | `--ink` | `--cream` on `--ink` | hover → `--red` 背景 |
| `.btn.danger` | `rgba(red, 0.30)` | `--red` | hover → `--red` 塗り |
| `.btn.starred` | `rgba(yellow, 0.40)` | `--ai-yellow` | star 状態固定 |
| `.btn.ai-summary` | `rgba(green, 0.35)` | `--ai-green` | active → `--ai-green` 塗り + cream |
| `.btn.ai-draft` | `rgba(yellow, 0.35)` | `--ai-yellow` | hover → yellow-bg |
| `.btn.ai-translate` | `rgba(blue, 0.35)` | `--ai-blue` | hover → blue-bg |
| `.btn.ai-calendar` | `rgba(blue, 0.35)` | `--ai-blue` | hover → blue-bg |
| `.btn.ai-regen` | `rgba(mauve, 0.35)` | `--ai-mauve` | hover → mauve-bg |
| `.btn.ghost` | transparent | `--ink-80` | hover → `--bone` 背景 |

#### Sizes

- **Default**: 13px text / 16px icon / 8px 12px padding
- **Compact**: 12px text / 14px icon / 6px 10px padding
- **Icon-only**: 34×34 正方形（ラベル `display:none`、`title` 属性で tooltip）
- **Auto-collapse**: ペイン幅 ≤ 600px で全 `.btn` を icon-only に縮退

#### `.kbd` (keyboard hint)

```
font: JetBrains Mono 10px
padding: 1px 5px
background: --cream
border: 1px solid --line
border-radius: 3px
color: --ink-60
```

Primary ボタン内では `background: rgba(white, 0.10)` + `border: rgba(white, 0.18)` + cream text。

### Line Icons (`04-button-proposal.html` Section 01)

全 12 アイコン共通仕様：

- **viewBox**: 24×24
- **stroke**: 1.5
- **stroke-linecap**: round
- **stroke-linejoin**: round
- **fill**: none（Star のみ `currentColor`）

| アイコン | 用途 | パスのキーポイント |
|---|---|---|
| Reply | 返信 | 左向き chevron + 折り返し弧 |
| Forward | 転送 | 右向き chevron + 折り返し弧 |
| Archive | アーカイブ | 蓋付き箱 + 横線 |
| Trash | 削除 | ゴミ箱 + 縦線2本 |
| Star | スター | 5 角星（filled） |
| Summary | 要約 | スパーク + 横線 3 本 |
| Draft | 下書き | ペン先 + sparkle |
| **Translate** | 翻訳 | **「A」(Inter 700) + 「あ」(Noto Sans JP 700)** ※他と異なりテキストアイコン |
| Calendar | カレンダー | 枠 + 上線 + 2 つの留め金 + ドット |
| Regen | 再生成 | 円形矢印（時計回り） |
| Send | 送信 | 紙飛行機 |
| Attach | 添付 | クリップ |

**翻訳アイコンだけは SVG ではなく `<span>` でテキスト合成**：

```html
<span class="icon translate-text">
  <span class="latin">A</span>
  <span class="kana">あ</span>
</span>
```

```css
.icon.translate-text {
  display: inline-flex; align-items: baseline; justify-content: center;
  gap: 1px; line-height: 1;
  width: 16px; height: 16px;
  font-weight: 700;
}
.icon.translate-text .latin {
  font-family: 'Inter', sans-serif;
  font-size: 11px; letter-spacing: -0.04em;
}
.icon.translate-text .kana {
  font-family: 'Noto Sans JP', sans-serif;
  font-size: 10px; font-weight: 700; letter-spacing: -0.04em;
}
```

### Mail Row

- 12px 16px 12px 13px padding（左 13px は border 用）
- 左 3px transparent border (unread → `--red` / active → `--red` + 内側 inset `--ink`)
- Hover: `--paper` 背景

### AI Panel (Summary 例)

- 機能ごとに `--ai-{feature}-bg` 背景 + `--ai-{feature}` 1px border + 10px radius
- 18px 22px padding
- Head: 26–28px 角丸 icon + 機能色 title (700/13px) + spacer + model 名 (mono 10px) + 再生成 button (26×26)
- Body: bullet list は `━` プレフィックス (mono 10px、機能色) で `padding-left: 18px; position: relative`
- Actions row: `--paperWh` チップ + 1px line border、mono 10px

### Toast

- 右下 `absolute`、24px 24px offset
- `--ink` 背景、`--cream` text
- 左 4px border = 機能色
- 12px 16px padding、8px radius
- `shadow-floating`
- slide-in animation（20px translateY、300ms ease-out）

---

## Banners (`05-banners.html`)

8 種類のバナーレイアウト：

| ID | 用途 | サイズ |
|---|---|---|
| B1 | GitHub README 標準 | 1280 × 420 |
| B2 | Modern Header (smart crop) | 1280 × 420 |
| B3 | Brutalist Diagonal (releases) | 1280 × 420 |
| B4 | Tall Poster (PH / Pinterest) | 800 × 1200 |
| B5 | Square Social (OG / Insta) | 1200 × 1200 |
| B6 | Wide Cinematic (LP hero) | 1920 × 600 |
| B7 | Dark Mode 反転 | 1280 × 420 |
| B8 | X / Twitter Header | 1500 × 500 |
| B9 | Email Signature | 600 × 120 |

各カードに DOM → SVG `<foreignObject>` → Canvas で PNG export ボタンあり。同じレンダリング ロジックを実装すれば、ブラウザ上で書き出し可能。

---

## Interactions & Behavior

### Mail list

- クリックで Mail Detail にロード（既存挙動踏襲）
- Hover で `--paper` 背景
- Unread → 既読化は既存 IMAP フラグ更新ロジック

### AI Action Buttons

- クリックで対応する AI Panel をトグル表示
- 処理中は `opacity: 0.6 + cursor: wait` (既存仕様)
- ショートカット: `y` 要約 / `d` 下書き / `t` 翻訳 / `l` カレンダー (既存仕様 `+page.svelte`)
- 縮退: ペイン幅 ≤ 600px で `.btn` → `.btn.icon-only` に自動切替

### Toast

- 5 秒で自動消滅（既存仕様）
- 可逆操作は「取り消す」リンクで undo
- 不可逆操作（送信）は確認のみ

### Auto-collapse responsive

サイドバー / メール詳細ペインそれぞれに ResizeObserver を仕込み、幅 ≤ 600px で `.compact-layout` クラスをトグル：

```css
.compact-layout .btn .label { display: none; }
.compact-layout .btn .kbd   { display: none; }
.compact-layout .btn { width: 34px; padding: 8px; justify-content: center; }
```

---

## State Management

既存の Svelte 5 runes パターンを踏襲：

- `$state()` でローカル UI 状態
- `$derived()` で計算プロパティ
- `lib/store.ts` の `loadSettings` / `saveSettings` で永続化
- `lib/types.ts` の `MailSummary` / `MailDetail` などのインターフェース踏襲

新規追加が必要そうな state（既存にあれば再利用）：

- `activeAiPanel: 'summary' | 'draft' | 'translate' | 'calendar' | null`
- `darkMode: boolean`（テーマ切替を実装する場合）
- `compactToolbar: boolean`（自動縮退用）

---

## Assets to Replace

### `app/src-tauri/icons/`

`02-logo-deliverables.html` の「APP ICON」セクションを開き、各サイズの DOWNLOAD PNG ボタンで書き出して下記に上書き：

- `icon.png` (1024×1024)
- `128x128@2x.png` (256×256)
- `128x128.png` (128×128)
- `32x32.png` (32×32)
- 16×16 用の追加生成は不要（macOS は 32 から縮小）

`icon.icns` 再生成は macOS の `iconutil` を使用：

```bash
mkdir icon.iconset
# 各サイズを iconset 内に配置（icon_16x16.png, icon_16x16@2x.png, ... icon_512x512@2x.png）
iconutil -c icns icon.iconset
```

Windows / Microsoft Store 用の `Square*Logo.png` は当面後回しで OK（README にも記載）。

### `app/src/lib/assets/favicon.svg`

現在 Svelte 初期テンプレートが入っている。`assets/favicon.svg` で上書き。

### `app/src/app.css`

既存：

```css
:root {
  --base: #1e1e2e;
  --red: #f38ba8;
  /* ... */
}
```

提案変更：

```css
:root {
  /* Catppuccin Mocha 既存（維持） */
  --base: #1e1e2e;
  --mantle: #181825;
  /* ...省略... */

  /* Brand override */
  --red: #c8281e;  /* 旧 #f38ba8 をブランド赤に */
  --pink: #f38ba8; /* 旧 --red を退避（破壊操作 destructive 用） */
}
```

---

## ⚠ Voice / Writing

UI 内テキストは日本語を主とし、簡潔・中立。AI 機能でも擬人化・煽り表現を避ける。

| Do | Don't |
|---|---|
| 1 件アーカイブしました。 | AI が気を利かせて 1 件処理してあげました 🎉 |
| 📝 要約する | ✨ あなたのために要約を生成します！ |
| IMAP 接続に失敗しました。設定 → アカウントを確認してください。 | プロトコルでサーバーに接続を試みた結果、ハンドシェイクで TLS エラーが発生したため通信を確立できませんでした。 |

---

## 実装手順（推奨）

1. **`app/src/app.css` を拡張** — Light 用変数を追加（dark のみだった場合）。`--red` をブランド赤に切替
2. **アセット差し替え** — `favicon.svg` / `app/src-tauri/icons/*.png` を `assets/` のものに置換
3. **ボタンコンポーネント新設** — `lib/components/Button.svelte`（label + icon + kbd プロパティ受け）
4. **アイコン set 新設** — `lib/icons/*.svelte` または単一 `Icon.svelte` (12 アイコン + 翻訳 textual)
5. **既存コンポーネントの絵文字を順次置換**：
   - `MailDetail.svelte` ツールバー
   - `ComposeModal.svelte` フッター
   - `AiPanel.svelte` 再生成・コピーボタン
   - `Sidebar.svelte` ヘッダー mark（智 + bar）
6. **テーマ切替** — `settings.json` に `theme: 'light' | 'dark'` を追加、`<html>` の data 属性切替
7. **README 更新** — `02-logo-deliverables.html` のバナーで生成した PNG を README 先頭に配置

---

## Open Questions

- ライトテーマを設定で出すのか、ダーク維持＋ブランドのみ Web/README で使うのか
- アプリ起動時のスプラッシュにロゴを使うか（Tauri Splashscreen 対応）
- Windows / Linux 対応（Phase 3）時のアイコン再生成方針

---

## Reference Files

開発前にまず `01-design-system.html` と `03-app-ui-sample.html` をブラウザで開いて全体感を掴んでください。

`02-logo-deliverables.html` は実 PNG を書き出すための作業ページです（DOWNLOAD ボタンで Canvas → PNG）。
