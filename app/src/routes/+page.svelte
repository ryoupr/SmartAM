<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import MailList from '$lib/components/MailList.svelte';
  import MailDetail from '$lib/components/MailDetail.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import ComposeModal from '$lib/components/ComposeModal.svelte';
  import ShortcutManager from '$lib/components/ShortcutManager.svelte';
  import ToastNotification from '$lib/components/ToastNotification.svelte';
  import UpdateBanner from '$lib/components/UpdateBanner.svelte';
  import ConfirmDeleteDialog from '$lib/components/ConfirmDeleteDialog.svelte';
  import type { MailSummary, MailDetail as MailDetailType } from '$lib/types';
  import { loadSettings, saveSettings, getImapConfig, getSmtpConfig, getLlmConfig, type AppSettings, DEFAULTS } from '$lib/store';
  import { refreshOAuthToken, fetchMailPage, fetchNewMailsSince, prefetchAllFolders, sendNotificationForNewMails, type FolderCache } from '$lib/mailSync';
  import { archiveMail, deleteMail, toggleStar, downloadAttachment, sendMail } from '$lib/mailActions';

  function trace(tag: string, msg: string) { console.log(`[${tag}] ${msg}`); invoke('frontend_trace', { tag, msg }).catch(() => {}); }

  let settings: AppSettings = $state(structuredClone(DEFAULTS));
  let mails: MailSummary[] = $state([]);
  let folderCache: FolderCache = new Map();
  let selectedMail: MailDetailType | null = $state(null);
  let selectedUid: number | null = $state(null);
  let loading = $state(false);
  let error: string | null = $state(null);
  let showSettings = $state(false);
  let composeMode: 'new' | 'reply' | 'forward' | null = $state(null);
  let composeBody = $state('');
  let toast: { msg: string; undo?: () => void } | null = $state(null);
  let confirmDelete = $state(false);
  let searchQuery = $state('');
  let activeFolder = $state('INBOX');
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let attachmentPaths: string[] = $state([]);
  let syncing = $state(false);
  let syncStatus = $state('');
  let loadingMore = $state(false);
  let mailOffset = 0;
  let hasMore = $state(true);
  let calendarNames: string[] = $state([]);
  let searchResults: MailSummary[] | null = $state(null);
  let searching = $state(false);
  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  let selectedUids: Set<number> = $state(new Set());
  let lastUndo: (() => void) | null = null;

  const filteredMails = $derived(searchResults !== null ? searchResults : mails);
  const ps = $derived(settings.mailsPerPage ?? 200);

  function acc() { return settings.accounts?.[settings.activeAccountIndex] ?? null; }
  function llm() { return getLlmConfig(settings.llm); }
  function cacheKey(f: string) { return `${settings.activeAccountIndex}:${f}`; }
  function saveFolderCache() { folderCache.set(cacheKey(activeFolder), { mails: [...mails], offset: mailOffset, hasMore }); }
  async function ensureValidToken() { await refreshOAuthToken(settings, settings.activeAccountIndex); }
  function showToast(msg: string, undo?: () => void) {
    toast = { msg, undo }; if (undo) lastUndo = undo;
    setTimeout(() => { toast = null; }, 5000);
    if (undo) setTimeout(() => { if (lastUndo === undo) lastUndo = null; }, 5000);
  }

  function onSearchInput(q: string) {
    searchQuery = q; if (searchTimer) clearTimeout(searchTimer);
    if (!q.trim()) { searchResults = null; searching = false; return; }
    searching = true;
    searchTimer = setTimeout(async () => {
      const a = acc(); if (!a) { searching = false; return; }
      await ensureValidToken();
      try { const r = await invoke<MailSummary[]>('search_mails', { config: getImapConfig(a), folder: activeFolder, query: q, limit: 100 }); if (searchQuery === q) searchResults = r; }
      catch { searchResults = null; } finally { searching = false; }
    }, 400);
  }

  async function fetchMails() {
    const a = acc(); if (!a) return;
    await ensureValidToken(); loading = true; syncing = true; syncStatus = ''; error = null; mailOffset = 0; hasMore = true;
    try {
      const [result, total] = await fetchMailPage(settings, activeFolder, 0, ps * 2);
      mails = result; mailOffset = result.length; hasMore = mailOffset < total; saveFolderCache();
      syncStatus = `${result.length}/${total}件 同期完了`; setTimeout(() => { syncStatus = ''; }, 3000);
      if (result.length > 0) invoke('preload_mails', { config: getImapConfig(a), folder: activeFolder, uids: result.map(m => m.uid) }).catch(() => {});
    } catch (e) { error = String(e); syncStatus = '同期失敗'; } finally { loading = false; syncing = false; }
  }

  async function fetchNewMails() {
    const a = acc(); if (!a) return;
    if (mails.length === 0) { await fetchMails(); return; }
    await ensureValidToken();
    try {
      const newMails = await fetchNewMailsSince(settings, activeFolder, Math.max(...mails.map(m => m.uid)));
      if (newMails.length > 0) {
        mails = [...newMails, ...mails]; syncStatus = `${newMails.length}件の新着`; setTimeout(() => { syncStatus = ''; }, 3000);
        invoke('preload_mails', { config: getImapConfig(a), folder: activeFolder, uids: newMails.map(m => m.uid) }).catch(() => {});
        await sendNotificationForNewMails(settings, newMails, mails, trace);
      }
    } catch (e) { trace('POLL', `fetch_new_mails error: ${e}`); }
  }

  async function loadMoreMails() {
    if (!hasMore || loadingMore) return; const a = acc(); if (!a) return;
    await ensureValidToken(); loadingMore = true;
    try {
      const [page, total] = await fetchMailPage(settings, activeFolder, mailOffset, ps);
      if (page.length === 0) { hasMore = false; }
      else { mails = [...mails, ...page]; mailOffset += page.length; hasMore = mailOffset < total; saveFolderCache(); invoke('preload_mails', { config: getImapConfig(a), folder: activeFolder, uids: page.map(m => m.uid) }).catch(() => {}); }
    } catch (e) { error = String(e); } finally { loadingMore = false; }
  }

  function startPolling() {
    if (pollTimer) clearInterval(pollTimer);
    // Legacy polling as supplement — IdleWatcher handles primary notification
    pollTimer = setInterval(fetchNewMails, (acc()?.syncInterval ?? 5) * 60 * 1000);
    // Start Rust-side IdleWatcher
    restartIdleWatcher();
  }

  function restartIdleWatcher() {
    const configs = settings.accounts
      .map((a, i) => a.notifications ? {
        account: { email: a.email, auth_type: a.auth_type, password: a.password, access_token: a.access_token, imap_host: a.imap_host, imap_port: a.imap_port },
        account_index: i,
        folders: a.notificationFolders ?? ['INBOX'],
        sync_interval_secs: (a.syncInterval ?? 5) * 60,
        notification_sound: a.notificationSound ?? true,
        notification_sound_name: settings.notificationSoundName ?? 'default',
      } : null)
      .filter((c): c is NonNullable<typeof c> => c !== null);
    invoke('restart_idle_watcher', { configs }).catch(e => trace('IDLE', `restart failed: ${e}`));
  }

  function fetchCalendarNames() {
    const a = acc();
    if (a?.auth_type === 'oauth' && a.access_token) {
      invoke<string[]>('list_google_calendars', { accessToken: a.access_token })
        .then(names => { calendarNames = names; if (names.length > 0 && !a.calendar?.calendarName && a.calendar) a.calendar.calendarName = names[0]; })
        .catch(() => { calendarNames = []; });
    } else { calendarNames = []; }
  }

  async function handleFolderChange(folder: string, accountIndex?: number) {
    saveFolderCache();
    if (accountIndex !== undefined) { settings.activeAccountIndex = accountIndex; fetchCalendarNames(); selectedUids = new Set(); startPolling(); }
    activeFolder = folder; selectedMail = null; selectedUid = null; searchQuery = ''; searchResults = null;
    const cached = folderCache.get(cacheKey(folder));
    if (cached) { mails = cached.mails; mailOffset = cached.offset; hasMore = cached.hasMore; fetchNewMails(); } else { await fetchMails(); }
  }

  async function handleSelect(uid: number) {
    const a = acc(); if (!a) return;
    const t0 = performance.now(); await ensureValidToken(); const t1 = performance.now(); selectedUid = uid;
    try {
      const detail = await invoke<MailDetailType>('fetch_mail_detail', { config: getImapConfig(a), folder: activeFolder, uid });
      if (selectedUid !== uid) return; selectedMail = detail;
      const m = mails.find(x => x.uid === uid); if (m && !m.seen) { m.seen = true; mails = mails; }
      trace('PERF', `select uid=${uid}: token=${(t1-t0)|0}ms, fetch=${(performance.now()-t1)|0}ms`);
      const idx = mails.findIndex(m => m.uid === uid);
      if (idx >= 0) { const nearby = mails.slice(Math.max(0, idx - 15), Math.min(mails.length, idx + 16)).map(m => m.uid).filter(u => u !== uid); if (nearby.length > 0) invoke('preload_mails', { config: getImapConfig(a), folder: activeFolder, uids: nearby }).catch(() => {}); }
    } catch (e) { error = String(e); }
  }

  async function handleArchive() {
    const a = acc(); if (!a || !selectedUid) return;
    const uid = selectedUid; const idx = mails.findIndex(m => m.uid === uid); const prevMails = mails;
    const prevSearch = searchResults;
    mails = mails.filter(m => m.uid !== uid);
    if (searchResults) searchResults = searchResults.filter(m => m.uid !== uid);
    if (selectedUids.has(uid)) { const next = new Set(selectedUids); next.delete(uid); selectedUids = next; }
    const next = mails[idx] ?? mails[idx - 1]; if (next) handleSelect(next.uid); else { selectedMail = null; selectedUid = null; }
    let aborted = false;
    showToast('📦 アーカイブしました', () => { aborted = true; mails = prevMails; searchResults = prevSearch; handleSelect(uid); });
    archiveMail(a, activeFolder, uid, 1, () => aborted).catch(e => { mails = prevMails; searchResults = prevSearch; handleSelect(uid); error = 'アーカイブ失敗: ' + String(e); });
  }

  async function handleBulkArchive() {
    const a = acc(); if (!a || selectedUids.size === 0) return;
    const uids = [...selectedUids]; const prevMails = mails; const count = uids.length;
    const prevSearch = searchResults;
    const uidSet = new Set(uids);
    mails = mails.filter(m => !uidSet.has(m.uid)); selectedMail = null; selectedUid = null; selectedUids = new Set();
    if (searchResults) searchResults = searchResults.filter(m => !uidSet.has(m.uid));
    showToast(`📦 ${count}件アーカイブしました`, () => { mails = prevMails; searchResults = prevSearch; selectedUids = new Set(uids); });
    for (const uid of uids) archiveMail(a, activeFolder, uid, 1, () => false).catch(() => {});
  }

  async function handleBulkDelete() {
    const a = acc(); if (!a || selectedUids.size === 0) return;
    const uids = [...selectedUids]; await ensureValidToken();
    for (const uid of uids) { try { await deleteMail(a, activeFolder, uid); } catch {} }
    const uidSet = new Set(uids);
    mails = mails.filter(m => !uidSet.has(m.uid)); selectedMail = null; selectedUid = null; selectedUids = new Set();
    if (searchResults) searchResults = searchResults.filter(m => !uidSet.has(m.uid));
    showToast(`🗑 ${uids.length}件削除しました`);
  }

  async function handleBulkStar(add: boolean) {
    const a = acc(); if (!a || selectedUids.size === 0) return; await ensureValidToken();
    for (const uid of [...selectedUids]) { try { await toggleStar(a, activeFolder, uid, add); } catch {} }
    showToast(add ? `⭐ ${selectedUids.size}件にスター追加` : `⭐ ${selectedUids.size}件のスター解除`); selectedUids = new Set();
  }

  async function handleDeleteExecute() {
    confirmDelete = false; const a = acc(); if (!a || !selectedUid) return; await ensureValidToken();
    const uid = selectedUid;
    try { await deleteMail(a, activeFolder, uid); mails = mails.filter(m => m.uid !== uid); if (searchResults) searchResults = searchResults.filter(m => m.uid !== uid); if (selectedUids.has(uid)) { const next = new Set(selectedUids); next.delete(uid); selectedUids = next; } selectedMail = null; selectedUid = null; showToast('🗑 削除しました'); }
    catch (e) { error = String(e); }
  }

  async function handleStar(add: boolean) {
    const a = acc(); if (!a || !selectedUid) return; await ensureValidToken();
    try { await toggleStar(a, activeFolder, selectedUid, add); showToast(add ? '⭐ スター追加' : '⭐ スター解除'); } catch (e) { error = String(e); }
  }

  async function handleDownloadAttachment(partIndex: number, filename: string) {
    const a = acc(); if (!a || !selectedUid) return; await ensureValidToken();
    try { showToast(`⬇ ${filename} を保存しました: ${await downloadAttachment(a, activeFolder, selectedUid, partIndex, filename)}`); } catch (e) { error = String(e); }
  }

  function openReply(presetBody?: string) { composeBody = presetBody ?? ''; composeMode = 'reply'; attachmentPaths = []; }

  async function handleSend(data: { to: string; cc: string; bcc: string; subject: string; body: string }) {
    const a = acc(); if (!a) return; await ensureValidToken();
    try { await sendMail(a, data, attachmentPaths); composeMode = null; attachmentPaths = []; showToast('✅ メールを送信しました'); } catch (e) { error = String(e); }
  }

  function execAction(action: string) {
    if (selectedUids.size > 0 && action === 'archive') { handleBulkArchive(); return; }
    if (selectedUids.size > 0 && action === 'delete') { handleBulkDelete(); return; }
    if (selectedUids.size > 0 && action === 'star') { handleBulkStar(true); return; }
    const idx = filteredMails.findIndex(m => m.uid === selectedUid);
    const btn = (s: string) => document.querySelector<HTMLButtonElement>(s)?.click();
    ({
      nextMail: () => { if (idx >= 0 && idx < filteredMails.length - 1) handleSelect(filteredMails[idx + 1].uid); else if (idx === -1 && filteredMails.length > 0) handleSelect(filteredMails[0].uid); },
      prevMail: () => { if (idx > 0) handleSelect(filteredMails[idx - 1].uid); },
      openMail: () => { if (selectedUid && !selectedMail) handleSelect(selectedUid); },
      backToList: () => { selectedMail = null; selectedUid = null; },
      reply: () => { if (selectedMail) openReply(); },
      forward: () => { if (selectedMail) { composeBody = `\n\n---------- Forwarded ----------\n${selectedMail.body_text ?? ''}`; composeMode = 'forward'; } },
      archive: () => { if (selectedMail) handleArchive(); },
      delete: () => { if (selectedMail) confirmDelete = true; },
      star: () => { if (selectedUid) handleStar(!((selectedMail as any)?.starred ?? false)); },
      undo: () => { if (lastUndo) { lastUndo(); lastUndo = null; } },
      compose: () => { composeBody = ''; composeMode = 'new'; attachmentPaths = []; },
      search: () => document.querySelector<HTMLInputElement>('.mail-list input')?.focus(),
      goInbox: () => handleFolderChange('INBOX'), goStarred: () => handleFolderChange('STARRED'),
      goSent: () => handleFolderChange('SENT'), goDrafts: () => handleFolderChange('DRAFTS'), goAll: () => handleFolderChange('ALL'),
      aiSummary: () => { if (selectedMail) btn('[title="AI要約"]'); },
      aiDraft: () => { if (selectedMail) btn('[title="返信下書き"]'); },
      aiTranslate: () => { if (selectedMail) btn('[title="翻訳"]'); },
      aiCalendar: () => { if (selectedMail) btn('[title="カレンダー登録"]'); },
      acceptInvite: () => btn('.btn-accept:not(:disabled)'), declineInvite: () => btn('.btn-decline:not(:disabled)'),
    } as Record<string, () => void>)[action]?.();
  }

  onMount(async () => {
    trace('MOUNT', 'start');
    try { settings = await loadSettings(); } catch (e) { trace('MOUNT', `loadSettings FAILED: ${e}`); }
    invoke('set_ai_budget', { limitUsd: settings.aiBudgetLimitUsd ?? 0 }).catch(() => {});
    invoke('set_log_level', { level: settings.logLevel ?? 'info' }).catch(() => {});
    if (acc()) { await fetchMails(); startPolling(); prefetchAllFolders(settings, folderCache, ps, trace); fetchCalendarNames(); }
    try { const { isPermissionGranted, requestPermission } = await import('@tauri-apps/plugin-notification'); if (!await isPermissionGranted()) await requestPermission(); } catch {}
    // Listen for new-mail events from IdleWatcher (Rust backend)
    try {
      const { listen } = await import('@tauri-apps/api/event');
      await listen<{ account_index: number; mails: typeof mails }>('new-mail', (event) => {
        if (event.payload.account_index === settings.activeAccountIndex) {
          const newMails = event.payload.mails;
          mails = [...newMails, ...mails];
          syncStatus = `${newMails.length}件の新着`;
          setTimeout(() => { syncStatus = ''; }, 3000);
        }
      });
    } catch (e) { trace('MOUNT', `event listen failed: ${e}`); }
    document.addEventListener('click', handleLinkClick);
    trace('MOUNT', 'done');
  });

  function handleLinkClick(e: MouseEvent) {
    const a = (e.target as HTMLElement)?.closest('a[href]') as HTMLAnchorElement | null; if (!a) return;
    const href = a.getAttribute('href');
    if (href?.startsWith('http://') || href?.startsWith('https://')) { e.preventDefault(); invoke('open_external_url', { url: href }).catch(() => {}); }
  }

  onDestroy(() => { if (pollTimer) clearInterval(pollTimer); if (searchTimer) clearTimeout(searchTimer); document.removeEventListener('click', handleLinkClick); });
