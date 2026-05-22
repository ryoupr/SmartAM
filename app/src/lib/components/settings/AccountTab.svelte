<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { DEFAULT_ACCOUNT_EXTRAS, type AppSettings, type Account } from '$lib/store';

  let { settings = $bindable() }: { settings: AppSettings } = $props();

  let activeSubTab = $state('accounts');
  let addingAccount = $state(false);
  let testResult = $state('');
  let oauthLoading = $state(false);
  let oauthError = $state('');
  let googleCalMap: Record<string, string[]> = $state({});
  let googleCalLoading = $state(false);

  let newAccount: Account = $state({
    email: '', auth_type: 'password' as const, password: '',
    access_token: '', refresh_token: '', token_expires_at: 0,
    imap_host: '', imap_port: 993, smtp_host: '', smtp_port: 587,
    signature: '', ...structuredClone(DEFAULT_ACCOUNT_EXTRAS)
  });

  const PRESETS: Record<string, { imap_host: string; smtp_host: string }> = {
    'gmail.com': { imap_host: 'imap.gmail.com', smtp_host: 'smtp.gmail.com' },
    'outlook.com': { imap_host: 'outlook.office365.com', smtp_host: 'smtp.office365.com' },
    'hotmail.com': { imap_host: 'outlook.office365.com', smtp_host: 'smtp.office365.com' },
    'yahoo.co.jp': { imap_host: 'imap.mail.yahoo.co.jp', smtp_host: 'smtp.mail.yahoo.co.jp' },
  };

  const perAccountTabs = [
    { suffix: 'signature', label: '署名' },
    { suffix: 'calendar', label: 'カレンダー連携' },
    { suffix: 'notification', label: '通知' },
  ];

  function tabAccIdx(): number { const m = activeSubTab.match(/:(\d+)$/); return m ? +m[1] : 0; }
  function tabType(): string { return activeSubTab.replace(/:\d+$/, ''); }

  function onEmailInput() {
    const domain = newAccount.email.split('@')[1]?.toLowerCase();
    if (domain && PRESETS[domain]) {
      if (!newAccount.imap_host) newAccount.imap_host = PRESETS[domain].imap_host;
      if (!newAccount.smtp_host) newAccount.smtp_host = PRESETS[domain].smtp_host;
    }
  }

  async function testImapConnection() {
    testResult = '接続テスト中...';
    try {
      const r = await invoke<string>('test_imap_connection', {
        config: { email: newAccount.email, auth_type: newAccount.auth_type || 'password', password: newAccount.password, access_token: newAccount.access_token || '', imap_host: newAccount.imap_host, imap_port: newAccount.imap_port }
      });
      testResult = `✅ ${r}`;
    } catch (e) { testResult = `❌ ${e}`; }
  }

  async function handleGoogleOAuth() {
    oauthLoading = true; oauthError = '';
    try {
      const tokens = await invoke<{ access_token: string; refresh_token: string; expires_at: number; email: string }>('google_oauth_login');
      const account: Account = {
        email: tokens.email, auth_type: 'oauth', password: '',
        access_token: tokens.access_token, refresh_token: tokens.refresh_token,
        token_expires_at: tokens.expires_at,
        imap_host: 'imap.gmail.com', imap_port: 993,
        smtp_host: 'smtp.gmail.com', smtp_port: 587,
        signature: '', ...structuredClone(DEFAULT_ACCOUNT_EXTRAS),
      };
      settings.accounts = [...settings.accounts, account];
      addingAccount = false;
    } catch (e) { oauthError = `OAuth エラー: ${e}`; }
    finally { oauthLoading = false; }
  }

  function addAccount() {
    settings.accounts = [...settings.accounts, JSON.parse(JSON.stringify(newAccount))];
    addingAccount = false;
    newAccount = { email: '', auth_type: 'password' as const, password: '', access_token: '', refresh_token: '', token_expires_at: 0, imap_host: '', imap_port: 993, smtp_host: '', smtp_port: 587, signature: '', ...structuredClone(DEFAULT_ACCOUNT_EXTRAS) };
    testResult = ''; oauthError = '';
  }

  function removeAccount(i: number) {
    settings.accounts = settings.accounts.filter((_, idx) => idx !== i);
    if (settings.activeAccountIndex >= settings.accounts.length) settings.activeAccountIndex = 0;
  }

  async function fetchGoogleCalendars() {
    const acc = settings.accounts[tabAccIdx()];
    if (!acc || acc.auth_type !== 'oauth' || !acc.access_token) return;
    googleCalLoading = true;
    try {
      const cals = await invoke<string[]>('list_google_calendars', { accessToken: acc.access_token });
      googleCalMap[acc.email] = cals;
      if (cals.length > 0 && (!acc.calendar.calendarName || !cals.includes(acc.calendar.calendarName))) {
        acc.calendar.calendarName = cals[0];
      }
      acc.calendar.googleConnected = true;
    } catch { googleCalMap[acc.email] = []; }
    finally { googleCalLoading = false; }
  }
