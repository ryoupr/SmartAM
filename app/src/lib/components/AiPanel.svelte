<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let { mailBody, llmConfig, onClose, onUseReply, initialPanel = 'summary' }: {
    mailBody: string;
    llmConfig: { base_url: string; model: string };
    onClose: () => void;
    onUseReply?: (text: string) => void;
    initialPanel?: string;
  } = $props();

  type Panel = 'summary' | 'nuance' | 'draft' | 'translate';
  type Nuance = { icon: string; label: string; description: string };

  let panel: Panel = $state(initialPanel as Panel);
  let loading = $state(true);
  let result = $state('');
  let nuances: Nuance[] = $state([]);
  let selectedNuance = $state('');
  let instruction = $state('');
  let targetLang = $state('English');
  let error = $state('');

  const langs = ['English','中文（簡体）','中文（繁體）','한국어','Français','Deutsch','Español','Português','Русский','العربية'];
  const colors: Record<string, string> = { summary:'var(--green)', nuance:'var(--yellow)', draft:'var(--yellow)', translate:'var(--blue)' };

  // Auto-trigger on mount based on initialPanel
  $effect(() => {
    if (initialPanel === 'summary') summarize();
    else if (initialPanel === 'nuance') startDraft();
    else if (initialPanel === 'translate') translate();
  });

  async function summarize() {
    panel = 'summary'; loading = true; error = '';
    try { result = await invoke('ai_summarize', { llm: llmConfig, mailBody }); }
    catch (e) { error = String(e); } finally { loading = false; }
  }

  async function startDraft() {
    panel = 'nuance'; loading = true; error = '';
    try { nuances = await invoke('ai_draft_nuances', { llm: llmConfig, mailBody }); }
    catch (e) { error = String(e); } finally { loading = false; }
  }

  async function generateReply(nuance: Nuance) {
    selectedNuance = `${nuance.icon} ${nuance.label}`;
    panel = 'draft'; loading = true; error = '';
    try { result = await invoke('ai_draft_reply', { llm: llmConfig, mailBody, nuance: nuance.label, instruction: '' }); }
    catch (e) { error = String(e); } finally { loading = false; }
  }

  async function regenerate() {
    loading = true; error = '';
    try { result = await invoke('ai_draft_reply', { llm: llmConfig, mailBody, nuance: selectedNuance, instruction }); }
    catch (e) { error = String(e); } finally { loading = false; }
  }

  async function translate() {
    panel = 'translate'; loading = true; error = '';
    try { result = await invoke('ai_translate', { llm: llmConfig, text: mailBody, targetLang }); }
    catch (e) { error = String(e); } finally { loading = false; }
  }
</script>

<div class="ai-panel" style="--ac:{colors[panel] ?? 'var(--mauve)'}">
  <div class="ai-hdr">
    {#if panel === 'summary'}📝 要約
    {:else if panel === 'nuance'}✍ Step 1: どう返信しますか？
    {:else if panel === 'draft'}✍ Step 2: 返信案（{selectedNuance}）
    {:else if panel === 'translate'}🌐 翻訳
    {/if}
    <button class="x" onclick={onClose}>✕</button>
  </div>

  {#if loading}
    <div class="loading">
      <div class="spinner">⏳ AIが処理中...</div>
      <div class="dots">● ● ●</div>
    </div>
  {:else if error}
    <div class="err">{error}</div>
  {:else if panel === 'nuance'}
    <div class="nuances">
      {#each nuances as n}
        {#if n.icon === '🚫'}
          <div class="nbtn no-reply">🚫 {n.label} — {n.description}</div>
        {:else}
          <button class="nbtn" onclick={() => generateReply(n)}>{n.icon} {n.label}（{n.description}）</button>
        {/if}
      {/each}
    </div>
  {:else if panel === 'draft'}
    <div class="result">{result}</div>
    <div class="draft-row">
      <button class="back" onclick={() => { panel = 'nuance'; loading = true; startDraft(); }}>← ニュアンス選択に戻る</button>
      <input bind:value={instruction} placeholder="例: もっと丁寧に" class="inst" />
      <button class="regen" onclick={regenerate}>🔄 再生成</button>
      <button class="use" onclick={() => onUseReply?.(result)}>使う</button>
    </div>
  {:else}
    <div class="result">{result}</div>
    {#if panel === 'translate'}
      <div class="tr-row">
        <select bind:value={targetLang} onchange={translate}>
          {#each langs as l}<option>{l}</option>{/each}
        </select>
      </div>
    {/if}
    <div class="act-row">
      <button class="copy" onclick={() => navigator.clipboard.writeText(result)}>📋 コピー</button>
      <button class="regen" onclick={panel === 'summary' ? summarize : translate}>🔄 再生成</button>
    </div>
  {/if}
</div>

<style>
  .ai-panel { border:1px solid var(--ac);border-radius:8px;background:var(--mantle);padding:12px;margin-top:12px }
  .ai-hdr { display:flex;justify-content:space-between;font-size:12px;font-weight:700;color:var(--ac);margin-bottom:8px }
  .x { background:none;border:none;color:var(--overlay);cursor:pointer;font-size:14px }
  .loading { text-align:center;padding:20px;color:var(--overlay) }
  .spinner { font-size:12px;margin-bottom:8px }
  .dots { font-size:14px;color:var(--ac);animation:pulse 1.5s infinite }
  @keyframes pulse { 0%,100%{opacity:.3} 50%{opacity:1} }
  .err { color:var(--red);font-size:11px;padding:8px }
  .result { font-size:12px;line-height:1.6;white-space:pre-wrap;padding:8px 0 }
  .nuances { display:flex;flex-direction:column;gap:6px }
  .nbtn { padding:8px 12px;border-radius:6px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);cursor:pointer;text-align:left;font-size:11px }
  .nbtn:hover { border-color:var(--yellow) }
  .nbtn.no-reply { color:var(--overlay);border-color:var(--surface1);cursor:default;font-style:italic }
  .nbtn.no-reply:hover { border-color:var(--surface1) }
  .draft-row { display:flex;gap:6px;align-items:center;margin-top:8px;flex-wrap:wrap }
  .back { background:none;border:none;color:var(--blue);font-size:10px;cursor:pointer;text-decoration:underline }
  .inst { flex:1;padding:6px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px }
  .regen { padding:5px 12px;border-radius:6px;border:none;background:var(--mauve);color:var(--base);font-size:10px;cursor:pointer }
  .use { padding:5px 12px;border-radius:6px;border:none;background:var(--green);color:var(--base);font-size:10px;cursor:pointer;font-weight:700 }
  .copy { padding:5px 12px;border-radius:6px;border:1px solid var(--ac);background:var(--surface0);color:var(--ac);font-size:10px;cursor:pointer }
  .act-row { display:flex;gap:6px;margin-top:8px }
  .tr-row { margin:8px 0 }
  .tr-row select { padding:4px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px }
</style>
