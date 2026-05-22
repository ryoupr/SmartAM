<script lang="ts">
  import type { MailSummary } from '$lib/types';
  import { formatMailDate } from '$lib/store';
  import Icon from './Icon.svelte';

  let { mails, selectedUid, onSelect, onLoadMore, onSearchInput, onRefresh, loading = false, loadingMore = false, searchQuery = $bindable(''), pageSize = 200, dateFormat = 'YYYY/MM/DD HH:mm:ss', timezone = 'Asia/Tokyo', selectedUids = $bindable(new Set<number>()), onMultiSelect, folderLabel = '受信トレイ' }: {
    mails: MailSummary[];
    selectedUid: number | null;
    onSelect: (uid: number) => void;
    onLoadMore?: () => void;
    onSearchInput?: (q: string) => void;
    onRefresh?: () => void;
    loading?: boolean;
    loadingMore?: boolean;
    searchQuery?: string;
    pageSize?: number;
    dateFormat?: string;
    timezone?: string;
    selectedUids?: Set<number>;
    onMultiSelect?: (uids: Set<number>) => void;
    folderLabel?: string;
  } = $props();

  let unreadCount = $derived(mails.filter(m => !m.seen).length);

  const ITEM_HEIGHT = 50;
  const OVERSCAN = 5;

  let listEl: HTMLDivElement | undefined = $state(undefined);
  let scrollTop = $state(0);
  let clientHeight = $state(600);

  let visibleStart = $derived(Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - OVERSCAN));
  let visibleEnd = $derived(Math.min(mails.length, Math.ceil((scrollTop + clientHeight) / ITEM_HEIGHT) + OVERSCAN));
  let visibleMails = $derived(mails.slice(visibleStart, visibleEnd));
  let totalHeight = $derived(mails.length * ITEM_HEIGHT);
  let offsetY = $derived(visibleStart * ITEM_HEIGHT);

  $effect(() => {
    if (!selectedUid || !listEl) return;
    const idx = mails.findIndex(m => m.uid === selectedUid);
    if (idx >= 0) {
      const itemTop = idx * ITEM_HEIGHT;
      if (itemTop < scrollTop || itemTop + ITEM_HEIGHT > scrollTop + clientHeight) {
        listEl.scrollTop = itemTop - clientHeight / 2 + ITEM_HEIGHT / 2;
      }
    }
  });

  let lastClickedUid: number | null = $state(null);

  function handleClick(uid: number, e: MouseEvent) {
    if (e.shiftKey && lastClickedUid !== null) {
      const startIdx = mails.findIndex(m => m.uid === lastClickedUid);
      const endIdx = mails.findIndex(m => m.uid === uid);
      if (startIdx >= 0 && endIdx >= 0) {
        const [from, to] = startIdx < endIdx ? [startIdx, endIdx] : [endIdx, startIdx];
        const next = new Set(selectedUids);
        for (let i = from; i <= to; i++) next.add(mails[i].uid);
        selectedUids = next;
        onMultiSelect?.(selectedUids);
      }
    } else if (e.ctrlKey || e.metaKey) {
      const next = new Set(selectedUids);
      if (next.has(uid)) next.delete(uid); else next.add(uid);
      selectedUids = next;
      onMultiSelect?.(selectedUids);
    } else {
      selectedUids = new Set();
      onSelect(uid);
    }
    lastClickedUid = uid;
  }

  function handleScroll(e: Event) {
    const el = e.target as HTMLElement;
    scrollTop = el.scrollTop;
    clientHeight = el.clientHeight;
    // Infinite scroll: load more when near bottom
    const remaining = el.scrollHeight - el.scrollTop - el.clientHeight;
    if (remaining < ITEM_HEIGHT * 10 && !loadingMore && !loading && onLoadMore) {
      onLoadMore();
    }
  }
</script>

