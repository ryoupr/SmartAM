<script lang="ts">
  let { toast, error, onDismissToast, onDismissError }: {
    toast: { msg: string; undo?: () => void } | null;
    error: string | null;
    onDismissToast: () => void;
    onDismissError: () => void;
  } = $props();
</script>

{#if toast}
  <div class="toast">
    {toast.msg}
    {#if toast.undo}
      <button class="undo" onclick={() => { toast?.undo?.(); onDismissToast(); }}>取り消す</button>
    {/if}
  </div>
{/if}
{#if error}
  <div class="toast err">
    {error}
    <button class="dismiss" onclick={onDismissError}>✕</button>
  </div>
{/if}

<style>
  .toast {
    position: fixed; bottom: 24px; right: 24px;
    padding: 12px 16px; border-radius: 8px;
    background: var(--ink, #1a1410); color: var(--cream, #f6efe4);
    border-left: 4px solid var(--ai-blue, var(--blue));
    font-size: 12px; z-index: 100;
    display: flex; gap: 12px; align-items: center;
    box-shadow: var(--shadow-floating, 0 16px 36px rgba(26,20,16,0.18));
    animation: slide-in 300ms ease-out;
  }
  .toast.err {
    border-left-color: var(--red);
    bottom: 72px;
  }
  .undo {
    background: none; border: none;
    color: var(--ai-blue, var(--blue));
    cursor: pointer; font-size: 11px; font-weight: 600;
    text-decoration: underline;
  }
  .dismiss {
    background: none; border: none;
    color: var(--cream, #f6efe4); cursor: pointer;
    font-size: 14px; margin-left: 8px;
  }
  @keyframes slide-in {
    from { transform: translateY(20px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
</style>
