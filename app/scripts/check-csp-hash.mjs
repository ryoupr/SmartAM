// ビルド前ガード: MailDetail.svelte の MAIL_BRIDGE_JS（メール本文 srcdoc iframe に注入する
// 高さ計測 + リンク仲介スクリプト）の SHA-256 が tauri.conf.json の script-src に登録されているか検証する。
//
// 背景: 本番ビルドでは srcdoc が Tauri のメイン CSP を継承し、Tauri が script-src に nonce を注入して
// 'unsafe-inline' が無効化される。そのため bridge は 'sha256-...' で許可している。MAIL_BRIDGE_JS を
// 変更してハッシュ更新を忘れると、本番のみ無言で bridge がブロックされメール本文の高さが伸びなくなる
// （tauri dev では再現しない）。このガードがビルドを止めて漏れを検出する。
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

const mailDetailPath = fileURLToPath(new URL('../src/lib/components/MailDetail.svelte', import.meta.url));
const tauriConfPath = fileURLToPath(new URL('../src-tauri/tauri.conf.json', import.meta.url));

const src = readFileSync(mailDetailPath, 'utf8');
const m = src.match(/const MAIL_BRIDGE_JS\s*=\s*`([^`]*)`/);
if (!m) {
  console.error('[csp-hash] MAIL_BRIDGE_JS が MailDetail.svelte に見つかりません');
  process.exit(1);
}

const hash = 'sha256-' + createHash('sha256').update(m[1], 'utf8').digest('base64');
const csp = JSON.parse(readFileSync(tauriConfPath, 'utf8'))?.app?.security?.csp ?? '';

if (!csp.includes(hash)) {
  console.error(`[csp-hash] NG: tauri.conf.json の script-src に ${hash} がありません。`);
  console.error('[csp-hash] MAIL_BRIDGE_JS を変更したら SHA-256 を再計算して script-src の sha256- を更新してください。');
  process.exit(1);
}

console.log(`[csp-hash] OK: ${hash} は script-src に登録済み`);
