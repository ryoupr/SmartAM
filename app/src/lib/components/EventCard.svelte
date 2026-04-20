<script lang="ts">
  import type { CalendarEvent } from '$lib/types';

  let { event, onAccept, onDecline }: {
    event: CalendarEvent;
    onAccept: () => Promise<void>;
    onDecline: () => Promise<void>;
  } = $props();

  let responding = $state(false);
  let toast = $state('');

  async function handleResponse(action: () => Promise<void>, label: string) {
    responding = true;
    toast = '';
    try {
      await action();
      toast = `✅ ${label}しました`;
    } catch (e) { toast = `❌ ${e}`; }
    finally { responding = false; }
  }

  function formatDt(raw: string): string {
    if (!raw) return '';
    // Handle YYYYMMDDTHHMMSS or YYYYMMDDTHHMMSSZ
    const m = raw.replace(/[^0-9T]/g, '').match(/^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})/);
    if (!m) return raw;
    const [, y, mo, d, h, mi] = m;
    const dt = new Date(+y, +mo - 1, +d, +h, +mi);
    const days = ['日', '月', '火', '水', '木', '金', '土'];
    return `${y}-${mo}-${d} (${days[dt.getDay()]}) ${h}:${mi}`;
  }

  function statusLabel(s: string): string {
    const map: Record<string, string> = { ACCEPTED: '承諾済', DECLINED: '辞退', 'NEEDS-ACTION': '未回答' };
    return map[s] || s || '未回答';
  }
  function statusColor(s: string): string {
    if (s === 'ACCEPTED') return 'var(--green)';
    if (s === 'DECLINED') return 'var(--red)';
    return 'var(--yellow)';
  }

  let attendeeSummary = $derived(
    event.attendees.length <= 2
      ? event.attendees.join(', ')
      : `${event.attendees.slice(0, 2).join(', ')} 他${event.attendees.length - 2}名`
  );
</script>

<div class="ev-card">
  <div class="ev-header">
    <span class="ev-icon">📅</span>
    <span class="ev-title">カレンダー招待</span>
    <span class="ev-badge" style="background:{statusColor(event.status)}">{statusLabel(event.status)}</span>
  </div>
  <div class="ev-sep"></div>
  <div class="ev-name">{event.summary}</div>
  {#if event.dtstart}<div class="ev-row">🕐 {formatDt(event.dtstart)}{#if event.dtend} 〜 {formatDt(event.dtend).split(') ')[1]}{/if}</div>{/if}
  {#if event.location}<div class="ev-row">📍 {event.location}</div>{/if}
  {#if event.organizer}<div class="ev-row">👤 主催: {event.organizer}</div>{/if}
  {#if event.attendees.length > 0}<div class="ev-row">👥 参加者: {attendeeSummary}</div>{/if}
  {#if event.description}<div class="ev-desc">{event.description.replace(/<br\s*\/?>/gi, '\n').replace(/\\n/g, '\n')}</div>{/if}
  <div class="ev-actions">
    <button class="btn-accept" disabled={responding} onclick={() => handleResponse(onAccept, '承諾')}>承諾</button>
    <button class="btn-decline" disabled={responding} onclick={() => handleResponse(onDecline, '辞退')}>辞退</button>
  </div>
  {#if toast}<div class="ev-toast">{toast}</div>{/if}
</div>

<style>
  .ev-card { border:2px solid var(--blue);border-radius:8px;padding:12px;margin:8px 0;background:var(--base) }
  .ev-header { display:flex;align-items:center;gap:6px;margin-bottom:8px }
  .ev-icon { font-size:18px }
  .ev-title { font-size:13px;font-weight:700;color:var(--blue) }
  .ev-badge { margin-left:auto;font-size:9px;font-weight:700;padding:2px 8px;border-radius:4px;color:var(--base) }
  .ev-sep { border-top:1px solid var(--surface1);margin-bottom:8px }
  .ev-name { font-size:13px;font-weight:700;margin-bottom:6px }
  .ev-row { font-size:11px;color:var(--subtext);margin-bottom:4px }
  .ev-desc { font-size:11px;color:var(--subtext);background:var(--surface0);padding:8px;border-radius:4px;margin:8px 0;white-space:pre-wrap }
  .ev-actions { display:flex;gap:6px;margin-top:10px }
  .ev-actions button { padding:4px 12px;border-radius:6px;border:none;font-size:10px;font-weight:700;cursor:pointer;color:var(--base) }
  .btn-accept { background:var(--green) }
  .btn-decline { background:var(--red) }
  .ev-actions button:disabled { opacity:.6;cursor:default }
  .ev-toast { font-size:10px;margin-top:6px;padding:4px 8px;border-radius:4px;background:var(--surface0) }
</style>
