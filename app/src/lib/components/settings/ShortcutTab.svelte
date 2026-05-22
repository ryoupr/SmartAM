<script lang="ts">
  import { DEFAULT_SHORTCUTS, type AppSettings } from '$lib/store';

  let { settings = $bindable() }: { settings: AppSettings } = $props();

  let recordingAction: string | null = $state(null);
  let scConflict = $state('');

  const groups: [string, [string, string][]][] = [
    ['ナビゲーション', [['nextMail','次のメール'],['prevMail','前のメール'],['openMail','メールを開く'],['backToList','一覧に戻る']]],
    ['フォルダ移動', [['goInbox','受信トレイ'],['goStarred','スター付き'],['goSent','送信済み'],['goDrafts','下書き'],['goAll','すべてのメール']]],
    ['メール操作', [['reply','返信'],['forward','転送'],['archive','アーカイブ'],['delete','削除'],['star','スター切替'],['undo','元に戻す']]],
    ['AI機能', [['aiSummary','📝 要約'],['aiDraft','✍ 返信下書き'],['aiTranslate','🌐 翻訳'],['aiCalendar','📅 カレンダー']]],
    ['その他', [['compose','新規作成'],['search','検索'],['help','ショートカット一覧']]],
  ];
</script>

<h3>キーボードショートカット</h3>
<div class="sc-hint">キーの欄をクリックして新しいキーを入力</div>
<button class="btn-sm dn" onclick={() => { settings.shortcuts = { ...DEFAULT_SHORTCUTS }; scConflict = ''; }}>デフォルトに戻す</button>
{#each groups as [groupName, items]}
  <div class="sc-cat">{groupName}</div>
  {#each items as [action, label]}
    <div class="sc-row">
      <span class="sc-label">{label}</span>
      <button
        class="sc-key" class:recording={recordingAction === action}
        onclick={(e) => { recordingAction = action; scConflict = ''; e.currentTarget.focus(); }}
        onkeydown={(e) => {
          if (recordingAction !== action) return;
          e.preventDefault(); e.stopPropagation();
          const k = e.key;
          if (k === 'Escape') { recordingAction = null; return; }
          const dup = Object.entries(settings.shortcuts).find(([a, v]) => v === k && a !== action);
          if (dup) { scConflict = `「${k}」は既に「${dup[0]}」に割り当て済み`; return; }
          settings.shortcuts[action] = k;
          scConflict = '';
          recordingAction = null;
        }}
      >{recordingAction === action ? '⌨ キーを押す...' : settings.shortcuts[action]}</button>
    </div>
  {/each}
{/each}
{#if scConflict}<div class="sc-conflict">⚠ {scConflict}</div>{/if}

<style>
  h3 { font-size:14px;margin-bottom:12px }
  .sc-hint { color:var(--overlay);font-size:10px;margin-bottom:8px }
  .sc-cat { color:var(--mauve);font-size:11px;font-weight:700;margin:12px 0 4px }
  .sc-row { display:flex;align-items:center;justify-content:space-between;padding:3px 0 }
  .sc-label { font-size:11px;color:var(--text) }
  .sc-key { min-width:80px;padding:4px 12px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:11px;font-weight:700;text-align:center;cursor:pointer }
  .sc-key.recording { border-color:var(--mauve);color:var(--mauve);font-weight:400;font-style:italic }
  .sc-conflict { margin-top:8px;padding:6px 10px;border-radius:4px;background:#3a1e1e;color:var(--red);font-size:10px }
  .btn-sm { padding:4px 12px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px;cursor:pointer }
  .btn-sm.dn { color:var(--red);border-color:var(--red) }
</style>
