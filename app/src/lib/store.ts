import { load } from '@tauri-apps/plugin-store';
import { invoke } from '@tauri-apps/api/core';
import type { AccountConfig } from './types';

export interface CalendarSettings {
  provider: 'apple' | 'google';
  calendarName: string;
  googleConnected: boolean;
}

export interface Account {
  email: string;
  auth_type: 'password' | 'oauth';
  password: string;
  access_token: string;
  refresh_token: string;
  token_expires_at: number;
  imap_host: string;
  imap_port: number;
  smtp_host: string;
  smtp_port: number;
  signature: string;
  notifications: boolean;
  notificationSound: boolean;
  notificationBadge: boolean;
  notificationFolders: string[];
  syncInterval: number;
  calendar: CalendarSettings;
}

export type LlmProvider = 'ollama' | 'openai' | 'anthropic' | 'bedrock' | 'gemini';

export interface LlmSettings {
  activeProvider: LlmProvider;
  ollama: { base_url: string; model: string };
  openai: { api_key: string; model: string };
  anthropic: { api_key: string; model: string };
  bedrock: { auth_mode: 'iam' | 'api_key'; region: string; access_key: string; secret_key: string; api_key: string; model: string };
  gemini: { api_key: string; model: string };
}

export interface ShortcutMap { [action: string]: string }

export const DEFAULT_SHORTCUTS: ShortcutMap = {
  nextMail: 'j', prevMail: 'k', openMail: 'Enter', backToList: 'u',
  reply: 'r', forward: 'f', archive: 'a', delete: '#', star: 's', undo: 'z',
  aiSummary: 'y', aiDraft: 'd', aiTranslate: 't', aiCalendar: 'l',
  compose: 'c', search: '/', help: '?',
  acceptInvite: 'A', declineInvite: 'D',
  goInbox: 'g i', goStarred: 'g s', goSent: 'g t', goDrafts: 'g d', goAll: 'g a',
};

export type LogLevel = 'error' | 'warn' | 'info' | 'debug' | 'trace';

export interface AppSettings {
  accounts: Account[];
  activeAccountIndex: number;
  llm: LlmSettings;
  mailsPerPage: number;
  aiBudgetLimitUsd: number;
  shortcuts: ShortcutMap;
  dateFormat: string;
  timezone: string;
  logLevel: LogLevel;
  theme: 'dark' | 'light';
}

export const DEFAULT_ACCOUNT_EXTRAS = {
  notifications: true,
  notificationSound: true,
  notificationBadge: false,
  notificationFolders: ['INBOX'],
  syncInterval: 5,
  calendar: { provider: 'google' as const, calendarName: '', googleConnected: false },
};

export const DEFAULTS: AppSettings = {
  accounts: [],
  activeAccountIndex: 0,
  llm: {
    activeProvider: 'ollama',
    ollama: { base_url: 'http://localhost:11434', model: 'llama3' },
    openai: { api_key: '', model: 'gpt-4o' },
    anthropic: { api_key: '', model: 'claude-3-5-sonnet-20241022' },
    bedrock: { auth_mode: 'api_key', region: 'us-east-1', access_key: '', secret_key: '', api_key: '', model: 'us.anthropic.claude-sonnet-4-20250514-v1:0' },
    gemini: { api_key: '', model: 'gemini-1.5-pro' },
  },
  mailsPerPage: 200,
  aiBudgetLimitUsd: 0,
  shortcuts: { ...DEFAULT_SHORTCUTS },
  dateFormat: 'YYYY/MM/DD HH:mm:ss',
  timezone: 'Asia/Tokyo',
  logLevel: 'info' as LogLevel,
  theme: 'light' as const,
};

let _store: Awaited<ReturnType<typeof load>> | null = null;

async function getStore() {
  if (!_store) _store = await load('settings.json', { defaults: {} });
  return _store;
}

