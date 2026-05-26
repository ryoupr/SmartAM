<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { Line, Doughnut } from 'svelte-chartjs';
  import { Chart, CategoryScale, LinearScale, PointElement, LineElement, ArcElement, Tooltip, Legend, Filler } from 'chart.js';
  import type { AppSettings } from '$lib/store';

  Chart.register(CategoryScale, LinearScale, PointElement, LineElement, ArcElement, Tooltip, Legend, Filler);

  let { settings = $bindable() }: { settings: AppSettings } = $props();

  type UsageSummary = {
    month: string;
    models: { model: string; input_tokens: number; output_tokens: number; cost_usd: number; requests: number }[];
    total_cost_usd: number;
    budget_limit_usd: number;
    budget_remaining_usd: number;
  };
  type DailyCost = { date: string; cost_usd: number; requests: number; is_estimated: boolean };
  type FeatureCost = { feature: string; cost_usd: number; requests: number };
  type HistoryEntry = { timestamp: string; model: string; feature: string; input_tokens: number; output_tokens: number; cost_usd: number };

  let aiUsage: UsageSummary | null = $state(null);
  let usageMonths: string[] = $state([]);
  let selectedMonth = $state('');
  let dailyCosts: DailyCost[] = $state([]);
  let featureCosts: FeatureCost[] = $state([]);
  let history: HistoryEntry[] = $state([]);
  let activeTab: 'overview' | 'history' = $state('overview');
  let loading = $state(false);
  let error: string | null = $state(null);

  const FEATURE_LABELS: Record<string, string> = {
    summarize: '要約', draft_nuances: 'ニュアンス生成',
    draft_reply: '返信文生成', translate: '翻訳', other: 'その他',
  };

  // [中13] CSS変数から色を取得
  function getCssVar(name: string, fallback: string): string {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
  }
  function getChartColors() {
    return {
      mauve: getCssVar('--mauve', '#cba6f7'),
      blue: getCssVar('--blue', '#89b4fa'),
      green: getCssVar('--green', '#a6e3a1'),
      yellow: getCssVar('--yellow', '#f9e2af'),
      red: getCssVar('--red', '#f38ba8'),
      overlay: getCssVar('--overlay', '#a6adc8'),
      text: getCssVar('--text', '#cdd6f4'),
      surface0: getCssVar('--surface0', '#313244'),
    };
  }

  async function fetchAll() {
    loading = true;
    error = null;
    try {
      const months = await invoke<string[]>('get_ai_usage_months');
      usageMonths = months;
      if (!selectedMonth) selectedMonth = months[0] ?? '';
      if (selectedMonth) {
        aiUsage = await invoke<UsageSummary>('get_ai_usage_for_month', { month: selectedMonth });
        featureCosts = await invoke<FeatureCost[]>('get_ai_feature_costs', { month: selectedMonth });
      } else {
        aiUsage = await invoke<UsageSummary>('get_ai_usage');
      }
      dailyCosts = await invoke<DailyCost[]>('get_ai_daily_costs', { days: 30 });
      history = await invoke<HistoryEntry[]>('get_ai_history');
    } catch (e: any) {
      error = e?.message ?? String(e) ?? 'データ取得に失敗しました';
    } finally {
      loading = false;
    }
  }

  async function onMonthChange() {
    if (!selectedMonth) return;
    loading = true;
    try {
      aiUsage = await invoke<UsageSummary>('get_ai_usage_for_month', { month: selectedMonth });
      featureCosts = await invoke<FeatureCost[]>('get_ai_feature_costs', { month: selectedMonth });
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally {
      loading = false;
    }
  }

  async function onBudgetChange() {
    await invoke('set_ai_budget', { limitUsd: settings.aiBudgetLimitUsd ?? 0 });
  }

  onMount(() => { fetchAll(); });

  // [中8] nullガード付きの安全な値取得
  let totalCost = $derived((aiUsage as UsageSummary | null)?.total_cost_usd ?? 0);
  let budgetLimit = $derived((aiUsage as UsageSummary | null)?.budget_limit_usd ?? 0);

  let lineChartData = $derived.by(() => {
    const c = getChartColors();
    return {
      labels: dailyCosts.map(d => d.date.slice(5)),
      datasets: [{
        label: 'コスト (USD)',
        data: dailyCosts.map(d => d.cost_usd),
        borderColor: c.mauve,
        backgroundColor: c.mauve + '1a',
        fill: true,
        tension: 0.3,
        pointRadius: 2,
      }],
    };
  });

  let lineChartOptions = $derived.by(() => {
    const c = getChartColors();
    return {
      responsive: true,
      maintainAspectRatio: false,
      plugins: { legend: { display: false } },
      scales: {
        x: { ticks: { font: { size: 10 }, color: c.overlay }, grid: { color: c.surface0 } },
        y: { ticks: { font: { size: 10 }, color: c.overlay, callback: (v: string | number) => `$${Number(v).toFixed(3)}` }, grid: { color: c.surface0 } },
      },
    };
  });

  let doughnutData = $derived.by(() => {
    const c = getChartColors();
    const colors = [c.mauve, c.blue, c.green, c.yellow, c.red];
    return {
      labels: featureCosts.map(f => FEATURE_LABELS[f.feature] ?? f.feature),
      datasets: [{
        data: featureCosts.map(f => f.cost_usd),
        backgroundColor: colors.slice(0, featureCosts.length),
        borderWidth: 0,
      }],
    };
  });

  let doughnutOptions = $derived.by(() => {
    const c = getChartColors();
    return {
      responsive: true,
      maintainAspectRatio: false,
      plugins: { legend: { position: 'right' as const, labels: { font: { size: 11 }, color: c.text, boxWidth: 12 } } },
    };
  });

  // [低17] $derived化
  let reversedHistory = $derived([...history].reverse());
</script>

<h3>AI 利用状況</h3>
<label class="fl">月額利用上限 (USD)
  <input type="number" step="0.5" min="0" bind:value={settings.aiBudgetLimitUsd} onchange={onBudgetChange} placeholder="0 = 無制限" />
</label>
<div class="cm" style="margin-bottom:12px">{settings.aiBudgetLimitUsd > 0 ? `$${settings.aiBudgetLimitUsd} を超えるとAI機能が停止します` : '上限なし（無制限）'}</div>

<!-- [高4] エラー表示 -->
{#if error}
  <div class="error-msg" role="alert"><span aria-hidden="true">⚠️</span> {error}</div>
{/if}

<div class="row" style="margin-bottom:12px;align-items:center">
  <button class="btn-sm gb" onclick={fetchAll} disabled={loading}>{loading ? '読込中...' : '更新'}</button>
  {#if usageMonths.length > 0}
    <select class="month-sel" bind:value={selectedMonth} onchange={onMonthChange}>
      {#each usageMonths as m}<option value={m}>{m}</option>{/each}
    </select>
  {/if}
  <!-- [高6] ARIAロール -->
  <div class="tab-btns" role="tablist">
    <button role="tab" aria-selected={activeTab === 'overview'} aria-controls="panel-overview" class:active={activeTab === 'overview'} onclick={() => activeTab = 'overview'}>概要</button>
    <button role="tab" aria-selected={activeTab === 'history'} aria-controls="panel-history" class:active={activeTab === 'history'} onclick={() => activeTab = 'history'}>履歴</button>
  </div>
</div>

{#if activeTab === 'overview'}
  <div id="panel-overview" role="tabpanel">
    {#if aiUsage}
      {#if budgetLimit > 0}
        {@const pct = Math.min(100, (totalCost / budgetLimit) * 100)}
        <div class="budget-bar-wrap">
          <div class="budget-bar" style="width:{pct}%;background:{pct >= 90 ? 'var(--red)' : pct >= 70 ? 'var(--yellow)' : 'var(--green)'}"></div>
        </div>
        <div class="cm">${totalCost.toFixed(4)} / ${budgetLimit.toFixed(2)} ({pct.toFixed(1)}%)</div>
      {:else}
        <div class="cm">今月合計: <strong>${totalCost.toFixed(4)}</strong></div>
      {/if}
    {/if}

    {#if dailyCosts.length > 0 && dailyCosts.some(d => d.cost_usd > 0)}
      <div class="card">
        <div class="ch"><span aria-hidden="true">📈</span> 日次コスト推移（30日）{#if dailyCosts.some(d => d.is_estimated)}<span class="estimated">※推定値</span>{/if}</div>
        <div class="chart-wrap">
          <Line data={lineChartData} options={lineChartOptions} />
        </div>
      </div>
    {/if}

    {#if featureCosts.length > 0 && featureCosts.some(f => f.cost_usd > 0)}
      <div class="card">
        <div class="ch"><span aria-hidden="true">🧩</span> 機能別内訳</div>
        <div class="chart-wrap-sm">
          <Doughnut data={doughnutData} options={doughnutOptions} />
        </div>
      </div>
    {/if}

    {#if aiUsage && (aiUsage.models?.length ?? 0) > 0}
      <div class="card">
        <div class="ch"><span aria-hidden="true">🤖</span> モデル別</div>
        {#each aiUsage.models as m}
          {@const maxCost = Math.max(...(aiUsage?.models ?? []).map(x => x.cost_usd ?? 0), 0.0001)}
          <div class="usage-row">
            <div class="usage-model">{m.model}</div>
            <div class="usage-bar-wrap"><div class="usage-bar" style="width:{((m.cost_usd ?? 0) / maxCost * 100).toFixed(1)}%"></div></div>
            <div class="usage-stats">
              <span>${(m.cost_usd ?? 0).toFixed(4)}</span>
              <span class="usage-detail">{m.requests ?? 0}回 · 入{(m.input_tokens ?? 0).toLocaleString()} · 出{(m.output_tokens ?? 0).toLocaleString()}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

{:else}
  <div id="panel-history" role="tabpanel">
    <div class="card">
      <div class="ch"><span aria-hidden="true">📋</span> リクエスト履歴（直近{history.length}件）</div>
      {#if history.length === 0}
        <div class="cm">履歴なし</div>
      {:else}
        <div class="history-list">
          {#each reversedHistory as h}
            <div class="history-item">
              <div class="history-time">{new Date(h.timestamp).toLocaleString('ja-JP', { month:'numeric', day:'numeric', hour:'2-digit', minute:'2-digit' })}</div>
              <div class="history-feat">{FEATURE_LABELS[h.feature] ?? h.feature}</div>
              <div class="history-cost">${(h.cost_usd ?? 0).toFixed(4)}</div>
              <div class="history-tokens">{(h.input_tokens ?? 0) + (h.output_tokens ?? 0)}tk</div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  h3 { font-size:14px;margin-bottom:12px }
  .fl { display:block;color:var(--overlay);font-size:11px;margin:6px 0 2px }
  .fl input { display:block;width:100%;padding:5px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:12px;margin-top:2px }
  .cm { color:var(--overlay);font-size:11px;margin-bottom:6px }
  .error-msg { padding:6px 10px;border-radius:4px;background:var(--surface0);border:1px solid var(--red);color:var(--red);font-size:11px;margin-bottom:8px }
  .card { background:var(--mantle);border:1px solid var(--surface1);border-radius:6px;padding:12px;margin:8px 0 }
  .ch { font-size:12px;font-weight:700;margin-bottom:8px }
  .estimated { font-weight:400;font-size:10px;color:var(--overlay);margin-left:4px }
  .row { display:flex;gap:6px }
  .btn-sm { padding:4px 12px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:11px;cursor:pointer }
  .btn-sm.gb { border-color:var(--green);color:var(--green) }
  .btn-sm:disabled { opacity:0.5;cursor:not-allowed }
  .month-sel { padding:4px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:11px }
  .tab-btns { display:flex;gap:2px;margin-left:auto }
  .tab-btns button { padding:3px 8px;border-radius:3px;border:1px solid var(--surface1);background:var(--surface0);color:var(--overlay);font-size:11px;cursor:pointer }
  .tab-btns button.active, .tab-btns button[aria-selected="true"] { background:var(--surface1);color:var(--text) }
  .chart-wrap { min-height:140px;aspect-ratio:3/1 }
  .chart-wrap-sm { min-height:120px;aspect-ratio:2/1 }
  .usage-row { display:flex;flex-direction:column;gap:2px;padding:5px 0;border-bottom:1px solid var(--surface1);font-size:11px }
  .usage-model { font-weight:700;font-size:11px;word-break:break-all }
  .usage-bar-wrap { height:5px;background:var(--surface0);border-radius:3px;overflow:hidden }
  .usage-bar { height:100%;background:var(--mauve);border-radius:3px }
  .usage-stats { display:flex;justify-content:space-between;align-items:center }
  .usage-stats span:first-child { color:var(--green);font-weight:700;font-size:12px }
  .usage-detail { color:var(--overlay);font-size:11px }
  .budget-bar-wrap { height:8px;background:var(--surface0);border-radius:4px;overflow:hidden;margin:6px 0 4px }
  .budget-bar { height:100%;border-radius:4px;transition:width .3s }
  .history-list { max-height:300px;overflow-y:auto }
  .history-item { display:grid;grid-template-columns:70px 1fr auto auto;gap:6px;padding:4px 0;border-bottom:1px solid var(--surface0);font-size:11px;align-items:center }
  .history-time { color:var(--overlay) }
  .history-feat { color:var(--text) }
  .history-cost { color:var(--green);font-weight:700;text-align:right }
  .history-tokens { color:var(--overlay);text-align:right }
</style>
