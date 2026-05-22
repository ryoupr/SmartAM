<script lang="ts">
  import Icon from './Icon.svelte';

  let { label, icon, kbd, variant = '', active = false, disabled = false, title, onclick }: {
    label?: string;
    icon?: string;
    kbd?: string;
    variant?: string;
    active?: boolean;
    disabled?: boolean;
    title?: string;
    onclick?: (e: MouseEvent) => void;
  } = $props();
</script>

<button class="btn {variant}" class:active {disabled} {title} {onclick}>
  {#if icon}<Icon name={icon} size={16} />{/if}
  {#if label}<span class="label">{label}</span>{/if}
  {#if kbd}<span class="kbd">{kbd}</span>{/if}
</button>

<style>
  .btn {
    display: inline-flex; align-items: center; gap: 8px;
    padding: 8px 12px; border-radius: 6px;
    border: 1.5px solid var(--line-2, var(--surface1));
    background: var(--paper-wh, var(--surface0));
    color: var(--ink, var(--text));
    font: 500 13px/1 'Inter', sans-serif; letter-spacing: -0.005em;
    cursor: pointer; transition: background .15s, border-color .15s;
  }
  .btn:hover { background: var(--bone, var(--surface1)); border-color: var(--ink-40, var(--subtext)); }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }

  /* Variants */
  .btn.primary { background: var(--ink, var(--base)); color: var(--cream, var(--text)); border-color: var(--ink, var(--surface1)); }
  .btn.primary:hover { background: var(--red); border-color: var(--red); }
  .btn.danger { color: var(--red); border-color: rgba(200,40,30,0.30); }
  .btn.danger:hover { background: var(--red); color: var(--cream, #fff); }
  .btn.starred { color: var(--ai-yellow, var(--yellow)); border-color: rgba(160,111,10,0.40); }
  .btn.ai-summary { color: var(--ai-green, var(--green)); border-color: rgba(47,122,59,0.35); }
  .btn.ai-summary.active { background: var(--ai-green, var(--green)); color: var(--cream, var(--base)); }
  .btn.ai-draft { color: var(--ai-yellow, var(--yellow)); border-color: rgba(160,111,10,0.35); }
  .btn.ai-draft.active { background: var(--ai-yellow-bg, var(--yellow)); }
  .btn.ai-translate { color: var(--ai-blue, var(--blue)); border-color: rgba(29,95,179,0.35); }
  .btn.ai-translate.active { background: var(--ai-blue-bg, var(--blue)); }
  .btn.ai-calendar { color: var(--ai-blue, var(--blue)); border-color: rgba(29,95,179,0.35); }
  .btn.ai-calendar.active { background: var(--ai-blue-bg, var(--blue)); }
  .btn.ai-regen { color: var(--ai-mauve, var(--mauve)); border-color: rgba(124,62,181,0.35); }
  .btn.ghost { border-color: transparent; background: transparent; color: var(--ink-80, var(--subtext)); }
  .btn.ghost:hover { background: var(--bone, var(--surface0)); }

  .label { white-space: nowrap; }
  .kbd {
    font: 500 10px 'JetBrains Mono', monospace;
    padding: 1px 5px; background: var(--cream, var(--surface0));
    border: 1px solid var(--line, var(--surface1)); border-radius: 3px;
    color: var(--ink-60, var(--overlay));
  }
</style>
