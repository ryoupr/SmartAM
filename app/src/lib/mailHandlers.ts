/**
 * Mail operation handlers extracted from +page.svelte.
 * Each function takes the page state as a context object and mutates it.
 */
import { invoke } from '@tauri-apps/api/core';
import { getImapConfig } from '$lib/store';
import { archiveMail, deleteMail, toggleStar, downloadAttachment } from '$lib/mailActions';
import type { MailSummary, MailDetail as MailDetailType } from '$lib/types';
import type { AppSettings } from '$lib/store';

export interface PageState {
  settings: AppSettings;
  mails: MailSummary[];
  selectedMail: MailDetailType | null;
  selectedUid: number | null;
  selectedUids: Set<number>;
  activeFolder: string;
  error: string | null;
  lastUndo: (() => void) | null;
  ensureValidToken: () => Promise<void>;
  showToast: (msg: string, undo?: () => void) => void;
  handleSelect: (uid: number) => Promise<void>;
  setMails: (m: MailSummary[]) => void;
  setSelectedMail: (m: MailDetailType | null) => void;
  setSelectedUid: (uid: number | null) => void;
  setSelectedUids: (s: Set<number>) => void;
  setError: (e: string | null) => void;
  setLastUndo: (fn: (() => void) | null) => void;
  setDetailLoading?: (v: boolean) => void;
}

export async function doArchive(ctx: PageState): Promise<void> {
  const a = ctx.settings.accounts[ctx.settings.activeAccountIndex];
  if (!a || !ctx.selectedUid) return;
  const uid = ctx.selectedUid;
  const idx = ctx.mails.findIndex(m => m.uid === uid);
  const prevMails = ctx.mails;
  ctx.setMails(ctx.mails.filter(m => m.uid !== uid));
  const next = ctx.mails[idx] ?? ctx.mails[idx - 1];
  if (next) ctx.handleSelect(next.uid); else { ctx.setSelectedMail(null); ctx.setSelectedUid(null); }
  let aborted = false;
  ctx.showToast('📦 アーカイブしました', () => { aborted = true; ctx.setMails(prevMails); ctx.handleSelect(uid); });
  archiveMail(a, ctx.activeFolder, uid, 1, () => aborted).catch(e => {
    ctx.setMails(prevMails); ctx.handleSelect(uid); ctx.setError('アーカイブ失敗: ' + String(e));
  });
}

export async function doBulkArchive(ctx: PageState): Promise<void> {
  const a = ctx.settings.accounts[ctx.settings.activeAccountIndex];
  if (!a || ctx.selectedUids.size === 0) return;
  const uids = [...ctx.selectedUids]; const prevMails = ctx.mails; const count = uids.length;
  ctx.setMails(ctx.mails.filter(m => !ctx.selectedUids.has(m.uid)));
  ctx.setSelectedMail(null); ctx.setSelectedUid(null); ctx.setSelectedUids(new Set());
  ctx.showToast(`📦 ${count}件アーカイブしました`, () => { ctx.setMails(prevMails); ctx.setSelectedUids(new Set(uids)); });
  for (const uid of uids) archiveMail(a, ctx.activeFolder, uid, 1, () => false).catch(() => {});
}

export async function doBulkDelete(ctx: PageState): Promise<void> {
  const a = ctx.settings.accounts[ctx.settings.activeAccountIndex];
  if (!a || ctx.selectedUids.size === 0) return;
  const uids = [...ctx.selectedUids]; await ctx.ensureValidToken();
  await Promise.allSettled(uids.map(uid => deleteMail(a, ctx.activeFolder, uid)));
  ctx.setMails(ctx.mails.filter(m => !new Set(uids).has(m.uid)));
  ctx.setSelectedMail(null); ctx.setSelectedUid(null); ctx.setSelectedUids(new Set());
  ctx.showToast(`🗑 ${uids.length}件削除しました`);
}

export async function doBulkStar(ctx: PageState, add: boolean): Promise<void> {
  const a = ctx.settings.accounts[ctx.settings.activeAccountIndex];
  if (!a || ctx.selectedUids.size === 0) return;
  await ctx.ensureValidToken();
  await Promise.allSettled([...ctx.selectedUids].map(uid => toggleStar(a, ctx.activeFolder, uid, add)));
  ctx.showToast(add ? `⭐ ${ctx.selectedUids.size}件にスター追加` : `⭐ ${ctx.selectedUids.size}件のスター解除`);
  ctx.setSelectedUids(new Set());
}

export async function doDeleteExecute(ctx: PageState): Promise<void> {
  const a = ctx.settings.accounts[ctx.settings.activeAccountIndex];
  if (!a || !ctx.selectedUid) return;
  await ctx.ensureValidToken();
  try {
    await deleteMail(a, ctx.activeFolder, ctx.selectedUid);
    ctx.setMails(ctx.mails.filter(m => m.uid !== ctx.selectedUid));
    ctx.setSelectedMail(null); ctx.setSelectedUid(null);
    ctx.showToast('🗑 削除しました');
  } catch (e) { ctx.setError(String(e)); }
}

export async function doStar(ctx: PageState, add: boolean): Promise<void> {
  const a = ctx.settings.accounts[ctx.settings.activeAccountIndex];
  if (!a || !ctx.selectedUid) return;
  await ctx.ensureValidToken();
  try { await toggleStar(a, ctx.activeFolder, ctx.selectedUid, add); ctx.showToast(add ? '⭐ スター追加' : '⭐ スター解除'); }
  catch (e) { ctx.setError(String(e)); }
}

export async function doDownloadAttachment(ctx: PageState, partIndex: number, filename: string): Promise<void> {
  const a = ctx.settings.accounts[ctx.settings.activeAccountIndex];
  if (!a || !ctx.selectedUid) return;
  await ctx.ensureValidToken();
  try { ctx.showToast(`⬇ ${filename} を保存しました: ${await downloadAttachment(a, ctx.activeFolder, ctx.selectedUid, partIndex, filename)}`); }
  catch (e) { ctx.setError(String(e)); }
}

export async function doSelect(ctx: PageState, uid: number, trace: (tag: string, msg: string) => void): Promise<void> {
  const a = ctx.settings.accounts[ctx.settings.activeAccountIndex];
  if (!a) return;
  const t0 = performance.now(); await ctx.ensureValidToken(); const t1 = performance.now();
  ctx.setSelectedUid(uid);
  ctx.setDetailLoading?.(true);
  try {
    const detail = await invoke<MailDetailType>('fetch_mail_detail', { config: getImapConfig(a), folder: ctx.activeFolder, uid });
    if (ctx.selectedUid !== uid) return;
    ctx.setSelectedMail(detail);
    const m = ctx.mails.find(x => x.uid === uid); if (m && !m.seen) { m.seen = true; ctx.setMails(ctx.mails); invoke('mark_mail_seen', { config: getImapConfig(a), folder: ctx.activeFolder, uid }).catch(() => {}); }
    trace('PERF', `select uid=${uid}: token=${(t1-t0)|0}ms, fetch=${(performance.now()-t1)|0}ms`);
    const idx = ctx.mails.findIndex(m => m.uid === uid);
    if (idx >= 0) {
      const nearby = ctx.mails.slice(Math.max(0, idx - 15), Math.min(ctx.mails.length, idx + 16)).map(m => m.uid).filter(u => u !== uid);
      if (nearby.length > 0) invoke('preload_mails', { config: getImapConfig(a), folder: ctx.activeFolder, uids: nearby }).catch(() => {});
    }
  } catch (e) { ctx.setError(String(e)); }
  finally { ctx.setDetailLoading?.(false); }
}
