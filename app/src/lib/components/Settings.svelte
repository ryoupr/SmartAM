<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { type AppSettings } from '$lib/store';
  import AccountTab from './settings/AccountTab.svelte';
  import LlmTab from './settings/LlmTab.svelte';
  import UsageTab from './settings/UsageTab.svelte';
  import ShortcutTab from './settings/ShortcutTab.svelte';

  let { settings, onClose, onSave }: {
    settings: AppSettings;
    onClose: () => void;
    onSave: (s: AppSettings) => void;
  } = $props();

  let local: AppSettings = $state(JSON.parse(JSON.stringify(settings)));
  let activeTab = $state('account');
  let saveToast = $state(false);

  const globalTabs = [
    { id: 'account', label: 'メールアカウント' },
    { id: 'llm', label: 'LLMプロバイダー' },
    { id: 'ai_usage', label: 'AI 利用状況' },
    { id: 'shortcuts', label: 'キーボードショートカット' },
    { id: 'display', label: '表示設定' },
    { id: 'about', label: 'SmartAM' },
  ];
</script>

<div class="overlay" role="dialog">
  <div class="win">
    <div class="hdr"><span>設定</span><button class="x" onclick={onClose}>✕</button></div>
    <div class="body">
      <nav class="nav">
        {#each globalTabs as tab}
          <button class="ni" class:active={activeTab === tab.id} onclick={() => activeTab = tab.id}>{tab.label}</button>
        {/each}
      </nav>
      <div class="content">
        {#if activeTab === 'account'}
          <AccountTab bind:settings={local} />
        {:else if activeTab === 'llm'}
          <LlmTab bind:settings={local} />
        {:else if activeTab === 'ai_usage'}
          <UsageTab bind:settings={local} />
        {:else if activeTab === 'shortcuts'}
          <ShortcutTab bind:settings={local} />
        {:else if activeTab === 'display'}
          <h3>表示設定</h3>
          <label class="fl">テーマ
            <select bind:value={local.theme} onchange={() => document.documentElement.setAttribute('data-theme', local.theme)}>
              <option value="dark">ダーク</option>
              <option value="light">ライト</option>
            </select>
          </label>
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
          <label class="fl">ログレベル
            <select bind:value={local.logLevel} onchange={() => invoke('set_log_level', { level: local.logLevel })}>
              <option value="error">Error</option>
              <option value="warn">Warn</option>
              <option value="info">Info（デフォルト）</option>
              <option value="debug">Debug</option>
              <option value="trace">Trace</option>
            </select>
          </label>
        {:else if activeTab === 'about'}
          <h3>SmartAM</h3>
          <div class="about-card">
            <div class="about-logo"><span class="chi-mark"><span class="chi">智</span><span class="chi-bar"></span></span></div>
            <div class="about-name">Smart<span class="am">AM</span></div>
            <div class="about-desc">AI-native Desktop Mail Client</div>
            <div class="about-ver">v0.1.10</div>
            <div class="about-info">
              <div>Platform: macOS (Tauri v2)</div>
              <div>Frontend: SvelteKit + TypeScript</div>
              <div>Backend: Rust</div>
            </div>
            <div class="about-links">
              <button class="btn-sm" onclick={() => invoke('open_external_url', { url: 'https://github.com/ryoupr/SmartAM' })}>GitHub</button>
              <button class="btn-sm" onclick={() => invoke('open_external_url', { url: 'https://github.com/ryoupr/SmartAM/releases' })}>Releases</button>
            </div>
            <div class="about-copy">© 2026 SmartAM · MIT License</div>
          </div>
        {/if}
      </div>
    </div>
    <div class="ftr">
      {#if saveToast}<span class="save-toast">✅ 保存しました</span>{/if}
      <button class="btn-save" onclick={() => { onSave(local); saveToast = true; setTimeout(() => saveToast = false, 3000); }}>保存</button>
    </div>
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
  .content { flex:1;padding:16px 24px;overflow-y:auto }
  .content h3 { font-size:14px;margin-bottom:12px }
  .fl { display:block;color:var(--overlay);font-size:10px;margin:6px 0 2px }
  .fl input,.fl select { display:block;width:100%;padding:5px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:11px;margin-top:2px }
  .ftr { display:flex;justify-content:flex-end;padding:10px 16px;border-top:1px solid var(--surface1) }
  .btn-save { padding:8px 24px;border-radius:6px;border:none;background:var(--mauve);color:var(--base);font-weight:700;cursor:pointer }
  .btn-sm { padding:4px 12px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px;cursor:pointer }
  .save-toast { color:var(--green);font-size:11px;margin-right:12px }
  .about-card { text-align:center;padding:24px 16px }
  .about-logo { font-size:48px;margin-bottom:8px;display:flex;justify-content:center }
  .chi-mark { display:inline-flex;flex-direction:column;align-items:center }
  .chi-mark .chi { font-family:'Noto Sans JP',sans-serif;font-weight:900;font-size:48px;letter-spacing:-0.05em;color:var(--text);line-height:1 }
  .chi-mark .chi-bar { width:90%;height:5px;background:var(--red);border-radius:2px;margin-top:-4px }
  .about-name { font-size:20px;font-weight:700;color:var(--text) }
  .about-name .am { color:var(--red) }
  .about-desc { font-size:11px;color:var(--overlay);margin:4px 0 8px }
  .about-ver { font-size:13px;font-weight:700;color:var(--mauve);margin-bottom:16px }
  .about-info { font-size:10px;color:var(--overlay);line-height:1.8 }
  .about-links { display:flex;gap:8px;justify-content:center;margin:16px 0 }
  .about-copy { font-size:9px;color:var(--surface1);margin-top:12px }
</style>
