import { invoke } from '@tauri-apps/api/core';
import type { MailSummary, MailDetail as MailDetailType, AccountConfig } from '$lib/types';
import { getImapConfig } from '$lib/store';
import type { Account } from '$lib/store';

// State
let mails = $state<MailSummary[]>([]);
let selectedMail = $state<MailDetailType | null>(null);
let selectedUid = $state<number | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);
let activeFolder = $state('INBOX');
let hasMore = $state(true);
let syncing = $state(false);
let syncStatus = $state('');
let searchQuery = $state('');
let searchResults = $state<MailSummary[] | null>(null);

let mailOffset = 0;

async function ensureValidToken(account: Account, onTokenRefreshed?: (account: Account) => void) {
  if (account.auth_type !== 'oauth') return;
  const now = Math.floor(Date.now() / 1000);
  if (account.token_expires_at > now + 60) return;
  const tokens = await invoke<{ access_token: string; refresh_token: string; expires_at: number }>(
    'google_oauth_refresh', { refreshToken: account.refresh_token }
  );
  account.access_token = tokens.access_token;
  account.token_expires_at = tokens.expires_at;
  onTokenRefreshed?.(account);
}

async function fetchMails(account: Account, pageSize = 200, onTokenRefreshed?: (a: Account) => void) {
  await ensureValidToken(account, onTokenRefreshed);
  loading = true; syncing = true; syncStatus = ''; error = null;
  mailOffset = 0; hasMore = true;
  try {
    const folder = activeFolder;
    const initCount = pageSize * 2;
    const [result, total] = await invoke<[MailSummary[], number]>(
      'fetch_mail_page', { config: getImapConfig(account), folder, offset: 0, limit: initCount }
    );
    mails = result;
    mailOffset = result.length;
    hasMore = mailOffset < total;
    syncStatus = `${result.length}/${total}件 同期完了`;
    setTimeout(() => { syncStatus = ''; }, 3000);
    if (result.length > 0) {
      invoke('preload_mails', { config: getImapConfig(account), folder, uids: result.map(m => m.uid) }).catch(() => {});
    }
  } catch (e) { error = String(e); syncStatus = '同期失敗'; }
  finally { loading = false; syncing = false; }
}

async function loadMore(account: Account, pageSize = 200, onTokenRefreshed?: (a: Account) => void) {
  if (!hasMore) return;
  await ensureValidToken(account, onTokenRefreshed);
  try {
    const folder = activeFolder;
    const [page, total] = await invoke<[MailSummary[], number]>(
      'fetch_mail_page', { config: getImapConfig(account), folder, offset: mailOffset, limit: pageSize }
    );
    if (page.length === 0) { hasMore = false; }
    else {
      mails = [...mails, ...page];
      mailOffset += page.length;
      hasMore = mailOffset < total;
      invoke('preload_mails', { config: getImapConfig(account), folder, uids: page.map(m => m.uid) }).catch(() => {});
    }
  } catch (e) { error = String(e); }
}

async function selectMail(account: Account, uid: number, onTokenRefreshed?: (a: Account) => void) {
  await ensureValidToken(account, onTokenRefreshed);
  selectedUid = uid;
  const folder = activeFolder;
  try {
    const detail = await invoke<MailDetailType>('fetch_mail_detail', { config: getImapConfig(account), folder, uid });
    if (selectedUid !== uid) return;
    selectedMail = detail;
    const m = mails.find(x => x.uid === uid);
    if (m && !m.seen) { m.seen = true; mails = mails; }
    // Preload surrounding mails
    const idx = mails.findIndex(m => m.uid === uid);
    if (idx >= 0) {
      const nearby = mails.slice(Math.max(0, idx - 15), Math.min(mails.length, idx + 16))
        .map(m => m.uid).filter(u => u !== uid);
      if (nearby.length > 0) invoke('preload_mails', { config: getImapConfig(account), folder, uids: nearby }).catch(() => {});
    }
  } catch (e) { error = String(e); }
}

async function archiveMail(account: Account, uid: number, onUndo?: () => void, onTokenRefreshed?: (a: Account) => void) {
  const prevMails = [...mails];
  mails = mails.filter(m => m.uid !== uid);
  const idx = prevMails.findIndex(m => m.uid === uid);
  const next = mails[idx] ?? mails[idx - 1];
  if (next) { selectedUid = next.uid; }
  else { selectedMail = null; selectedUid = null; }

  let aborted = false;
  let retryTimer: ReturnType<typeof setTimeout> | null = null;
  const undoFn = () => { aborted = true; if (retryTimer) clearTimeout(retryTimer); mails = prevMails; onUndo?.(); };

  const doArchive = async (attempt: number) => {
    if (aborted) return;
    try {
      await ensureValidToken(account, onTokenRefreshed);
      await invoke('archive_mail', { config: getImapConfig(account), folder: activeFolder, uid });
    } catch (e) {
      if (aborted) return;
      if (attempt < 10) retryTimer = setTimeout(() => doArchive(attempt + 1), 1000 * attempt);
      else { mails = prevMails; error = 'アーカイブ失敗: ' + String(e); }
    }
  };
  doArchive(1);
  return undoFn;
}

async function deleteMail(account: Account, uid: number, onTokenRefreshed?: (a: Account) => void) {
  await ensureValidToken(account, onTokenRefreshed);
  await invoke('delete_mail', { config: getImapConfig(account), folder: activeFolder, uid });
  mails = mails.filter(m => m.uid !== uid);
  selectedMail = null; selectedUid = null;
}

async function starMail(account: Account, uid: number, add: boolean, onTokenRefreshed?: (a: Account) => void) {
  await ensureValidToken(account, onTokenRefreshed);
  await invoke('toggle_star', { config: getImapConfig(account), folder: activeFolder, uid, add });
}

async function searchMails(account: Account, query: string, onTokenRefreshed?: (a: Account) => void) {
  if (!query.trim()) { searchResults = null; return; }
  await ensureValidToken(account, onTokenRefreshed);
  try {
    const results = await invoke<MailSummary[]>('search_mails', {
      config: getImapConfig(account), folder: activeFolder, query, limit: 100
    });
    searchResults = results;
  } catch (e) { searchResults = null; }
}

function setActiveFolder(folder: string) {
  activeFolder = folder;
  selectedMail = null; selectedUid = null;
  searchQuery = ''; searchResults = null;
}

function clearSearch() {
  searchQuery = ''; searchResults = null;
}

export function getMailStore() {
  return {
    get mails() { return mails; },
    get selectedMail() { return selectedMail; },
    get selectedUid() { return selectedUid; },
    get loading() { return loading; },
    get error() { return error; },
    get activeFolder() { return activeFolder; },
    get hasMore() { return hasMore; },
    get syncing() { return syncing; },
    get syncStatus() { return syncStatus; },
    get searchQuery() { return searchQuery; },
    get filteredMails() { return searchResults !== null ? searchResults : mails; },
    set error(v: string | null) { error = v; },
    fetchMails,
    loadMore,
    selectMail,
    archiveMail,
    deleteMail,
    starMail,
    searchMails,
    setActiveFolder,
    clearSearch,
  };
}
