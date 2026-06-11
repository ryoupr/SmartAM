<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { getLlmConfig, type AppSettings, type LlmProvider } from '$lib/store';

  let { settings = $bindable() }: { settings: AppSettings } = $props();

  let llmTestResult = $state('');
  interface BedrockModelInfo {
    model_id: string;
    model_name: string;
    provider_name: string;
    input_modalities: string[];
    output_modalities: string[];
    input_price: number | null;
    output_price: number | null;
  }
  let bedrockModels: BedrockModelInfo[] = $state([]);
  let bedrockModelsLoading = $state(false);
  let sortCol = $state<string>('');
  let sortAsc = $state(true);

  let sortedBedrockModels = $derived.by(() => {
    let list = bedrockModels.filter(m => m.output_modalities.includes('TEXT') && !m.output_modalities.includes('SPEECH') && !m.output_modalities.includes('VIDEO'));
    if (!sortCol) return list;
    return [...list].sort((a, b) => {
      let va: any, vb: any;
      switch (sortCol) {
        case 'provider': va = a.provider_name; vb = b.provider_name; break;
        case 'model': va = a.model_name; vb = b.model_name; break;
        case 'input': va = a.input_price ?? 9999; vb = b.input_price ?? 9999; break;
        case 'output': va = a.output_price ?? 9999; vb = b.output_price ?? 9999; break;
        case 'modality': va = a.input_modalities.join(','); vb = b.input_modalities.join(','); break;
        default: return 0;
      }
      const cmp = typeof va === 'string' ? va.localeCompare(vb) : va - vb;
      return sortAsc ? cmp : -cmp;
    });
  });

  function toggleSort(col: string) {
    if (sortCol === col) { sortAsc = !sortAsc; }
    else { sortCol = col; sortAsc = true; }
  }

  const providers: { id: LlmProvider; label: string }[] = [
    { id: 'ollama', label: 'Ollama（ローカル）' },
    { id: 'openai', label: 'OpenAI' },
    { id: 'anthropic', label: 'Anthropic' },
    { id: 'bedrock', label: 'AWS Bedrock' },
    { id: 'gemini', label: 'Google Gemini' },
  ];

  async function fetchBedrockModels() {
    if (!settings.llm.bedrock.api_key || !settings.llm.bedrock.region) return;
    bedrockModelsLoading = true;
    try {
      bedrockModels = await invoke<BedrockModelInfo[]>('list_bedrock_models', { region: settings.llm.bedrock.region, apiKey: settings.llm.bedrock.api_key });
    } catch { bedrockModels = []; }
    finally { bedrockModelsLoading = false; }
  }

  function fmtPrice(p: number | null): string {
    if (p === null || p === 0) return '-';
    if (p < 0.01) return `$${p.toFixed(4)}`;
    if (p < 1) return `$${p.toFixed(3)}`;
    return `$${p.toFixed(2)}`;
  }

  async function testLlmConnection() {
    llmTestResult = '接続テスト中...';
    try {
      const llm = getLlmConfig(settings.llm);
      await invoke<string>('ai_summarize', { llm, mailBody: 'test' });
      llmTestResult = '✅ 接続成功';
    } catch (e) { llmTestResult = `❌ ${e}`; }
  }
</script>

