<script lang="ts">
  let { name, size = 16 }: { name: string; size?: number } = $props();
</script>

{#if name === 'translate'}
  <span class="icon translate-text" style:width="{size}px" style:height="{size}px">
    <span class="latin">A</span><span class="kana">あ</span>
  </span>
{:else}
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
    {#if name === 'reply'}
      <path d="M9 4L4 9l5 5"/><path d="M4 9h11a4 4 0 0 1 0 8h-1"/>
    {:else if name === 'forward'}
      <path d="M15 4l5 5-5 5"/><path d="M20 9H9a4 4 0 0 0 0 8h1"/>
    {:else if name === 'archive'}
      <rect x="3" y="3" width="18" height="5" rx="1"/><path d="M3 8v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V8"/><path d="M10 12h4"/>
    {:else if name === 'trash'}
      <path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M5 6v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V6"/><path d="M10 11v6"/><path d="M14 11v6"/>
    {:else if name === 'star'}
      <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" fill="currentColor"/>
    {:else if name === 'summary'}
      <path d="M9 5h10"/><path d="M9 9h7"/><path d="M9 13h10"/><path d="M5 5l1 1 2-2"/><path d="M5 13l1 1 2-2"/>
    {:else if name === 'draft'}
      <path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/>
    {:else if name === 'calendar'}
      <rect x="3" y="4" width="18" height="18" rx="2"/><path d="M3 10h18"/><path d="M8 2v4"/><path d="M16 2v4"/><circle cx="12" cy="15" r="1.5" fill="currentColor" stroke="none"/>
    {:else if name === 'regen'}
      <path d="M21 2v6h-6"/><path d="M3 12a9 9 0 0 1 15-6.7L21 8"/><path d="M3 22v-6h6"/><path d="M21 12a9 9 0 0 1-15 6.7L3 16"/>
    {:else if name === 'send'}
      <path d="M22 2L11 13"/><path d="M22 2l-7 20-4-9-9-4z"/>
    {:else if name === 'attach'}
      <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"/>
    {/if}
  </svg>
{/if}

<style>
  .translate-text {
    display: inline-flex; align-items: baseline; justify-content: center;
    gap: 1px; line-height: 1; font-weight: 700;
  }
  .latin { font-family: 'Inter', sans-serif; font-size: 11px; letter-spacing: -0.04em; }
  .kana { font-family: 'Noto Sans JP', sans-serif; font-size: 10px; font-weight: 700; letter-spacing: -0.04em; }
</style>
