<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { getLlmConfig, DEFAULT_ACCOUNT_EXTRAS, DEFAULT_SHORTCUTS, type AppSettings, type Account, type LlmProvider } from '$lib/store';

  let { settings, onClose, onSave }: {
    settings: AppSettings;
    onClose: () => void;
    onSave: (s: AppSettings) => void;
  } = $props();

  let local: AppSettings = $state(JSON.parse(JSON.stringify(settings)));
  let activeTab = $state('account');
  let addingAccount = $state(false);
  let testResult = $state('');
  let llmTestResult = $state('');
  let newAccount: Account = $state({ email: '', auth_type: 'password' as const, password: '', access_token: '', refresh_token: '', token_expires_at: 0, imap_host: '', imap_port: 993, smtp_host: '', smtp_port: 587, signature: '', ...structuredClone(DEFAULT_ACCOUNT_EXTRAS) });
  let oauthLoading = $state(false);
  let oauthError = $state('');
  let bedrockModels: string[] = $state([]);
  let bedrockModelsLoading = $state(false);
  let googleCalMap: Record<string, string[]> = $state({});
  let googleCalLoading = $state(false);
  let recordingAction: string | null = $state(null);
  let scConflict = $state('');

  type UsageSummary = { month: string; models: { model: string; input_tokens: number; output_tokens: number; cost_usd: number; requests: number }[]; total_cost_usd: number; budget_limit_usd: number; budget_remaining_usd: number };
  let aiUsage: UsageSummary | null = $state(null);

  async function fetchAiUsage() {
    try { aiUsage = await invoke<UsageSummary>('get_ai_usage'); } catch { aiUsage = null; }
  }

  async function fetchGoogleCalendars() {
    const acc = local.accounts[tabAccIdx()];
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

  async function fetchBedrockModels() {
    if (!local.llm.bedrock.api_key || !local.llm.bedrock.region) return;
    bedrockModelsLoading = true;
    try {
      bedrockModels = await invoke<string[]>('list_bedrock_models', { region: local.llm.bedrock.region, apiKey: local.llm.bedrock.api_key });
    } catch (e) { bedrockModels = []; }
    finally { bedrockModelsLoading = false; }
  }

  const PRESETS: Record<string, { imap_host: string; smtp_host: string }> = {
    'gmail.com': { imap_host: 'imap.gmail.com', smtp_host: 'smtp.gmail.com' },
    'outlook.com': { imap_host: 'outlook.office365.com', smtp_host: 'smtp.office365.com' },
    'hotmail.com': { imap_host: 'outlook.office365.com', smtp_host: 'smtp.office365.com' },
    'yahoo.co.jp': { imap_host: 'imap.mail.yahoo.co.jp', smtp_host: 'smtp.mail.yahoo.co.jp' },
  };

  function onEmailInput() {
    const domain = newAccount.email.split('@')[1]?.toLowerCase();
    if (domain && PRESETS[domain]) {
      if (!newAccount.imap_host) newAccount.imap_host = PRESETS[domain].imap_host;
      if (!newAccount.smtp_host) newAccount.smtp_host = PRESETS[domain].smtp_host;
    }
  }

  const globalTabs = [
    { id: 'account', label: 'メールアカウント' },
    { id: 'llm', label: 'LLMプロバイダー' },
    { id: 'ai_usage', label: 'AI 利用状況' },
    { id: 'shortcuts', label: 'キーボードショートカット' },
    { id: 'display', label: '表示設定' },
  ];
  const perAccountTabs = [
    { suffix: 'signature', label: '署名' },
    { suffix: 'calendar', label: 'カレンダー連携' },
    { suffix: 'notification', label: '通知' },
  ];
  function tabAccIdx(): number { const m = activeTab.match(/:(\d+)$/); return m ? +m[1] : 0; }
  function tabType(): string { return activeTab.replace(/:\d+$/, ''); }

  const providers: { id: LlmProvider; label: string; icon: string }[] = [
    { id: 'ollama', label: 'Ollama（ローカル）', icon: '🟢' },
    { id: 'openai', label: 'OpenAI', icon: '⚪' },
    { id: 'anthropic', label: 'Anthropic', icon: '⚪' },
    { id: 'bedrock', label: 'AWS Bedrock', icon: '⚪' },
    { id: 'gemini', label: 'Google Gemini', icon: '⚪' },
  ];

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
    oauthLoading = true;
    oauthError = '';
    try {
      const tokens = await invoke<{ access_token: string; refresh_token: string; expires_at: number; email: string }>('google_oauth_login');
      const account: Account = {
        email: tokens.email,
        auth_type: 'oauth',
        password: '',
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_expires_at: tokens.expires_at,
        imap_host: 'imap.gmail.com',
        imap_port: 993,
        smtp_host: 'smtp.gmail.com',
        smtp_port: 587,
        signature: '',
        ...structuredClone(DEFAULT_ACCOUNT_EXTRAS),
      };
      local.accounts = [...local.accounts, account];
      addingAccount = false;
    } catch (e) { oauthError = `OAuth エラー: ${e}`; }
    finally { oauthLoading = false; }
  }

  async function testLlmConnection() {
    llmTestResult = '接続テスト中...';
    try {
      const llm = getLlmConfig(local.llm);
      await invoke<string>('ai_summarize', { llm, mailBody: 'test' });
      llmTestResult = '✅ 接続成功';
    } catch (e) { llmTestResult = `❌ ${e}`; }
  }

  function addAccount() {
    local.accounts = [...local.accounts, JSON.parse(JSON.stringify(newAccount))];
    addingAccount = false;
    newAccount = { email: '', auth_type: 'password' as const, password: '', access_token: '', refresh_token: '', token_expires_at: 0, imap_host: '', imap_port: 993, smtp_host: '', smtp_port: 587, signature: '', ...structuredClone(DEFAULT_ACCOUNT_EXTRAS) };
    testResult = '';
    oauthError = '';
  }

  function removeAccount(i: number) {
    local.accounts = local.accounts.filter((_, idx) => idx !== i);
    if (local.activeAccountIndex >= local.accounts.length) local.activeAccountIndex = 0;
  }

  function activeAcc() { return local.accounts[local.activeAccountIndex]; }