export async function loadSettings(): Promise<AppSettings> {
  const store = await getStore();
  const data = await store.get<any>('settings');
  if (!data || typeof data !== 'object' || !Array.isArray(data.accounts)) {
    return structuredClone(DEFAULTS);
  }
  const merged = { ...structuredClone(DEFAULTS), ...data };
  if (!merged.shortcuts) merged.shortcuts = { ...DEFAULT_SHORTCUTS };
  else merged.shortcuts = { ...DEFAULT_SHORTCUTS, ...merged.shortcuts };
  // migrate: move global notification/calendar into each account
  for (const acc of merged.accounts) {
    if (acc.notifications === undefined) Object.assign(acc, structuredClone(DEFAULT_ACCOUNT_EXTRAS));
    if (!acc.calendar) acc.calendar = structuredClone(DEFAULT_ACCOUNT_EXTRAS.calendar);
  }
  return merged;
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  const store = await getStore();
  const json = JSON.stringify(settings);
  const plain = JSON.parse(json);
  invoke('frontend_trace', { tag: 'SAVE', msg: `accounts: ${plain.accounts?.length}, keys: ${Object.keys(plain).join(',')}` }).catch(() => {});
  await store.set('settings', plain);
  await store.save();
  const verify = await store.get<AppSettings>('settings');
  invoke('frontend_trace', { tag: 'SAVE', msg: `verify accounts: ${verify?.accounts?.length}` }).catch(() => {});
}

export function getImapConfig(account: Account): AccountConfig {
  return {
    email: account.email,
    auth_type: account.auth_type || 'password',
    password: account.password,
    access_token: account.access_token || '',
    imap_host: account.imap_host,
    imap_port: account.imap_port,
  };
}

export function getSmtpConfig(account: Account) {
  return {
    email: account.email,
    auth_type: account.auth_type || 'password',
    password: account.password,
    access_token: account.access_token || '',
    smtp_host: account.smtp_host,
    smtp_port: account.smtp_port,
  };
}

export function getLlmConfig(llm: LlmSettings): { base_url: string; model: string; api_key: string } {
  switch (llm.activeProvider) {
    case 'ollama': return { base_url: llm.ollama.base_url, model: `ollama/${llm.ollama.model}`, api_key: '' };
    case 'openai': return { base_url: 'https://api.openai.com/v1', model: llm.openai.model, api_key: llm.openai.api_key };
    case 'anthropic': return { base_url: 'https://api.anthropic.com/v1', model: llm.anthropic.model, api_key: llm.anthropic.api_key };
    case 'bedrock':
      if (llm.bedrock.auth_mode === 'api_key' && llm.bedrock.api_key) {
        return { base_url: `https://bedrock-runtime.${llm.bedrock.region}.amazonaws.com`, model: llm.bedrock.model, api_key: llm.bedrock.api_key };
      }
      // IAM auth: requires SigV4 signing in Rust backend
      return { base_url: `bedrock://${llm.bedrock.region}`, model: llm.bedrock.model, api_key: '' };
    case 'gemini': return { base_url: 'https://generativelanguage.googleapis.com/v1beta/openai', model: llm.gemini.model, api_key: llm.gemini.api_key };
  }
}

export function formatMailDate(raw: string, format: string, tz: string): string {
  const d = new Date(raw);
  if (isNaN(d.getTime())) return raw;
  try {
    const parts = new Intl.DateTimeFormat('en-US', {
      timeZone: tz, year: 'numeric', month: '2-digit', day: '2-digit',
      hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false,
    }).formatToParts(d);
    const p = (type: string) => parts.find(p => p.type === type)?.value ?? '';
    const Y = p('year'), M = p('month'), D = p('day'), H = p('hour'), m = p('minute'), s = p('second');
    // Today → show time only
    const nowParts = new Intl.DateTimeFormat('en-US', {
      timeZone: tz, year: 'numeric', month: '2-digit', day: '2-digit',
    }).formatToParts(new Date());
    const np = (type: string) => nowParts.find(p => p.type === type)?.value ?? '';
    if (Y === np('year') && M === np('month') && D === np('day')) return `${H}:${m}`;
    return format
      .replace('YYYY', Y).replace('MM', M).replace('DD', D)
      .replace('HH', H).replace('mm', m).replace('ss', s);
  } catch { return raw; }
}

// --- Keychain Integration ---