<h3>LLM プロバイダー設定</h3>
{#each providers as p}
  {@const isActive = settings.llm.activeProvider === p.id}
  <div class="card" class:ac={isActive}>
    <div class="ch" style="cursor:pointer" onclick={() => settings.llm.activeProvider = p.id}>
      <span class="radio" class:on={isActive}></span>
      {isActive ? '🟢' : '⚪'} {p.label}
    </div>
    {#if p.id === 'ollama'}
      <label class="fl">Base URL <input bind:value={settings.llm.ollama.base_url} /></label>
      <label class="fl">モデル <select bind:value={settings.llm.ollama.model}>
        <option>llama3</option><option>mistral</option><option>codellama</option><option>gemma2</option>
      </select></label>
    {:else if p.id === 'openai'}
      <label class="fl">API Key <input type="password" bind:value={settings.llm.openai.api_key} placeholder="sk-..." /></label>
      <label class="fl">モデル <select bind:value={settings.llm.openai.model}>
        <option>gpt-4o</option><option>gpt-4o-mini</option><option>gpt-4-turbo</option>
      </select></label>
    {:else if p.id === 'anthropic'}
      <label class="fl">API Key <input type="password" bind:value={settings.llm.anthropic.api_key} placeholder="sk-ant-..." /></label>
      <label class="fl">モデル <select bind:value={settings.llm.anthropic.model}>
        <option>claude-3-5-sonnet-20241022</option><option>claude-3-opus-20240229</option><option>claude-3-haiku-20240307</option>
      </select></label>
    {:else if p.id === 'bedrock'}
      <label class="fl">認証方式 <select bind:value={settings.llm.bedrock.auth_mode}>
        <option value="api_key">Bedrock API Key</option>
        <option value="iam">AWS IAM (LiteLLM経由)</option>
      </select></label>
      <label class="fl">Region <select bind:value={settings.llm.bedrock.region}>
        <option>us-east-1</option><option>us-west-2</option><option>ap-northeast-1</option><option>eu-west-1</option>
      </select></label>
      {#if settings.llm.bedrock.auth_mode === 'api_key'}
        <label class="fl">Bedrock API Key <input type="password" bind:value={settings.llm.bedrock.api_key} onchange={fetchBedrockModels} placeholder="Bedrock API Key を入力" /></label>
      {:else}
        <label class="fl">Access Key <input type="password" bind:value={settings.llm.bedrock.access_key} placeholder="AKIA..." /></label>
        <label class="fl">Secret Key <input type="password" bind:value={settings.llm.bedrock.secret_key} /></label>
      {/if}
      <label class="fl">モデル
        {#if bedrockModels.length > 0}
          <div class="model-table-wrap">
            <table class="model-table">
              <thead><tr>
                <th class="sortable" onclick={() => toggleSort('provider')}>Provider{sortCol==='provider'?(sortAsc?' ▲':' ▼'):''}</th>
                <th class="sortable" onclick={() => toggleSort('model')}>Model{sortCol==='model'?(sortAsc?' ▲':' ▼'):''}</th>
                <th class="sortable" onclick={() => toggleSort('input')}>In $/1M{sortCol==='input'?(sortAsc?' ▲':' ▼'):''}</th>
                <th class="sortable" onclick={() => toggleSort('output')}>Out $/1M{sortCol==='output'?(sortAsc?' ▲':' ▼'):''}</th>
                <th class="sortable" onclick={() => toggleSort('modality')}>Modality{sortCol==='modality'?(sortAsc?' ▲':' ▼'):''}</th>
              </tr></thead>
              <tbody>
                {#each sortedBedrockModels as m}
                  {@const selected = settings.llm.bedrock.model === m.model_id}
                  <tr class:sel={selected} onclick={() => settings.llm.bedrock.model = m.model_id}>
                    <td class="prov">{m.provider_name}</td>
                    <td class="mn">{m.model_name}</td>
                    <td class="pr">{fmtPrice(m.input_price)}</td>
                    <td class="pr">{fmtPrice(m.output_price)}</td>
                    <td class="mod">{m.input_modalities.join(',')}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <input bind:value={settings.llm.bedrock.model} placeholder="us.anthropic.claude-sonnet-4-20250514-v1:0" />
          {#if settings.llm.bedrock.auth_mode === 'api_key' && settings.llm.bedrock.api_key}
            <button class="btn-sm gb" onclick={fetchBedrockModels} disabled={bedrockModelsLoading} style="margin-top:4px">
              {bedrockModelsLoading ? '取得中...' : 'モデル一覧を取得'}
            </button>
          {/if}
        {/if}
      </label>
    {:else if p.id === 'gemini'}
      <label class="fl">API Key <input type="password" bind:value={settings.llm.gemini.api_key} placeholder="AIza..." /></label>
      <label class="fl">モデル <select bind:value={settings.llm.gemini.model}>
        <option>gemini-1.5-pro</option><option>gemini-1.5-flash</option><option>gemini-2.0-flash</option>
      </select></label>
    {/if}
    {#if isActive}
      <div class="row mt"><button class="btn-sm gb" onclick={testLlmConnection}>接続テスト</button></div>
      {#if llmTestResult}<div class="tr">{llmTestResult}</div>{/if}
    {/if}
  </div>
{/each}

<style>
  h3 { font-size:14px;margin-bottom:12px }
  .fl { display:block;color:var(--overlay);font-size:10px;margin:6px 0 2px }
  .fl input,.fl select { display:block;width:100%;padding:5px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:11px;margin-top:2px }
  .card { background:var(--mantle);border:1px solid var(--surface1);border-radius:6px;padding:12px;margin:8px 0 }
  .card.ac { border-color:var(--green) }
  .ch { font-size:12px;font-weight:700;margin-bottom:4px;display:flex;align-items:center;gap:6px }
  .radio { width:14px;height:14px;border-radius:50%;border:2px solid var(--surface1);flex-shrink:0 }
  .radio.on { border-color:var(--green);background:var(--green) }
  .row { display:flex;gap:6px }
  .mt { margin-top:8px }
  .btn-sm { padding:4px 12px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px;cursor:pointer }
  .btn-sm.gb { border-color:var(--green);color:var(--green) }
  .tr { margin-top:6px;font-size:11px }
  .model-table-wrap { max-height:200px;overflow-y:auto;margin-top:4px;border:1px solid var(--surface1);border-radius:4px }
  .model-table { width:100%;border-collapse:collapse;font-size:10px }
  .model-table th { position:sticky;top:0;background:var(--surface0);padding:3px 6px;text-align:left;font-weight:600;border-bottom:1px solid var(--surface1) }
  .model-table th.sortable { cursor:pointer;user-select:none }
  .model-table th.sortable:hover { color:var(--green) }
  .model-table td { padding:3px 6px;border-bottom:1px solid var(--surface0);cursor:pointer }
  .model-table tr:hover td { background:var(--surface0) }
  .model-table tr.sel td { background:var(--green);color:var(--base);font-weight:600 }
  .model-table .prov { color:var(--overlay);white-space:nowrap }
  .model-table .pr { text-align:right;font-family:monospace;white-space:nowrap }
  .model-table .mod { color:var(--overlay);font-size:9px }
</style>
