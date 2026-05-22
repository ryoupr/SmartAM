import { invoke } from '@tauri-apps/api/core';
import { getImapConfig, getSmtpConfig } from '$lib/store';
import type { AppSettings } from '$lib/store';

type Acc = NonNullable<ReturnType<() => AppSettings['accounts'][number]>>;

export async function archiveMail(a: Acc, folder: string, uid: number, attempt = 1, aborted: () => boolean): Promise<void> {
  if (aborted()) return;
  try {
    await invoke('archive_mail', { config: getImapConfig(a), folder, uid });
  } catch (e) {
    if (aborted()) return;
    if (attempt < 10) setTimeout(() => archiveMail(a, folder, uid, attempt + 1, aborted), 1000 * attempt);
    else throw e;
  }
}

export async function deleteMail(a: Acc, folder: string, uid: number): Promise<void> {
  await invoke('delete_mail', { config: getImapConfig(a), folder, uid });
}

export async function toggleStar(a: Acc, folder: string, uid: number, add: boolean): Promise<void> {
  await invoke('toggle_star', { config: getImapConfig(a), folder, uid, add });
}

export async function downloadAttachment(a: Acc, folder: string, uid: number, partIndex: number, filename: string): Promise<string> {
  return invoke<string>('download_attachment', { config: getImapConfig(a), folder, uid, partIndex, filename });
}

export async function sendMail(
  a: Acc,
  data: { to: string; cc: string; bcc: string; subject: string; body: string },
  attachmentPaths: string[]
): Promise<void> {
  const toArr = data.to.split(',').map(s => s.trim()).filter(Boolean);
  const ccArr = data.cc.split(',').map(s => s.trim()).filter(Boolean);
  const bccArr = data.bcc.split(',').map(s => s.trim()).filter(Boolean);
  if (attachmentPaths.length > 0) {
    await invoke('send_mail_with_attachments', { config: getSmtpConfig(a), to: toArr, cc: ccArr, bcc: bccArr, subject: data.subject, body: data.body, attachmentPaths });
  } else {
    await invoke('send_mail', { config: getSmtpConfig(a), to: toArr, cc: ccArr, bcc: bccArr, subject: data.subject, body: data.body });
  }
}