</script>

<ShortcutManager shortcuts={settings.shortcuts} onAction={execAction} disabled={showSettings || !!composeMode} />

<div class="layout">
  <Sidebar
    onOpenSettings={() => { trace('BTN', 'settings clicked'); showSettings = true; }}
    onCompose={() => { composeBody = ''; composeMode = 'new'; attachmentPaths = []; }}
    onRefresh={fetchNewMails} onFolderSelect={(i, folder) => handleFolderChange(folder, i)}
    {activeFolder} accounts={settings?.accounts ?? []} activeAccountIndex={settings?.activeAccountIndex ?? 0}
    llmLabel={settings ? `${settings.llm.activeProvider} (${settings.llm[settings.llm.activeProvider]?.model ?? ''})` : ''}
    {syncing} {syncStatus}
  />
  <MailList mails={filteredMails} {selectedUid} onSelect={handleSelect} onLoadMore={loadMoreMails} {loading} loadingMore={loadingMore || searching} bind:searchQuery onSearchInput={onSearchInput} pageSize={ps} dateFormat={settings.dateFormat} timezone={settings.timezone} bind:selectedUids />
  <MailDetail
    mail={selectedMail} onArchive={handleArchive} onDelete={() => confirmDelete = true} onStar={handleStar}
    onReply={() => openReply()}
    onForward={() => { composeBody = `\n\n---------- Forwarded ----------\n${selectedMail?.body_text ?? ''}`; composeMode = 'forward'; }}
    onUseAiReply={(text) => openReply(text)} onDownloadAttachment={handleDownloadAttachment}
    onFetchAttachmentData={async (partIndex) => { const a = acc(); if (!a || !selectedUid) throw new Error('no account'); await ensureValidToken(); return invoke<string>('fetch_attachment_data', { config: getImapConfig(a), folder: activeFolder, uid: selectedUid, partIndex }); }}
    llmConfig={llm()} smtpConfig={acc() ? getSmtpConfig(acc()!) : null}
    calendarName={acc()?.calendar?.calendarName ?? '仕事'} calendarProvider={acc()?.calendar?.provider ?? 'apple'}
    {calendarNames} dateFormat={settings.dateFormat} timezone={settings.timezone}
  />