</script>

<div class="overlay" role="dialog">
  <div class="win">
    <div class="hdr"><span>設定</span><button class="x" onclick={onClose}>✕</button></div>
    <div class="body">
      <nav class="nav">
        {#each globalTabs as tab}
          <button class="ni" class:active={activeTab === tab.id} onclick={() => activeTab = tab.id}>{tab.label}</button>
        {/each}
        {#if local.accounts.length > 0}
          <div class="nav-sep"></div>
          {#each local.accounts as acc, i}
            <div class="nav-acc">{acc.email}</div>
            {#each perAccountTabs as pt}
              <button class="ni sub" class:active={activeTab === `${pt.suffix}:${i}`} onclick={() => activeTab = `${pt.suffix}:${i}`}>{pt.label}</button>
            {/each}
          {/each}
        {/if}
      </nav>
      <div class="content">

        {#if activeTab === 'account'}
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
          {#each local.accounts as acc, i}
            <div class="card" class:ac={i === local.activeAccountIndex}>
              <div class="ch"><span class="dot g"></span> {acc.email} {#if acc.auth_type === 'oauth'}<span class="oauth-badge-sm">OAuth</span>{/if}</div>
              <div class="cm">IMAP: {acc.imap_host}:{acc.imap_port} | SMTP: {acc.smtp_host}:{acc.smtp_port}</div>
              <div class="row">
                {#if i !== local.activeAccountIndex}<button class="btn-sm" onclick={() => local.activeAccountIndex = i}>メインに設定</button>{/if}
                <button class="btn-sm dn" onclick={() => removeAccount(i)}>削除</button>
              </div>
            </div>
          {/each}

        {:else if tabType() === 'signature'}
          {#key activeTab}
          {@const acc = local.accounts[tabAccIdx()]}
          <h3>{acc.email} - 署名管理</h3>
          <label class="fl">署名テキスト</label>
          <textarea class="sig" rows="6" bind:value={acc.signature}></textarea>
          <div class="preview-label">プレビュー:</div>
          <div class="sig-preview">{acc.signature || '（署名未設定）'}</div>
          {/key}

        {:else if activeTab === 'llm'}
          <h3>LLM プロバイダー設定</h3>
          {#each providers as p}
            {@const isActive = local.llm.activeProvider === p.id}
            <div class="card" class:ac={isActive}>
              <div class="ch" style="cursor:pointer" onclick={() => local.llm.activeProvider = p.id}>
                <span class="radio" class:on={isActive}></span>
                {isActive ? '🟢' : '⚪'} {p.label}
              </div>
              {#if p.id === 'ollama'}
                <label class="fl">Base URL <input bind:value={local.llm.ollama.base_url} /></label>
                <label class="fl">モデル <select bind:value={local.llm.ollama.model}>
                  <option>llama3</option><option>mistral</option><option>codellama</option><option>gemma2</option>
                </select></label>
              {:else if p.id === 'openai'}
                <label class="fl">API Key <input type="password" bind:value={local.llm.openai.api_key} placeholder="sk-..." /></label>
                <label class="fl">モデル <select bind:value={local.llm.openai.model}>
                  <option>gpt-4o</option><option>gpt-4o-mini</option><option>gpt-4-turbo</option>
                </select></label>
              {:else if p.id === 'anthropic'}
                <label class="fl">API Key <input type="password" bind:value={local.llm.anthropic.api_key} placeholder="sk-ant-..." /></label>
                <label class="fl">モデル <select bind:value={local.llm.anthropic.model}>
                  <option>claude-3-5-sonnet-20241022</option><option>claude-3-opus-20240229</option><option>claude-3-haiku-20240307</option>
                </select></label>
              {:else if p.id === 'bedrock'}
                <label class="fl">認証方式 <select bind:value={local.llm.bedrock.auth_mode}>
                  <option value="api_key">Bedrock API Key</option>
                  <option value="iam">AWS IAM (LiteLLM経由)</option>
                </select></label>
                <label class="fl">Region <select bind:value={local.llm.bedrock.region}>
                  <option>us-east-1</option><option>us-west-2</option><option>ap-northeast-1</option><option>eu-west-1</option>
                </select></label>
                {#if local.llm.bedrock.auth_mode === 'api_key'}
                  <label class="fl">Bedrock API Key <input type="password" bind:value={local.llm.bedrock.api_key} onchange={fetchBedrockModels} placeholder="Bedrock API Key を入力" /></label>
                {:else}
                  <label class="fl">Access Key <input type="password" bind:value={local.llm.bedrock.access_key} placeholder="AKIA..." /></label>
                  <label class="fl">Secret Key <input type="password" bind:value={local.llm.bedrock.secret_key} /></label>
                {/if}
                <label class="fl">モデル
                  {#if bedrockModels.length > 0}
                    <select bind:value={local.llm.bedrock.model}>
                      {#each bedrockModels as m}<option value={m}>{m}</option>{/each}
                    </select>
                  {:else}
                    <input bind:value={local.llm.bedrock.model} placeholder="us.anthropic.claude-sonnet-4-20250514-v1:0" />
                    {#if local.llm.bedrock.auth_mode === 'api_key' && local.llm.bedrock.api_key}
                      <button class="btn-sm gb" onclick={fetchBedrockModels} disabled={bedrockModelsLoading} style="margin-top:4px">
                        {bedrockModelsLoading ? '取得中...' : 'モデル一覧を取得'}
                      </button>
                    {/if}
                  {/if}
                </label>
              {:else if p.id === 'gemini'}
                <label class="fl">API Key <input type="password" bind:value={local.llm.gemini.api_key} placeholder="AIza..." /></label>
                <label class="fl">モデル <select bind:value={local.llm.gemini.model}>
                  <option>gemini-1.5-pro</option><option>gemini-1.5-flash</option><option>gemini-2.0-flash</option>
                </select></label>
              {/if}
              {#if isActive}
                <div class="row mt"><button class="btn-sm gb" onclick={testLlmConnection}>接続テスト</button></div>
                {#if llmTestResult}<div class="tr">{llmTestResult}</div>{/if}
              {/if}
            </div>
          {/each}

        {:else if activeTab === 'ai_usage'}
          <h3>AI 利用状況</h3>
          <label class="fl">月額利用上限 (USD)
            <input type="number" step="0.5" min="0" bind:value={local.aiBudgetLimitUsd} placeholder="0 = 無制限" />
          </label>
          <div class="cm" style="margin-bottom:12px">{local.aiBudgetLimitUsd > 0 ? `$${local.aiBudgetLimitUsd} を超えるとAI機能が停止します` : '上限なし（無制限）'}</div>
          <button class="btn-sm gb" onclick={fetchAiUsage} style="margin-bottom:12px">利用状況を更新</button>
          {#if aiUsage}
            <div class="card">
              <div class="ch">📊 {aiUsage.month}</div>
              {#if aiUsage.models.length === 0}
                <div class="cm">利用データなし</div>
              {:else}
                {#each aiUsage.models as m}
                  <div class="usage-row">
                    <span class="usage-model">{m.model}</span>
                    <span class="usage-detail">入力: {m.input_tokens.toLocaleString()} tokens / 出力: {m.output_tokens.toLocaleString()} tokens / {m.requests}回</span>
                    <span class="usage-cost">${m.cost_usd.toFixed(4)}</span>
                  </div>
                {/each}
                <div class="usage-total">
                  合計: <strong>${aiUsage.total_cost_usd.toFixed(4)}</strong>
                  {#if aiUsage.budget_limit_usd > 0}
                    / ${aiUsage.budget_limit_usd.toFixed(2)} （残り: ${aiUsage.budget_remaining_usd.toFixed(4)}）
                  {/if}
                </div>
                {#if aiUsage.budget_limit_usd > 0 && aiUsage.budget_remaining_usd <= 0}
                  <div class="usage-warn">AI機能は上限に達したため停止中です</div>
                {/if}
              {/if}
            </div>
          {/if}

        {:else if tabType() === 'calendar'}
          {#key activeTab}
          {@const acc = local.accounts[tabAccIdx()]}
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
          {#key activeTab}
          {@const acc = local.accounts[tabAccIdx()]}
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
            <select bind:value={local.mailsPerPage}>
              <option value={50}>50件</option><option value={100}>100件</option><option value={200}>200件</option><option value={500}>500件</option>
            </select>
          </label>
          {/key}

        {:else if activeTab === 'shortcuts'}
          <h3>キーボードショートカット</h3>
          <div class="sc-hint">キーの欄をクリックして新しいキーを入力</div>
          <button class="btn-sm dn" onclick={() => { local.shortcuts = { ...DEFAULT_SHORTCUTS }; scConflict = ''; }}>デフォルトに戻す</button>
          {#each [
            ['ナビゲーション', [['nextMail','次のメール'],['prevMail','前のメール'],['openMail','メールを開く'],['backToList','一覧に戻る']]],
            ['フォルダ移動', [['goInbox','受信トレイ'],['goStarred','スター付き'],['goSent','送信済み'],['goDrafts','下書き'],['goAll','すべてのメール']]],
            ['メール操作', [['reply','返信'],['forward','転送'],['archive','アーカイブ'],['delete','削除'],['star','スター切替'],['undo','元に戻す']]],
            ['AI機能', [['aiSummary','📝 要約'],['aiDraft','✍ 返信下書き'],['aiTranslate','🌐 翻訳'],['aiCalendar','📅 カレンダー']]],
            ['その他', [['compose','新規作成'],['search','検索'],['help','ショートカット一覧']]],
          ] as group}
            <div class="sc-cat">{group[0]}</div>
            {#each group[1] as item}
              <div class="sc-row">
                <span class="sc-label">{item[1]}</span>
                <button
                  class="sc-key" class:recording={recordingAction === item[0]}
                  onclick={() => { recordingAction = item[0]; scConflict = ''; }}
                  onkeydown={(e) => {
                    if (recordingAction !== item[0]) return;
                    e.preventDefault(); e.stopPropagation();
                    const k = e.key;
                    if (k === 'Escape') { recordingAction = null; return; }
                    const dup = Object.entries(local.shortcuts).find(([a, v]) => v === k && a !== item[0]);
                    if (dup) { scConflict = `「${k}」は既に「${dup[0]}」に割り当て済み`; return; }
                    local.shortcuts[item[0]] = k;
                    scConflict = '';
                    recordingAction = null;
                  }}
                >{recordingAction === item[0] ? '⌨ キーを押す...' : local.shortcuts[item[0]]}</button>
              </div>
            {/each}
          {/each}
          {#if scConflict}<div class="sc-conflict">⚠ {scConflict}</div>{/if}

        {:else if activeTab === 'display'}
          <h3>表示設定</h3>
          <label class="fl">日時形式
            <select bind:value={local.dateFormat}>
              <option value="YYYY/MM/DD HH:mm:ss">YYYY/MM/DD HH:mm:ss</option>
              <option value="YYYY-MM-DD HH:mm:ss">YYYY-MM-DD HH:mm:ss</option>
              <option value="YYYY/MM/DD HH:mm">YYYY/MM/DD HH:mm</option>
              <option value="MM/DD HH:mm">MM/DD HH:mm</option>
              <option value="DD/MM/YYYY HH:mm:ss">DD/MM/YYYY HH:mm:ss</option>
            </select>
          </label>
          <label class="fl">タイムゾーン
            <select bind:value={local.timezone}>
              {#each ['Asia/Tokyo','America/New_York','America/Chicago','America/Denver','America/Los_Angeles','Europe/London','Europe/Paris','Europe/Berlin','Asia/Shanghai','Asia/Singapore','Australia/Sydney','Pacific/Auckland','UTC'] as tz}
                <option value={tz}>{tz}</option>
              {/each}
            </select>
          </label>

        {/if}
      </div>
    </div>
    <div class="ftr"><button class="btn-save" onclick={() => onSave(local)}>保存</button></div>
  </div>
</div>

<style>
  .overlay { position:fixed;inset:0;background:rgba(0,0,0,.5);display:flex;align-items:center;justify-content:center;z-index:50 }
  .win { width:860px;height:620px;background:var(--base);border:1px solid var(--surface1);border-radius:8px;display:flex;flex-direction:column }
  .hdr { display:flex;justify-content:space-between;padding:10px 16px;font-weight:700;border-bottom:1px solid var(--surface1) }
  .x { background:none;border:none;color:var(--overlay);cursor:pointer;font-size:16px }
  .body { display:flex;flex:1;overflow:hidden }
  .nav { width:190px;background:var(--mantle);border-right:1px solid var(--surface1);padding:8px 0 }
  .ni { display:block;width:100%;padding:8px 16px;border:none;background:none;color:var(--subtext);font-size:11px;cursor:pointer;text-align:left }
  .ni.active { background:var(--surface1);color:var(--mauve) }
  .ni.sub { padding-left:28px;font-size:10px }
  .nav-sep { height:1px;background:var(--surface1);margin:6px 8px }
  .nav-acc { padding:6px 12px 2px;font-size:10px;font-weight:700;color:var(--text);white-space:nowrap;overflow:hidden;text-overflow:ellipsis }
  .content { flex:1;padding:16px 24px;overflow-y:auto }
  .content h3 { font-size:14px;margin-bottom:12px }
  .fl { display:block;color:var(--overlay);font-size:10px;margin:6px 0 2px }
  .fl input,.fl select { display:block;width:100%;padding:5px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:11px;margin-top:2px }
  .card { background:var(--mantle);border:1px solid var(--surface1);border-radius:6px;padding:12px;margin:8px 0 }
  .card.ac { border-color:var(--green) }
  .ch { font-size:12px;font-weight:700;margin-bottom:4px;display:flex;align-items:center;gap:6px }
  .cm { color:var(--overlay);font-size:10px;margin-bottom:6px }
  .row { display:flex;gap:6px;margin-top:6px }
  .mt { margin-top:8px }
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
  .hint { color:var(--overlay);font-size:11px }
  .fl input::placeholder,.fl select::placeholder { color:var(--surface1);opacity:1 }
  .ftr { display:flex;justify-content:flex-end;padding:10px 16px;border-top:1px solid var(--surface1) }
  .btn-save { padding:8px 24px;border-radius:6px;border:none;background:var(--mauve);color:var(--base);font-weight:700;cursor:pointer }
  .divider { display:flex;align-items:center;gap:8px;margin:12px 0 8px;color:var(--overlay);font-size:9px }
  .divider::before,.divider::after { content:'';flex:1;height:1px;background:var(--surface1) }
  .usage-row { display:flex;flex-direction:column;gap:2px;padding:6px 0;border-bottom:1px solid var(--surface1);font-size:10px }
  .usage-model { font-weight:700;font-size:10px;word-break:break-all }
  .usage-detail { color:var(--overlay);font-size:9px }
  .usage-cost { color:var(--green);font-size:11px;font-weight:700 }
  .usage-total { padding:8px 0;font-size:11px;color:var(--text) }
  .usage-warn { padding:6px 10px;border-radius:4px;background:#3a1e1e;color:var(--red);font-size:10px;font-weight:700;margin-top:4px }
  .oauth-section { display:flex;align-items:center;gap:8px;margin:8px 0 }
  .btn-google { padding:6px 16px;border-radius:6px;border:1px solid #dadce0;background:#fff;color:#3c4043;font-size:11px;font-weight:600;cursor:pointer;display:flex;align-items:center;gap:6px }
  .btn-google:hover { background:#f8f9fa }
  .btn-google:disabled { opacity:.6;cursor:not-allowed }
  .oauth-badge { display:inline-block;padding:2px 8px;border-radius:4px;background:#1e3a2e;color:var(--green);font-size:10px;font-weight:700 }
  .oauth-badge-sm { display:inline-block;padding:1px 6px;border-radius:3px;background:#1e3a2e;color:var(--green);font-size:9px;font-weight:700;margin-left:4px }
  .sc-hint { color:var(--overlay);font-size:10px;margin-bottom:8px }
  .sc-cat { color:var(--mauve);font-size:11px;font-weight:700;margin:12px 0 4px }
  .sc-row { display:flex;align-items:center;justify-content:space-between;padding:3px 0 }
  .sc-label { font-size:11px;color:var(--text) }
  .sc-key { min-width:80px;padding:4px 12px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:11px;font-weight:700;text-align:center;cursor:pointer }
  .sc-key.recording { border-color:var(--mauve);color:var(--mauve);font-weight:400;font-style:italic }
  .sc-conflict { margin-top:8px;padding:6px 10px;border-radius:4px;background:#3a1e1e;color:var(--red);font-size:10px }
</style>