<div class="mail-list" onscroll={handleScroll} bind:this={listEl}>
  <div class="list-header">
    <span class="list-title">{folderLabel}</span>
    <span class="list-count">{mails.length} 件{unreadCount > 0 ? ` · 新着 ${unreadCount}` : ''}</span>
    {#if onRefresh}<button class="refresh" onclick={onRefresh} title="更新"><Icon name="regen" size={14} /></button>{/if}
  </div>
  <div class="search">
    <input type="text" placeholder="🔍 検索..." bind:value={searchQuery} oninput={(e) => onSearchInput?.(e.currentTarget.value)} />
    {#if searchQuery}<button class="clear" onclick={() => { searchQuery = ''; onSearchInput?.(''); }}>✕</button>{/if}
  </div>
  {#if searchQuery}<div class="result-count">{mails.length}件の検索結果</div>{/if}
  {#if selectedUids.size > 0}<div class="result-count">{selectedUids.size}件選択中</div>{/if}
  {#if loading}
    <div class="empty">読み込み中...</div>
  {:else}
    <div class="virtual-container" style:height="{totalHeight}px">
      <div class="virtual-offset" style:transform="translateY({offsetY}px)">
        {#each visibleMails as mail}
          <button class="mail-item" class:selected={selectedUid === mail.uid || selectedUids.has(mail.uid)} class:unread={!mail.seen} data-uid={mail.uid} onclick={(e) => handleClick(mail.uid, e)}>
            <div class="mail-header">
              <span class="from">{mail.from}</span>
              <span class="date">{formatMailDate(mail.date, dateFormat, timezone)}</span>
            </div>
            <div class="subject">{mail.subject}</div>
          </button>
        {:else}
          <div class="empty">メールがありません</div>
        {/each}
      </div>
    </div>
    {#if loadingMore}
      <div class="loading-more">読み込み中...</div>
    {/if}
  {/if}
</div>

<style>
  .mail-list { width:380px;min-width:380px;border-right:1px solid var(--surface1);overflow-y:auto;display:flex;flex-direction:column }
  .list-header { display:flex;align-items:center;padding:12px 12px 4px;gap:8px;flex-shrink:0 }
  .list-title { font-size:17px;font-weight:700;color:var(--ink, var(--text)) }
  .list-count { font-size:11px;color:var(--ink-60, var(--overlay));flex:1 }
  .refresh { background:none;border:none;color:var(--ink-60, var(--overlay));cursor:pointer;padding:4px;border-radius:4px }
  .refresh:hover { background:var(--bone, var(--surface0)) }
  .search { padding:8px;position:relative;flex-shrink:0 }
  .search input { width:100%;padding:6px 28px 6px 10px;border-radius:6px;border:1.5px solid var(--line, var(--surface1));background:var(--paper-wh, var(--surface0));color:var(--ink, var(--text));font-size:11px }
  .search input:focus { outline:none;border-color:var(--red) }
  .search input::placeholder { color:var(--ink-40, var(--overlay)) }
  .clear { position:absolute;right:14px;top:50%;transform:translateY(-50%);background:none;border:none;color:var(--overlay);cursor:pointer;font-size:11px }
  .result-count { padding:0 12px 4px;color:var(--ink-60, var(--overlay));font-size:9px;flex-shrink:0 }
  .virtual-container { position:relative;flex:1 }
  .virtual-offset { position:absolute;left:0;right:0;top:0 }
  .mail-item { padding:12px 16px 12px 13px;border:none;background:none;text-align:left;cursor:pointer;border-bottom:1px solid var(--line, var(--surface1));border-left:3px solid transparent;height:50px;box-sizing:border-box;outline:none;width:100%;display:block }
  .mail-item:hover { background:var(--paper, var(--surface0)) }
  .mail-item.selected { background:var(--paper-wh, var(--surface0));border-left-color:var(--red) }
  .mail-item.unread { border-left-color:var(--red) }
  .mail-item.unread .from { font-weight:600 }
  .mail-item.unread .subject { font-weight:500 }
  .mail-header { display:flex;justify-content:space-between;margin-bottom:4px }
  .from { color:var(--ink, var(--text));font-size:11px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;flex:1;min-width:0 }
  .date { color:var(--ink-60, var(--overlay));font-size:9px;font-family:'JetBrains Mono',monospace }
  .subject { color:var(--ink-80, var(--text));font-size:10px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis }
  .empty { color:var(--ink-40, var(--overlay));text-align:center;padding:40px;font-size:12px }
  .loading-more { text-align:center;padding:12px;color:var(--overlay);font-size:10px }
</style>
