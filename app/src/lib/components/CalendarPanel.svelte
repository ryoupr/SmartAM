<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let { mailBody, llmConfig, calendarName = '仕事', calendarNames = [], onClose }: {
    mailBody: string;
    llmConfig: { base_url: string; model: string; api_key: string };
    calendarName?: string;
    calendarNames?: string[];
    onClose: () => void;
  } = $props();

  type CalEvent = { title: string; start: string; end: string; location: string };

  let events: CalEvent[] = $state([]);
  let loading = $state(true);
  let registering = $state(false);
  let toast = $state('');
  let calName = $state(calendarName);

  $effect(() => {
    invoke<CalEvent[]>('detect_calendar_events', { llm: llmConfig, mailBody })
      .then(r => { events = r; loading = false; })
      .catch(() => { loading = false; });
  });

  async function register(ev: CalEvent) {
    registering = true;
    try {
      await invoke('register_calendar_event', { event: ev, calendarName: calName });
      toast = '✅ カレンダーに登録しました';
    } catch (e) { toast = `❌ ${e}`; }
    finally { registering = false; }
  }
</script>

<div class="cal-panel">
  <div class="cal-header"><span>📅 検出されたイベント</span><button class="close" onclick={onClose}>✕</button></div>
  {#if loading}
    <div class="loading">⏳ イベントを検出中...</div>
  {:else if events.length === 0}
    <div class="empty">イベントが見つかりませんでした</div>
  {:else}
    {#each events as ev, i}
      <div class="event-card">
        <input class="ev-input title" bind:value={ev.title} />
        <div class="ev-row">
          <input class="ev-input" bind:value={ev.start} />
          <span class="ev-sep">〜</span>
          <input class="ev-input" bind:value={ev.end} />
        </div>
        <input class="ev-input" bind:value={ev.location} placeholder="場所（任意）" />
        <div class="event-actions">
          <label class="cal-select">
            登録先:
            {#if calendarNames.length > 0}
              <select bind:value={calName}>
                {#each calendarNames as c}<option value={c}>{c}</option>{/each}
              </select>
            {:else}
              <input bind:value={calName} class="cal-input" />
            {/if}
          </label>
          <button class="btn-register" disabled={registering} onclick={() => register(ev)}>登録する</button>
        </div>
      </div>
    {/each}
  {/if}
  {#if toast}<div class="toast">{toast}</div>{/if}
</div>

<style>
  .cal-panel { border:1px solid var(--blue);border-radius:8px;background:var(--mantle);padding:12px;margin-top:12px }
  .cal-header { display:flex;justify-content:space-between;font-size:12px;font-weight:700;color:var(--blue);margin-bottom:8px }
  .close { background:none;border:none;color:var(--overlay);cursor:pointer }
  .loading,.empty { color:var(--overlay);text-align:center;padding:16px;font-size:12px }
  .event-card { background:#1e3a5f;border:1px solid var(--blue);border-radius:6px;padding:10px;margin-bottom:8px;display:flex;flex-direction:column;gap:6px }
  .ev-input { padding:4px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:11px;width:100% }
  .ev-input.title { font-weight:700;font-size:12px }
  .ev-input:focus { border-color:var(--blue);outline:none }
  .ev-row { display:flex;align-items:center;gap:4px }
  .ev-row .ev-input { flex:1 }
  .ev-sep { color:var(--blue);font-size:10px }
  .event-actions { display:flex;justify-content:space-between;align-items:center;margin-top:8px }
  .cal-select { font-size:9px;color:var(--overlay);display:flex;align-items:center;gap:4px }
  .cal-select select { padding:3px 6px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px }
  .cal-input { padding:3px 6px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px;width:120px }
  .btn-register { padding:6px 16px;border-radius:6px;border:none;background:#1e66f5;color:#fff;font-weight:700;font-size:11px;cursor:pointer }
  .btn-register:disabled { opacity:.6 }
  .toast { margin-top:8px;padding:6px 12px;border-radius:6px;background:#1e3a2e;color:var(--green);font-size:11px }
</style>
