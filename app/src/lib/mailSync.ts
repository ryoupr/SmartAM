import { invoke } from '@tauri-apps/api/core';
import { getImapConfig, saveSettings } from '$lib/store';
import type { MailSummary } from '$lib/types';
import type { AppSettings } from '$lib/store';

export type FolderCache = Map<string, { mails: MailSummary[]; offset: number; hasMore: boolean }>;

export async function refreshOAuthToken(settings: AppSettings, index: number): Promise<void> {
  const a = settings.accounts[index];
  if (!a || a.auth_type !== 'oauth') return;
  const now = Math.floor(Date.now() / 1000);
  if (a.token_expires_at > now + 60) return;
  const tokens = await invoke<{ access_token: string; refresh_token: string; expires_at: number }>(
    'google_oauth_refresh', { refreshToken: a.refresh_token }
  );
  a.access_token = tokens.access_token;
  a.token_expires_at = tokens.expires_at;
  settings.accounts[index] = { ...a };
  await saveSettings(settings);
}

export async function fetchMailPage(
  settings: AppSettings,
  folder: string,
  offset: number,
  limit: number
): Promise<[MailSummary[], number]> {
  const a = settings.accounts[settings.activeAccountIndex];
  if (!a) return [[], 0];
  return invoke<[MailSummary[], number]>('fetch_mail_page', { config: getImapConfig(a), folder, offset, limit });
}

export async function fetchNewMailsSince(
  settings: AppSettings,
  folder: string,
  sinceUid: number
): Promise<MailSummary[]> {
  const a = settings.accounts[settings.activeAccountIndex];
  if (!a) return [];
  return invoke<MailSummary[]>('fetch_new_mails', { config: getImapConfig(a), folder, sinceUid });
}

export async function prefetchAllFolders(
  settings: AppSettings,
  folderCache: FolderCache,
  pageSize: number,
  trace: (tag: string, msg: string) => void
): Promise<void> {
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
            const tokens = await invoke<{ access_token: string; refresh_token: string; expires_at: number }>(
              'google_oauth_refresh', { refreshToken: a.refresh_token }
            );
            a.access_token = tokens.access_token;
            a.token_expires_at = tokens.expires_at;
            settings.accounts[i] = { ...a };
            await saveSettings(settings);
          }
        }
        const [result, total] = await invoke<[MailSummary[], number]>(
          'fetch_mail_page', { config: getImapConfig(a), folder, offset: 0, limit: pageSize * 2 }
        );
        folderCache.set(key, { mails: result, offset: result.length, hasMore: result.length < total });
        trace('PREFETCH', `${a.email}/${folder}: ${result.length}/${total}`);
      } catch (e) { trace('PREFETCH', `${a.email}/${folder} failed: ${e}`); }
    }
  }
}

export async function sendNotificationForNewMails(
  settings: AppSettings,
  newMails: MailSummary[],
  allMails: MailSummary[],
  trace: (tag: string, msg: string) => void
): Promise<void> {
  const a = settings.accounts[settings.activeAccountIndex];
  if (!a?.notifications) return;
  try {
    const { sendNotification } = await import('@tauri-apps/plugin-notification');
    const latest = newMails[0];
    const body = newMails.length === 1
      ? `${latest.from}: ${latest.subject}`
      : `${latest.from}: ${latest.subject} 他${newMails.length - 1}件`;
    const opts: Parameters<typeof sendNotification>[0] = { title: 'SmartAM', body };
    if (!a.notificationSound) opts.silent = true;
    sendNotification(opts);
    if (a.notificationBadge) {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const unread = allMails.filter(m => !m.seen).length;
      await getCurrentWindow().setBadgeCount(unread || undefined);
    }
  } catch (e) { trace('NOTIFY', `sendNotification error: ${e}`); }
}