export async function saveCredentialToKeychain(account: string, keyType: string, secret: string): Promise<void> {
  if (!secret) return;
  await invoke('store_keychain', { account, keyType, secret });
}

export async function getCredentialFromKeychain(account: string, keyType: string): Promise<string> {
  try {
    return await invoke<string>('get_keychain', { account, keyType });
  } catch {
    return '';
  }
}

export async function deleteCredentialFromKeychain(account: string, keyType: string): Promise<void> {
  await invoke('delete_keychain', { account, keyType }).catch(() => {});
}

/**
 * Migrate credentials from settings.json to Keychain.
 * Call once on app startup. After migration, credentials are cleared from settings.
 */
export async function migrateCredentialsToKeychain(settings: AppSettings): Promise<boolean> {
  let migrated = false;
  for (const acc of settings.accounts) {
    if (acc.password) {
      await saveCredentialToKeychain(acc.email, 'imap-password', acc.password);
      acc.password = '';
      migrated = true;
    }
    if (acc.access_token) {
      await saveCredentialToKeychain(acc.email, 'oauth-access-token', acc.access_token);
      acc.access_token = '';
      migrated = true;
    }
    if (acc.refresh_token) {
      await saveCredentialToKeychain(acc.email, 'oauth-refresh-token', acc.refresh_token);
      acc.refresh_token = '';
      migrated = true;
    }
  }
  // LLM API keys
  const llm = settings.llm;
  if (llm.openai.api_key) {
    await saveCredentialToKeychain('openai', 'llm-api-key', llm.openai.api_key);
    llm.openai.api_key = '';
    migrated = true;
  }
  if (llm.anthropic.api_key) {
    await saveCredentialToKeychain('anthropic', 'llm-api-key', llm.anthropic.api_key);
    llm.anthropic.api_key = '';
    migrated = true;
  }
  if (llm.gemini.api_key) {
    await saveCredentialToKeychain('gemini', 'llm-api-key', llm.gemini.api_key);
    llm.gemini.api_key = '';
    migrated = true;
  }
  if (llm.bedrock.api_key) {
    await saveCredentialToKeychain('bedrock', 'llm-api-key', llm.bedrock.api_key);
    llm.bedrock.api_key = '';
    migrated = true;
  }
  if (llm.bedrock.access_key) {
    await saveCredentialToKeychain('bedrock', 'aws-access-key', llm.bedrock.access_key);
    llm.bedrock.access_key = '';
    migrated = true;
  }
  if (llm.bedrock.secret_key) {
    await saveCredentialToKeychain('bedrock', 'aws-secret-key', llm.bedrock.secret_key);
    llm.bedrock.secret_key = '';
    migrated = true;
  }
  return migrated;
}

/**
 * Load credentials from Keychain back into settings object (for runtime use).
 * Call after loadSettings() to hydrate credentials.
 */
export async function hydrateCredentialsFromKeychain(settings: AppSettings): Promise<void> {
  for (const acc of settings.accounts) {
    if (!acc.password) acc.password = await getCredentialFromKeychain(acc.email, 'imap-password');
    if (!acc.access_token) acc.access_token = await getCredentialFromKeychain(acc.email, 'oauth-access-token');
    if (!acc.refresh_token) acc.refresh_token = await getCredentialFromKeychain(acc.email, 'oauth-refresh-token');
  }
  const llm = settings.llm;
  if (!llm.openai.api_key) llm.openai.api_key = await getCredentialFromKeychain('openai', 'llm-api-key');
  if (!llm.anthropic.api_key) llm.anthropic.api_key = await getCredentialFromKeychain('anthropic', 'llm-api-key');
  if (!llm.gemini.api_key) llm.gemini.api_key = await getCredentialFromKeychain('gemini', 'llm-api-key');
  if (!llm.bedrock.api_key) llm.bedrock.api_key = await getCredentialFromKeychain('bedrock', 'llm-api-key');
  if (!llm.bedrock.access_key) llm.bedrock.access_key = await getCredentialFromKeychain('bedrock', 'aws-access-key');
  if (!llm.bedrock.secret_key) llm.bedrock.secret_key = await getCredentialFromKeychain('bedrock', 'aws-secret-key');
}