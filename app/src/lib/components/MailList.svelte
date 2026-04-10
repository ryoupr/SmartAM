<script lang="ts">
  import type { MailSummary } from '$lib/types';

  let { mails, selectedUid, onSelect, onLoadMore, onSearchInput, loading = false, loadingMore = false, searchQuery = $bindable(''), pageSize = 200 }: {
    mails: MailSummary[];
    selectedUid: number | null;
    onSelect: (uid: number) => void;
    onLoadMore?: () => void;
    onSearchInput?: (q: string) => void;
    loading?: boolean;
    loadingMore?: boolean;
    searchQuery?: string;
    pageSize?: number;
  } = $props();

  function handleScroll(e: Event) {
    const el = e.target as HTMLElement;
    // Each mail item is ~50px, calculate how many items are below viewport
    const itemHeight = 50;
    const remaining = el.scrollHeight - el.scrollTop - el.clientHeight;
    const remainingItems = remaining / itemHeight;
    // Trigger when remaining items < half a page
    if (remainingItems < pageSize / 2 && !loadingMore && !loading && onLoadMore) {
      onLoadMore();
    }
  }
</script>

<div class="mail-list" onscroll={handleScroll}>
  <div class="search">
    <input type="text" placeholder="🔍 検索..." bind:value={searchQuery} oninput={(e) => onSearchInput?.(e.currentTarget.value)} />
    {#if searchQuery}<button class="clear" onclick={() => { searchQuery = ''; onSearchInput?.(''); }}>✕</button>{/if}
  </div>
  {#if searchQuery}<div class="result-count">{mails.length}件の検索結果</div>{/if}
  {#if loading}
    <div class="empty">読み込み中...</div>
  {:else}
    {#each mails as mail}
      <button class="mail-item" class:selected={selectedUid === mail.uid} class:unread={!mail.seen} onclick={() => onSelect(mail.uid)}>
        <div class="mail-header">
          <span class="from">{mail.from}</span>
          <span class="date">{mail.date}</span>
        </div>
        <div class="subject">{mail.subject}</div>
      </button>
    {:else}
      <div class="empty">メールがありません</div>
    {/each}
    {#if loadingMore}
      <div class="loading-more">読み込み中...</div>
    {/if}
  {/if}
</div>

<style>
  .mail-list { width:290px;min-width:290px;border-right:1px solid var(--surface1);overflow-y:auto;display:flex;flex-direction:column }
  .search { padding:8px;position:relative }
  .search input { width:100%;padding:6px 28px 6px 10px;border-radius:6px;border:none;background:var(--surface0);color:var(--text);font-size:11px }
  .search input:focus { outline:1px solid var(--mauve) }
  .search input::placeholder { color:var(--overlay) }
  .clear { position:absolute;right:14px;top:50%;transform:translateY(-50%);background:none;border:none;color:var(--overlay);cursor:pointer;font-size:11px }
  .result-count { padding:0 12px 4px;color:var(--overlay);font-size:9px }
  .mail-item { padding:10px 12px;border:none;background:none;text-align:left;cursor:pointer;border-bottom:1px solid var(--surface1);border-left:2px solid transparent }
  .mail-item:hover { background:var(--surface0) }
  .mail-item.selected { background:var(--surface0);border-left-color:var(--mauve) }
  .mail-item.unread .from { font-weight:700 }
  .mail-header { display:flex;justify-content:space-between;margin-bottom:4px }
  .from { color:var(--text);font-size:11px }
  .date { color:var(--overlay);font-size:9px }
  .subject { color:var(--text);font-size:10px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis }
  .empty { color:var(--overlay);text-align:center;padding:40px;font-size:12px }
  .loading-more { text-align:center;padding:12px;color:var(--overlay);font-size:10px }
</style>