</script>

<div class="account-layout">
  <nav class="sub-nav">
    <button class="ni" class:active={activeSubTab === 'accounts'} onclick={() => activeSubTab = 'accounts'}>アカウント管理</button>
    {#if settings.accounts.length > 0}
      <div class="nav-sep"></div>
      {#each settings.accounts as acc, i}
        <div class="nav-acc">{acc.email}</div>
        {#each perAccountTabs as pt}
          <button class="ni sub" class:active={activeSubTab === `${pt.suffix}:${i}`} onclick={() => activeSubTab = `${pt.suffix}:${i}`}>{pt.label}</button>
        {/each}
      {/each}
    {/if}
  </nav>

  <div class="sub-content">
    {#if activeSubTab === 'accounts'}
      <h3>メールアカウント管理</h3>
      <button class="btn-add" onclick={() => addingAccount = true}>+ アカウントを追加</button>
      {#if addingAccount}
        <div class="card">
          <div class="oauth-section">
            <button class="btn-google" onclick={handleGoogleOAuth} disabled={oauthLoading}>
              {#if oauthLoading}認証中...{:else}Google でログイン{/if}
            </button>
            {#if newAccount.auth_type === 'oauth'}
              <span class="oauth-badge">OAuth 認証済み ({newAccount.email})</span>
            {/if}
          </div>
          {#if oauthError}<div class="tr" style="color:var(--red)">{oauthError}</div>{/if}
          {#if newAccount.auth_type !== 'oauth'}
            <div class="divider"><span>または手動設定</span></div>
            <label class="fl">メールアドレス <input bind:value={newAccount.email} oninput={onEmailInput} placeholder="user@example.com" /></label>
            <label class="fl">パスワード <input type="password" bind:value={newAccount.password} /></label>
            <label class="fl">IMAPホスト <input bind:value={newAccount.imap_host} placeholder="imap.gmail.com" /></label>
            <label class="fl">IMAPポート <input type="number" bind:value={newAccount.imap_port} /></label>
            <label class="fl">SMTPホスト <input bind:value={newAccount.smtp_host} placeholder="smtp.gmail.com" /></label>
            <label class="fl">SMTPポート <input type="number" bind:value={newAccount.smtp_port} /></label>
          {/if}
          <div class="row">
            <button class="btn-sm gb" onclick={testImapConnection}>接続テスト</button>
            <button class="btn-sm pu" onclick={addAccount} disabled={!newAccount.email}>追加</button>
            <button class="btn-sm" onclick={() => addingAccount = false}>キャンセル</button>
          </div>
          {#if testResult}<div class="tr">{testResult}</div>{/if}
        </div>
      {/if}
      {#each settings.accounts as acc, i}
        <div class="card" class:ac={i === settings.activeAccountIndex}>
          <div class="ch"><span class="dot g"></span> {acc.email} {#if acc.auth_type === 'oauth'}<span class="oauth-badge-sm">OAuth</span>{/if}</div>
          <div class="cm">IMAP: {acc.imap_host}:{acc.imap_port} | SMTP: {acc.smtp_host}:{acc.smtp_port}</div>
          <div class="row">
            {#if i !== settings.activeAccountIndex}<button class="btn-sm" onclick={() => settings.activeAccountIndex = i}>メインに設定</button>{/if}
            <button class="btn-sm dn" onclick={() => removeAccount(i)}>削除</button>
          </div>
        </div>
      {/each}

    {:else if tabType() === 'signature'}
      {#key activeSubTab}
      {@const acc = settings.accounts[tabAccIdx()]}
      <h3>{acc.email} - 署名管理</h3>
      <label class="fl">署名テキスト</label>
      <textarea class="sig" rows="6" bind:value={acc.signature}></textarea>
      <div class="preview-label">プレビュー:</div>
      <div class="sig-preview">{acc.signature || '（署名未設定）'}</div>
      {/key}

    {:else if tabType() === 'calendar'}
      {#key activeSubTab}
      {@const acc = settings.accounts[tabAccIdx()]}
      <h3>{acc.email} - カレンダー連携</h3>
      <div class="card" class:ac={acc.calendar.provider === 'apple'}>
        <div class="ch" style="cursor:pointer" onclick={() => acc.calendar.provider = 'apple'}>
          <span class="radio" class:on={acc.calendar.provider === 'apple'}></span>
          {acc.calendar.provider === 'apple' ? '🟢' : '⚪'} Apple Calendar
        </div>
        {#if acc.calendar.provider === 'apple'}
          <div class="cm">接続済み</div>
          <label class="fl">カレンダー <input bind:value={acc.calendar.calendarName} /></label>
        {/if}
      </div>
      <div class="card" class:ac={acc.calendar.provider === 'google'}>
        <div class="ch" style="cursor:pointer" onclick={() => { acc.calendar.provider = 'google'; if (!googleCalMap[acc.email]?.length) fetchGoogleCalendars(); }}>
          <span class="radio" class:on={acc.calendar.provider === 'google'}></span>
          {acc.calendar.provider === 'google' ? '🟢' : '⚪'} Google Calendar
        </div>
        {#if acc.calendar.provider === 'google'}
          {#if acc.auth_type !== 'oauth'}
            <div class="cm" style="color:var(--overlay)">このアカウントはGoogle OAuthで追加されていません</div>
          {:else if googleCalLoading}
            <div class="cm">カレンダー一覧を取得中...</div>
          {:else if googleCalMap[acc.email]?.length}
            <div class="cm" style="color:var(--green)">接続済み</div>
            <label class="fl">デフォルトカレンダー
              <select bind:value={acc.calendar.calendarName}>
                {#each googleCalMap[acc.email] as cal}<option value={cal}>{cal}</option>{/each}
              </select>
            </label>
          {:else}
            <button class="btn-sm pu" onclick={fetchGoogleCalendars}>カレンダー一覧を取得</button>
          {/if}
        {/if}
      </div>
      {/key}

    {:else if tabType() === 'notification'}
      {#key activeSubTab}
      {@const acc = settings.accounts[tabAccIdx()]}
      <h3>{acc.email} - 通知設定</h3>
      <label class="cb"><input type="checkbox" bind:checked={acc.notifications} /> 新着メール通知</label>
      <label class="cb"><input type="checkbox" bind:checked={acc.notificationSound} /> サウンド</label>
      <label class="cb"><input type="checkbox" bind:checked={acc.notificationBadge} /> バッジ表示（Dock）</label>
      <label class="fl">同期間隔
        <select bind:value={acc.syncInterval}>
          <option value={1}>1分</option><option value={5}>5分</option><option value={15}>15分</option><option value={30}>30分</option>
        </select>
      </label>
      <label class="fl">1ページあたりのメール数
        <select bind:value={settings.mailsPerPage}>
          <option value={50}>50件</option><option value={100}>100件</option><option value={200}>200件</option><option value={500}>500件</option>
        </select>
      </label>
      {/key}
    {/if}
  </div>
</div>

<style>
  .account-layout { display:flex;height:100%;gap:0 }
  .sub-nav { width:160px;background:var(--mantle);border-right:1px solid var(--surface1);padding:8px 0;flex-shrink:0 }
  .sub-content { flex:1;padding:0 0 0 16px;overflow-y:auto }
  .ni { display:block;width:100%;padding:8px 16px;border:none;background:none;color:var(--subtext);font-size:11px;cursor:pointer;text-align:left }
  .ni.active { background:var(--surface1);color:var(--mauve) }
  .ni.sub { padding-left:28px;font-size:10px }
  .nav-sep { height:1px;background:var(--surface1);margin:6px 8px }
  .nav-acc { padding:6px 12px 2px;font-size:10px;font-weight:700;color:var(--text);white-space:nowrap;overflow:hidden;text-overflow:ellipsis }
  .fl { display:block;color:var(--overlay);font-size:10px;margin:6px 0 2px }
  .fl input,.fl select { display:block;width:100%;padding:5px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:11px;margin-top:2px }
  .card { background:var(--mantle);border:1px solid var(--surface1);border-radius:6px;padding:12px;margin:8px 0 }
  .card.ac { border-color:var(--green) }
  .ch { font-size:12px;font-weight:700;margin-bottom:4px;display:flex;align-items:center;gap:6px }
  .cm { color:var(--overlay);font-size:10px;margin-bottom:6px }
  .row { display:flex;gap:6px;margin-top:6px }
  .dot { width:8px;height:8px;border-radius:50% }
  .dot.g { background:var(--green) }
  .radio { width:14px;height:14px;border-radius:50%;border:2px solid var(--surface1);flex-shrink:0 }
  .radio.on { border-color:var(--green);background:var(--green) }
  .btn-sm { padding:4px 12px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px;cursor:pointer }
  .btn-sm.dn { color:var(--red);border-color:var(--red) }
  .btn-sm.pu { background:var(--mauve);color:var(--base);border:none }
  .btn-sm.gb { border-color:var(--green);color:var(--green) }
  .btn-add { padding:6px 16px;border-radius:6px;border:none;background:var(--mauve);color:var(--base);font-weight:700;font-size:11px;cursor:pointer;margin-bottom:12px }
  .tr { margin-top:6px;font-size:11px }
  .sig { width:100%;padding:8px;border-radius:6px;border:1px solid var(--surface1);background:var(--mantle);color:var(--text);font-size:11px;font-family:inherit }
  .preview-label { color:var(--overlay);font-size:9px;margin-top:8px }
  .sig-preview { padding:8px;border-radius:6px;border:1px dashed var(--surface1);background:var(--base);color:var(--overlay);font-size:11px;white-space:pre-line;min-height:40px;margin-top:4px }
  .cb { display:block;font-size:11px;margin:6px 0;color:var(--text) }
  .divider { display:flex;align-items:center;gap:8px;margin:12px 0 8px;color:var(--overlay);font-size:9px }
  .divider::before,.divider::after { content:'';flex:1;height:1px;background:var(--surface1) }
  .oauth-section { display:flex;align-items:center;gap:8px;margin:8px 0 }
  .btn-google { padding:6px 16px;border-radius:6px;border:1px solid #dadce0;background:#fff;color:#3c4043;font-size:11px;font-weight:600;cursor:pointer;display:flex;align-items:center;gap:6px }
  .btn-google:hover { background:#f8f9fa }
  .btn-google:disabled { opacity:.6;cursor:not-allowed }
  .oauth-badge { display:inline-block;padding:2px 8px;border-radius:4px;background:#1e3a2e;color:var(--green);font-size:10px;font-weight:700 }
  .oauth-badge-sm { display:inline-block;padding:1px 6px;border-radius:3px;background:#1e3a2e;color:var(--green);font-size:9px;font-weight:700;margin-left:4px }
  h3 { font-size:14px;margin-bottom:12px }
</style>