</div>

{#if confirmDelete}
  <ConfirmDeleteDialog subject={selectedMail?.subject ?? ''} onConfirm={handleDeleteExecute} onCancel={() => confirmDelete = false} />
{/if}

{#if composeMode}
  <ComposeModal mode={composeMode} to={composeMode === 'reply' ? (selectedMail?.from.replace(/.*<(.+)>.*/, '$1') ?? '') : ''} subject={selectedMail?.subject ?? ''} body={composeBody} signature={acc()?.signature ?? ''} onClose={() => composeMode = null} onSend={handleSend} bind:attachmentPaths />
{/if}

{#if showSettings}
  <Settings {settings} onClose={() => showSettings = false} onSave={async (s) => {
    try { const plain = JSON.parse(JSON.stringify(s)); await saveSettings(plain); settings = plain; invoke('set_ai_budget', { limitUsd: plain.aiBudgetLimitUsd ?? 0 }).catch(() => {}); showSettings = false; startPolling(); await fetchMails(); fetchCalendarNames(); }
    catch (e) { error = '設定の保存に失敗: ' + String(e); }
  }} />
{/if}

<UpdateBanner onError={(msg) => error = msg} />
<ToastNotification {toast} {error} onDismissToast={() => toast = null} onDismissError={() => error = null} />

<style>
  .layout { display:flex;flex:1;height:100vh;overflow:hidden }
</style>
