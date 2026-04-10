<script lang="ts">
  import { slide } from 'svelte/transition';
  import type { Account } from '$lib/store';
  let { onOpenSettings, onCompose, onRefresh, onFolderSelect, accounts = [], activeAccountIndex = 0, activeFolder = 'INBOX', llmLabel = '', syncing = false, syncStatus = '' }: {
    onOpenSettings?: () => void;
    onCompose?: () => void;
    onRefresh?: () => void;
    onFolderSelect?: (accountIndex: number, folder: string) => void;
    accounts?: Account[];
    activeAccountIndex?: number;
    activeFolder?: string;
    llmLabel?: string;
    syncing?: boolean;
    syncStatus?: string;
  } = $props();

  const folders = [
    { id: 'INBOX', label: '受信トレイ' },
    { id: 'STARRED', label: 'スター付き' },
    { id: 'SENT', label: '送信済み' },
    { id: 'DRAFTS', label: '下書き' },
    { id: 'ALL', label: 'すべてのメール' },
    { id: 'SPAM', label: '迷惑メール' },
    { id: 'TRASH', label: 'ゴミ箱' },
  ];
  let collapsed: Set<number> = $state(new Set());

  function toggleCollapse(i: number) {
    const next = new Set(collapsed);
    if (next.has(i)) next.delete(i); else next.add(i);
    collapsed = next;
  }
</script>

<aside class="sidebar">
  <div class="logo">✉ SmartAM</div>
  <button class="compose" onclick={() => onCompose?.()}>+ 新規作成</button>
  {#if accounts.length > 0}
    {#each accounts as a, i}
      <button class="acc-header" class:active={i === activeAccountIndex} onclick={() => toggleCollapse(i)}>
        <span class="dot" class:pulse={syncing && i === activeAccountIndex}></span>
        <span class="acc-email">{a.email}</span>
        <span class="chevron" class:open={!collapsed.has(i)}>{collapsed.has(i) ? '▸' : '▾'}</span>
        {#if i === activeAccountIndex}
          <button class="refresh-btn" onclick={(e: MouseEvent) => { e.stopPropagation(); onRefresh?.(); }}>🔄</button>
        {/if}
      </button>
      {#if !collapsed.has(i)}
        <div transition:slide={{ duration: 150 }}>
        {#each folders as folder}
          <button class="nav-item" class:active={i === activeAccountIndex && activeFolder === folder.id} onclick={() => onFolderSelect?.(i, folder.id)}>
            {folder.label}
          </button>
        {/each}
        {#if syncing && i === activeAccountIndex}
          <div class="sync-info"><div class="sync-bar"><div class="sync-bar-fill"></div></div><span class="sync-label">同期中...</span></div>
        {:else if syncStatus && i === activeAccountIndex}
          <div class="sync-info"><span class="sync-label">{syncStatus}</span></div>
        {/if}
        </div>
      {/if}
    {/each}
  {:else}
    <button class="nav-item" onclick={() => onOpenSettings?.()}>+ アカウントを追加</button>
  {/if}
  <div class="llm-badge">🤖 {llmLabel || 'LLM未設定'}</div>
  <button class="settings-btn" onclick={() => onOpenSettings?.()}>⚙ 設定</button>
</aside>

<style>
  .sidebar { width:220px;min-width:220px;background:var(--mantle);border-right:1px solid var(--surface1);display:flex;flex-direction:column;padding:8px 0;overflow-y:auto }
  .logo { color:var(--mauve);font-size:14px;font-weight:700;text-align:center;padding:8px }
  .compose { margin:4px 12px 8px;padding:8px;border-radius:6px;border:none;background:var(--mauve);color:var(--base);font-weight:700;cursor:pointer;font-size:12px }
  .acc-header { display:flex;align-items:center;gap:6px;padding:8px 12px 4px;color:var(--subtext);font-size:10px;border:none;background:none;width:100%;text-align:left;cursor:pointer }
  .acc-header:hover { background:var(--surface0) }
  .acc-header.active { color:var(--green) }
  .acc-email { overflow:hidden;text-overflow:ellipsis;white-space:nowrap;flex:1;font-weight:700 }
  .dot { width:8px;height:8px;border-radius:50%;background:var(--green);flex-shrink:0 }
  .dot.pulse { animation:pulse 1.2s ease-in-out infinite }
  @keyframes pulse { 0%,100%{opacity:1} 50%{opacity:.3} }
  .chevron { font-size:8px;color:var(--overlay) }
  .refresh-btn { background:none;border:none;cursor:pointer;font-size:10px;margin-left:auto;flex-shrink:0 }
  .nav-item { display:block;width:calc(100% - 16px);padding:4px 12px 4px 28px;border:none;background:none;color:var(--subtext);font-size:11px;cursor:pointer;text-align:left;border-radius:4px;margin:1px 8px }
  .nav-item:hover { background:var(--surface0) }
  .nav-item.active { background:var(--surface0);color:var(--text) }
  .sync-info { padding:0 12px 4px 28px;height:14px;display:flex;align-items:center;position:relative }
  .sync-bar { position:absolute;left:28px;right:12px;bottom:0;height:2px;background:var(--surface1);border-radius:1px;overflow:hidden }
  .sync-bar-fill { width:40%;height:100%;background:var(--green);border-radius:1px;animation:slide 1.2s ease-in-out infinite }
  @keyframes slide { 0%{transform:translateX(-100%)} 100%{transform:translateX(350%)} }
  .sync-label { font-size:8px;color:var(--overlay) }
  .llm-badge { margin:auto 8px 4px;padding:6px 12px;border-radius:6px;background:var(--surface0);color:var(--green);font-size:10px;border:1px solid var(--surface1) }
  .settings-btn { margin:4px 8px 8px;padding:6px 12px;border-radius:6px;border:1px solid var(--surface1);background:none;color:var(--overlay);font-size:10px;cursor:pointer;text-align:left }
</style>
