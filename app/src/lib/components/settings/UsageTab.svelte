<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { AppSettings } from '$lib/store';

  let { settings = $bindable() }: { settings: AppSettings } = $props();

  type UsageSummary = {
    month: string;
    models: { model: string; input_tokens: number; output_tokens: number; cost_usd: number; requests: number }[];
    total_cost_usd: number;
    budget_limit_usd: number;
    budget_remaining_usd: number;
  };

  let aiUsage: UsageSummary | null = $state(null);
  let usageMonths: string[] = $state([]);
  let selectedMonth = $state('');

  async function fetchAiUsage() {
    try {
      const months = await invoke<string[]>('get_ai_usage_months');
      usageMonths = months;
      if (!selectedMonth) selectedMonth = months[0] ?? '';
      if (selectedMonth) {
        aiUsage = await invoke<UsageSummary>('get_ai_usage_for_month', { month: selectedMonth });
      } else {
        aiUsage = await invoke<UsageSummary>('get_ai_usage');
      }
    } catch { aiUsage = null; }
  }

  async function onMonthChange() {
    if (!selectedMonth) return;
    try { aiUsage = await invoke<UsageSummary>('get_ai_usage_for_month', { month: selectedMonth }); } catch { aiUsage = null; }
  }
</script>

<h3>AI 利用状況</h3>
<label class="fl">月額利用上限 (USD)
  <input type="number" step="0.5" min="0" bind:value={settings.aiBudgetLimitUsd} placeholder="0 = 無制限" />
</label>
<div class="cm" style="margin-bottom:12px">{settings.aiBudgetLimitUsd > 0 ? `$${settings.aiBudgetLimitUsd} を超えるとAI機能が停止します` : '上限なし（無制限）'}</div>
<div class="row" style="margin-bottom:12px;align-items:center">
  <button class="btn-sm gb" onclick={fetchAiUsage}>利用状況を更新</button>
  {#if usageMonths.length > 0}
    <select class="month-sel" bind:value={selectedMonth} onchange={onMonthChange}>
      {#each usageMonths as m}<option value={m}>{m}</option>{/each}
    </select>
  {/if}
</div>
{#if aiUsage}
  <div class="card">
    <div class="ch">📊 {aiUsage.month}</div>
    {#if aiUsage.budget_limit_usd > 0}
      {@const pct = Math.min(100, (aiUsage.total_cost_usd / aiUsage.budget_limit_usd) * 100)}
      <div class="budget-bar-wrap">
        <div class="budget-bar" style="width:{pct}%;background:{pct >= 90 ? 'var(--red)' : pct >= 70 ? 'var(--yellow)' : 'var(--green)'}"></div>
      </div>
      <div class="cm">${aiUsage.total_cost_usd.toFixed(4)} / ${aiUsage.budget_limit_usd.toFixed(2)} ({pct.toFixed(1)}%)</div>
    {/if}
    {#if aiUsage.models.length === 0}
      <div class="cm">利用データなし</div>
    {:else}
      {@const maxCost = Math.max(...aiUsage.models.map(m => m.cost_usd), 0.0001)}
      {#each aiUsage.models as m}
        <div class="usage-row">
          <div class="usage-model">{m.model}</div>
          <div class="usage-bar-wrap">
            <div class="usage-bar" style="width:{(m.cost_usd / maxCost * 100).toFixed(1)}%"></div>
          </div>
          <div class="usage-stats">
            <span>${m.cost_usd.toFixed(4)}</span>
            <span class="usage-detail">{m.requests}回 · 入力 {m.input_tokens.toLocaleString()} · 出力 {m.output_tokens.toLocaleString()}</span>
          </div>
        </div>
      {/each}
      <div class="usage-total">
        合計: <strong>${aiUsage.total_cost_usd.toFixed(4)}</strong>
        {#if aiUsage.budget_limit_usd > 0}（残り: ${aiUsage.budget_remaining_usd.toFixed(4)}）{/if}
      </div>
      {#if aiUsage.budget_limit_usd > 0 && aiUsage.budget_remaining_usd <= 0}
        <div class="usage-warn">AI機能は上限に達したため停止中です</div>
      {/if}
    {/if}
  </div>
{/if}

<style>
  h3 { font-size:14px;margin-bottom:12px }
  .fl { display:block;color:var(--overlay);font-size:10px;margin:6px 0 2px }
  .fl input { display:block;width:100%;padding:5px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:11px;margin-top:2px }
  .cm { color:var(--overlay);font-size:10px;margin-bottom:6px }
  .card { background:var(--mantle);border:1px solid var(--surface1);border-radius:6px;padding:12px;margin:8px 0 }
  .ch { font-size:12px;font-weight:700;margin-bottom:4px }
  .row { display:flex;gap:6px }
  .btn-sm { padding:4px 12px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px;cursor:pointer }
  .btn-sm.gb { border-color:var(--green);color:var(--green) }
  .month-sel { padding:4px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px }
  .usage-row { display:flex;flex-direction:column;gap:2px;padding:6px 0;border-bottom:1px solid var(--surface1);font-size:10px }
  .usage-model { font-weight:700;font-size:10px;word-break:break-all }
  .usage-detail { color:var(--overlay);font-size:9px }
  .usage-total { padding:8px 0;font-size:11px;color:var(--text) }
  .usage-warn { padding:6px 10px;border-radius:4px;background:#3a1e1e;color:var(--red);font-size:10px;font-weight:700;margin-top:4px }
  .usage-bar-wrap { height:6px;background:var(--surface0);border-radius:3px;overflow:hidden;margin:2px 0 }
  .usage-bar { height:100%;background:var(--mauve);border-radius:3px;transition:width .3s }
  .usage-stats { display:flex;justify-content:space-between;align-items:center }
  .usage-stats span:first-child { color:var(--green);font-weight:700;font-size:11px }
  .budget-bar-wrap { height:8px;background:var(--surface0);border-radius:4px;overflow:hidden;margin:6px 0 4px }
  .budget-bar { height:100%;border-radius:4px;transition:width .3s }
</style>
