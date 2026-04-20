<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import MailList from '$lib/components/MailList.svelte';
  import MailDetail from '$lib/components/MailDetail.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import ComposeModal from '$lib/components/ComposeModal.svelte';
  import type { MailSummary, MailDetail as MailDetailType } from '$lib/types';
  import { loadSettings, saveSettings, getImapConfig, getSmtpConfig, getLlmConfig, type AppSettings, type ShortcutMap, DEFAULTS } from '$lib/store';

  function trace(tag: string, msg: string) {
    console.log(`[${tag}] ${msg}`);
    invoke('frontend_trace', { tag, msg }).catch(() => {});
  }

  let settings: AppSettings = $state(structuredClone(DEFAULTS));
  let mails: MailSummary[] = $state([]);
  let folderCache: Map<string, { mails: MailSummary[]; offset: number; hasMore: boolean }> = new Map();
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
  function pageSize() { return settings.mailsPerPage ?? 200; }
  let calendarNames: string[] = $state([]);

  let searchResults: MailSummary[] | null = $state(null);
  let searching = $state(false);
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  let filteredMails: MailSummary[] = $derived(
    searchResults !== null ? searchResults : mails
  );

  function onSearchInput(q: string) {
    searchQuery = q;
    if (searchTimer) clearTimeout(searchTimer);
    if (!q.trim()) { searchResults = null; searching = false; return; }
    searching = true;
    searchTimer = setTimeout(() => doSearch(q), 400);
  }

  async function doSearch(q: string) {
    const a = acc(); if (!a) { searching = false; return; }
    await ensureValidToken();
    try {
      const folder = activeFolder;
      searchResults = await invoke<MailSummary[]>('search_mails', { config: getImapConfig(a), folder, query: q, limit: 100 });
    } catch (e) { trace('SEARCH', `error: ${e}`); searchResults = null; }
    finally { searching = false; }
  }

  function acc() { return settings.accounts?.[settings.activeAccountIndex] ?? null; }
  function llm() { return getLlmConfig(settings.llm); }

  async function ensureValidToken() {
    const a = acc();
    if (!a || a.auth_type !== 'oauth') return;
    const now = Math.floor(Date.now() / 1000);
    if (a.token_expires_at > now + 60) return;
    try {
      const tokens = await invoke<{ access_token: string; refresh_token: string; expires_at: number }>('google_oauth_refresh', { refreshToken: a.refresh_token });
      a.access_token = tokens.access_token;
      a.token_expires_at = tokens.expires_at;
      settings.accounts[settings.activeAccountIndex] = { ...a };
      await saveSettings(settings);
    } catch (e) { error = 'トークン更新失敗: ' + String(e); }
  }

  onMount(async () => {
    trace('MOUNT', 'start');
    try { settings = await loadSettings(); trace('MOUNT', `loadSettings ok, accounts: ${settings.accounts.length}`); }
    catch (e) { trace('MOUNT', `loadSettings FAILED: ${e}`); }
    invoke('set_ai_budget', { limitUsd: settings.aiBudgetLimitUsd ?? 0 }).catch(() => {});
    if (acc()) {
      await fetchMails(); startPolling();
      prefetchAllFolders();
      fetchCalendarNames();
    }
    try {
      const { isPermissionGranted, requestPermission } = await import('@tauri-apps/plugin-notification');
      const granted = await isPermissionGranted();
      if (!granted) await requestPermission();
      trace('MOUNT', 'notification ok');
    } catch (e) { trace('MOUNT', `notification skip: ${e}`); }
    trace('MOUNT', 'done');

    // Intercept link clicks in mail body → open in system browser
    document.addEventListener('click', handleLinkClick);
  });

  function handleLinkClick(e: MouseEvent) {
    const a = (e.target as HTMLElement)?.closest('a[href]') as HTMLAnchorElement | null;
    if (!a) return;
    const href = a.getAttribute('href');
    if (href && (href.startsWith('http://') || href.startsWith('https://'))) {
      e.preventDefault();
      invoke('open_external_url', { url: href }).catch(() => {});
    }
  }

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
    document.removeEventListener('click', handleLinkClick);
  });

  function startPolling() {
    if (pollTimer) clearInterval(pollTimer);
    const interval = (acc()?.syncInterval ?? 5) * 60 * 1000;
    pollTimer = setInterval(async () => {
      await fetchNewMails();
    }, interval);
  }

  async function fetchNewMails() {
    const a = acc(); if (!a) return;
    if (mails.length === 0) { await fetchMails(); return; }
    await ensureValidToken();
    const maxUid = Math.max(...mails.map(m => m.uid));
    const folder = activeFolder;
    try {
      const newMails = await invoke<MailSummary[]>('fetch_new_mails', { config: getImapConfig(a), folder, sinceUid: maxUid });
      if (newMails.length > 0) {
        mails = [...newMails, ...mails];
        syncStatus = `${newMails.length}件の新着`;
        setTimeout(() => { syncStatus = ''; }, 3000);
        // Preload new mails
        const preloadUids = newMails.map(m => m.uid);
        invoke('preload_mails', { config: getImapConfig(a), folder, uids: preloadUids }).catch(() => {});
        // Notification
        if (acc()?.notifications) {
          try {
            const { sendNotification } = await import('@tauri-apps/plugin-notification');
            sendNotification({ title: 'SmartAM', body: `${newMails.length}件の新着メール` });
          } catch {}
        }
      }
    } catch (e) { trace('POLL', `fetch_new_mails error: ${e}`); }
  }

  function cacheKey(folder: string) { return `${settings.activeAccountIndex}:${folder}`; }

  function saveFolderCache() {
    folderCache.set(cacheKey(activeFolder), { mails: [...mails], offset: mailOffset, hasMore });
  }

  async function fetchMails() {
    const a = acc(); if (!a) return;
    await ensureValidToken();
    loading = true; syncing = true; syncStatus = ''; error = null;
    mailOffset = 0; hasMore = true;
    try {
      const folder = activeFolder;
      const initCount = pageSize() * 2;
      const [result, total] = await invoke<[MailSummary[], number]>('fetch_mail_page', { config: getImapConfig(a), folder, offset: 0, limit: initCount });
      mails = result;
      mailOffset = result.length;
      hasMore = mailOffset < total;
      saveFolderCache();
      syncStatus = `${result.length}/${total}件 同期完了`;
      setTimeout(() => { syncStatus = ''; }, 3000);
      if (result.length > 0) {
        const preloadUids = result.map(m => m.uid);
        invoke('preload_mails', { config: getImapConfig(a), folder, uids: preloadUids }).catch(() => {});
      }
    } catch (e) { error = String(e); syncStatus = '同期失敗'; }
    finally { loading = false; syncing = false; }
  }

  async function loadMoreMails() {
    if (!hasMore || loadingMore) return;
    const a = acc(); if (!a) return;
    await ensureValidToken();
    loadingMore = true;
    try {
      const folder = activeFolder;
      const [page, total] = await invoke<[MailSummary[], number]>('fetch_mail_page', { config: getImapConfig(a), folder, offset: mailOffset, limit: pageSize() });
      if (page.length === 0) { hasMore = false; }
      else {
        mails = [...mails, ...page];
        mailOffset += page.length;
        hasMore = mailOffset < total;
        saveFolderCache();
        const newUids = page.map(m => m.uid);
        invoke('preload_mails', { config: getImapConfig(a), folder, uids: newUids }).catch(() => {});
      }
    } catch (e) { error = String(e); }
    finally { loadingMore = false; }
  }

  async function prefetchAllFolders() {
    const allFolders = ['INBOX', 'STARRED', 'SENT', 'DRAFTS', 'ALL', 'SPAM', 'TRASH'];
    for (let i = 0; i < settings.accounts.length; i++) {
      const a = settings.accounts[i];
      if (!a) continue;
      for (const folder of allFolders) {
        const key = `${i}:${folder}`;
        if (folderCache.has(key)) continue;
        try {
          if (a.auth_type === 'oauth') {
            const now = Math.floor(Date.now() / 1000);
            if (a.token_expires_at <= now + 60) {
              const tokens = await invoke<{ access_token: string; refresh_token: string; expires_at: number }>('google_oauth_refresh', { refreshToken: a.refresh_token });
              a.access_token = tokens.access_token;
              a.token_expires_at = tokens.expires_at;
              settings.accounts[i] = { ...a };
            }
          }
          const [result, total] = await invoke<[MailSummary[], number]>('fetch_mail_page', { config: getImapConfig(a), folder, offset: 0, limit: pageSize() * 2 });
          folderCache.set(key, { mails: result, offset: result.length, hasMore: result.length < total });
          trace('PREFETCH', `${a.email}/${folder}: ${result.length}/${total}`);
        } catch (e) { trace('PREFETCH', `${a.email}/${folder} failed: ${e}`); }
      }
    }
  }

  function fetchCalendarNames() {
    const a = acc();
    if (a?.auth_type === 'oauth' && a.access_token) {
      invoke<string[]>('list_google_calendars', { accessToken: a.access_token })
        .then(names => {
          calendarNames = names;
          if (names.length > 0 && !a.calendar?.calendarName) {
            a.calendar.calendarName = names[0];
          }
        })
        .catch(() => { calendarNames = []; });
    } else { calendarNames = []; }
  }

  async function handleFolderChange(folder: string, accountIndex?: number) {
    saveFolderCache();
    if (accountIndex !== undefined) { settings.activeAccountIndex = accountIndex; fetchCalendarNames(); }
    activeFolder = folder;
    selectedMail = null; selectedUid = null;
    searchQuery = ''; searchResults = null;

    // Restore from cache if available
    const cached = folderCache.get(cacheKey(folder));
    if (cached) {
      mails = cached.mails;
      mailOffset = cached.offset;
      hasMore = cached.hasMore;
      // Background: check for new mails
      fetchNewMails();
    } else {
      await fetchMails();
    }
  }

  async function handleSelect(uid: number) {
    const a = acc(); if (!a) return;
    const t0 = performance.now();
    await ensureValidToken();
    const t1 = performance.now();
    selectedUid = uid;
    const folder = activeFolder;
    try {
      selectedMail = await invoke<MailDetailType>('fetch_mail_detail', { config: getImapConfig(a), folder, uid });
      const t2 = performance.now();
      trace('PERF', `select uid=${uid}: token=${(t1-t0)|0}ms, fetch=${(t2-t1)|0}ms, total=${(t2-t0)|0}ms`);
      // Preload surrounding mails in background
      const idx = mails.findIndex(m => m.uid === uid);
      if (idx >= 0) {
        const start = Math.max(0, idx - 15);
        const end = Math.min(mails.length, idx + 16);
        const nearby = mails.slice(start, end).map(m => m.uid).filter(u => u !== uid);
        if (nearby.length > 0) {
          invoke('preload_mails', { config: getImapConfig(a), folder, uids: nearby }).catch(() => {});
        }
      }
    }
    catch (e) { error = String(e); }
  }


  async function handleArchive() {
    const a = acc(); if (!a || !selectedUid) return;
    const uid = selectedUid;
    const idx = mails.findIndex(m => m.uid === uid);
    const prevMails = mails;

    // Optimistic: remove immediately
    mails = mails.filter(m => m.uid !== uid);
    const next = mails[idx] ?? mails[idx - 1];
    if (next) { handleSelect(next.uid); } else { selectedMail = null; selectedUid = null; }
    showToast('📦 アーカイブしました', () => { mails = prevMails; handleSelect(uid); });

    // Fire-and-forget IMAP with retry
    const doArchive = async (attempt: number) => {
      try {
        await ensureValidToken();
        await invoke('archive_mail', { config: getImapConfig(a), folder: activeFolder, uid });
      } catch (e) {
        if (attempt < 10) { setTimeout(() => doArchive(attempt + 1), 1000 * attempt); }
        else { mails = prevMails; handleSelect(uid); error = 'アーカイブ失敗: ' + String(e); }
      }
    };
    doArchive(1);
  }

  function handleDeleteConfirm() { confirmDelete = true; }
  async function handleDeleteExecute() {
    confirmDelete = false;
    const a = acc(); if (!a || !selectedUid) return;
    await ensureValidToken();
    try {
      await invoke('delete_mail', { config: getImapConfig(a), folder: activeFolder, uid: selectedUid });
      mails = mails.filter(m => m.uid !== selectedUid);
      selectedMail = null; selectedUid = null;
      showToast('🗑 削除しました');
    } catch (e) { error = String(e); }
  }

  async function handleStar(add: boolean) {
    const a = acc(); if (!a || !selectedUid) return;
    await ensureValidToken();
    try {
      await invoke('toggle_star', { config: getImapConfig(a), folder: activeFolder, uid: selectedUid, add });
      showToast(add ? '⭐ スター追加' : '⭐ スター解除');
    } catch (e) { error = String(e); }
  }

  async function handleDownloadAttachment(partIndex: number, filename: string) {
    const a = acc(); if (!a || !selectedUid) return;
    await ensureValidToken();
    try {
      const path = await invoke<string>('download_attachment', {
        config: getImapConfig(a), folder: activeFolder, uid: selectedUid, partIndex, filename
      });
      showToast(`⬇ ${filename} を保存しました: ${path}`);
    } catch (e) { error = String(e); }
  }

  function openReply(presetBody?: string) { composeBody = presetBody ?? ''; composeMode = 'reply'; attachmentPaths = []; }

  async function handleSend(data: { to: string; cc: string; bcc: string; subject: string; body: string }) {
    const a = acc(); if (!a) return;
    await ensureValidToken();
    try {
      const toArr = data.to.split(',').map(s => s.trim()).filter(Boolean);
      const ccArr = data.cc.split(',').map(s => s.trim()).filter(Boolean);
      const bccArr = data.bcc.split(',').map(s => s.trim()).filter(Boolean);
      if (attachmentPaths.length > 0) {
        await invoke('send_mail_with_attachments', {
          config: getSmtpConfig(a), to: toArr, cc: ccArr, bcc: bccArr,
          subject: data.subject, body: data.body, attachmentPaths
        });
      } else {
        await invoke('send_mail', {
          config: getSmtpConfig(a), to: toArr, cc: ccArr, bcc: bccArr,
          subject: data.subject, body: data.body
        });
      }
      composeMode = null; attachmentPaths = [];
      showToast('✅ メールを送信しました');
    } catch (e) { error = String(e); }
  }

  function showToast(msg: string, undo?: () => void) {
    toast = { msg, undo }; if (undo) lastUndo = undo; setTimeout(() => toast = null, 5000);
  }

  // --- Keyboard shortcuts ---
  let gPending = $state(false);
  let gTimer: ReturnType<typeof setTimeout> | null = null;
  let lastUndo: (() => void) | null = null;

  function sc() { return settings.shortcuts; }

  function reverseMap(): Map<string, string> {
    const m = new Map<string, string>();
    for (const [action, key] of Object.entries(sc())) m.set(key, action);
    return m;
  }

  function handleKeydown(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;
    if (showSettings || composeMode) return;

    const s = sc();
    const key = e.key;

    // g-sequence
    if (gPending) {
      gPending = false;
      if (gTimer) { clearTimeout(gTimer); gTimer = null; }
      const combo = `g ${key}`;
      const rm = reverseMap();
      const action = rm.get(combo);
      if (action) { e.preventDefault(); execAction(action); }
      return;
    }
    if (key === 'g' && Object.values(s).some(v => v.startsWith('g '))) {
      gPending = true;
      gTimer = setTimeout(() => { gPending = false; }, 1000);
      return;
    }

    // Arrow keys: navigate mails when detail is open
    if (selectedMail && (key === 'ArrowDown' || key === 'ArrowUp')) {
      e.preventDefault();
      execAction(key === 'ArrowDown' ? 'nextMail' : 'prevMail');
      return;
    }

    const rm = reverseMap();
    const action = rm.get(key);
    if (action) { e.preventDefault(); execAction(action); }
  }

  function execAction(action: string) {
    const idx = mails.findIndex(m => m.uid === selectedUid);
    switch (action) {
      case 'nextMail':
        if (idx >= 0 && idx < mails.length - 1) handleSelect(mails[idx + 1].uid);
        else if (idx === -1 && mails.length > 0) handleSelect(mails[0].uid);
        break;
      case 'prevMail':
        if (idx > 0) handleSelect(mails[idx - 1].uid);
        break;
      case 'openMail':
        if (selectedUid && !selectedMail) handleSelect(selectedUid);
        break;
      case 'backToList':
        selectedMail = null; selectedUid = null;
        break;
      case 'reply': if (selectedMail) openReply(); break;
      case 'forward': if (selectedMail) { composeBody = `\n\n---------- Forwarded ----------\n${selectedMail.body_text ?? ''}`; composeMode = 'forward'; } break;
      case 'archive': if (selectedMail) handleArchive(); break;
      case 'delete': if (selectedMail) handleDeleteConfirm(); break;
      case 'star': if (selectedUid) { const next = true; handleStar(next); } break;
      case 'undo': if (lastUndo) { lastUndo(); lastUndo = null; } break;
      case 'compose': composeBody = ''; composeMode = 'new'; attachmentPaths = []; break;
      case 'search': document.querySelector<HTMLInputElement>('.mail-list input')?.focus(); break;
      case 'goInbox': handleFolderChange('INBOX'); break;
      case 'goStarred': handleFolderChange('STARRED'); break;
      case 'goSent': handleFolderChange('SENT'); break;
      case 'goDrafts': handleFolderChange('DRAFTS'); break;
      case 'goAll': handleFolderChange('ALL'); break;
      case 'aiSummary': if (selectedMail) document.querySelector<HTMLButtonElement>('[title="AI要約"]')?.click(); break;
      case 'aiDraft': if (selectedMail) document.querySelector<HTMLButtonElement>('[title="返信下書き"]')?.click(); break;
      case 'aiTranslate': if (selectedMail) document.querySelector<HTMLButtonElement>('[title="翻訳"]')?.click(); break;
      case 'aiCalendar': if (selectedMail) document.querySelector<HTMLButtonElement>('[title="カレンダー登録"]')?.click(); break;
      case 'acceptInvite': document.querySelector<HTMLButtonElement>('.btn-accept:not(:disabled)')?.click(); break;
      case 'declineInvite': document.querySelector<HTMLButtonElement>('.btn-decline:not(:disabled)')?.click(); break;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="layout">
  <Sidebar
    onOpenSettings={() => { trace('BTN', 'settings clicked'); showSettings = true; }}
    onCompose={() => { composeBody = ''; composeMode = 'new'; attachmentPaths = []; }}
    onRefresh={fetchNewMails}
    onFolderSelect={(i, folder) => handleFolderChange(folder, i)}
    {activeFolder}
    accounts={settings?.accounts ?? []}
    activeAccountIndex={settings?.activeAccountIndex ?? 0}
    llmLabel={settings ? `${settings.llm.activeProvider} (${settings.llm[settings.llm.activeProvider]?.model ?? ''})` : ''}
    {syncing}
    {syncStatus}
  />
  <MailList mails={filteredMails} {selectedUid} onSelect={handleSelect} onLoadMore={loadMoreMails} {loading} loadingMore={loadingMore || searching} bind:searchQuery onSearchInput={onSearchInput} pageSize={pageSize()} dateFormat={settings.dateFormat} timezone={settings.timezone} />
  <MailDetail
    mail={selectedMail}
    onArchive={handleArchive}
    onDelete={handleDeleteConfirm}
    onStar={handleStar}
    onReply={() => openReply()}
    onForward={() => { composeBody = `\n\n---------- Forwarded ----------\n${selectedMail?.body_text ?? ''}`; composeMode = 'forward'; }}
    onUseAiReply={(text) => openReply(text)}
    onDownloadAttachment={handleDownloadAttachment}
    onFetchAttachmentData={async (partIndex) => {
      const a = acc(); if (!a || !selectedUid) throw new Error('no account');
      await ensureValidToken();
      return invoke<string>('fetch_attachment_data', { config: getImapConfig(a), folder: activeFolder, uid: selectedUid, partIndex });
    }}
    llmConfig={llm()}
    smtpConfig={acc() ? getSmtpConfig(acc()!) : null}
    calendarName={acc()?.calendar?.calendarName ?? '仕事'}
    {calendarNames}
  />
</div>

{#if confirmDelete}
  <div class="dialog-overlay">
    <div class="dialog">
      <div class="dialog-title">メールを削除</div>
      <p class="dialog-msg">「{selectedMail?.subject}」を削除しますか？<br/>この操作はゴミ箱に移動します。</p>
      <div class="dialog-actions">
        <button class="btn-cancel" onclick={() => confirmDelete = false}>キャンセル</button>
        <button class="btn-delete" onclick={handleDeleteExecute}>削除する</button>
      </div>
    </div>
  </div>
{/if}

{#if composeMode}
  <ComposeModal
    mode={composeMode}
    to={composeMode === 'reply' ? (selectedMail?.from.replace(/.*<(.+)>.*/, '$1') ?? '') : ''}
    subject={selectedMail?.subject ?? ''}
    body={composeBody}
    signature={acc()?.signature ?? ''}
    onClose={() => composeMode = null}
    onSend={handleSend}
    bind:attachmentPaths
  />
{/if}

{#if showSettings}
  <Settings {settings} onClose={() => showSettings = false} onSave={async (s) => { try { const plain = JSON.parse(JSON.stringify(s)); await saveSettings(plain); settings = plain; invoke('set_ai_budget', { limitUsd: plain.aiBudgetLimitUsd ?? 0 }).catch(() => {}); showSettings = false; startPolling(); await fetchMails(); } catch (e) { error = '設定の保存に失敗: ' + String(e); } }} />
{/if}

{#if toast}
  <div class="toast-bar">{toast.msg}{#if toast.undo}<button class="undo" onclick={() => { toast?.undo?.(); toast = null; }}>元に戻す</button>{/if}</div>
{/if}
{#if error}<div class="toast-err">{error} <button onclick={() => error = null}>✕</button></div>{/if}

<style>
  .layout { display:flex;flex:1;height:100vh;overflow:hidden }
  .dialog-overlay { position:fixed;inset:0;background:rgba(0,0,0,.5);display:flex;align-items:center;justify-content:center;z-index:60 }
  .dialog { background:var(--base);border:1px solid var(--red);border-radius:8px;padding:24px;width:380px;text-align:center }
  .dialog-title { font-size:14px;font-weight:700;margin-bottom:12px }
  .dialog-msg { font-size:12px;color:var(--text);margin-bottom:16px;line-height:1.5 }
  .dialog-actions { display:flex;gap:8px;justify-content:center }
  .btn-cancel { padding:8px 20px;border-radius:6px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);cursor:pointer }
  .btn-delete { padding:8px 20px;border-radius:6px;border:none;background:var(--red);color:var(--base);font-weight:700;cursor:pointer }
  .toast-bar { position:fixed;bottom:16px;right:16px;padding:10px 16px;background:#1e3a2e;color:var(--green);border:1px solid var(--green);border-radius:8px;font-size:12px;z-index:100;display:flex;gap:12px;align-items:center }
  .undo { background:none;border:none;color:var(--blue);cursor:pointer;font-size:11px;text-decoration:underline }
  .toast-err { position:fixed;bottom:56px;right:16px;padding:10px 16px;background:#3a1e1e;color:var(--red);border:1px solid var(--red);border-radius:8px;font-size:12px;z-index:100 }
  .toast-err button { background:none;border:none;color:var(--red);cursor:pointer;margin-left:8px }
</style>
