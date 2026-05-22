<script lang="ts">
  let { toast, error, onDismissToast, onDismissError }: {
    toast: { msg: string; undo?: () => void } | null;
    error: string | null;
    onDismissToast: () => void;
    onDismissError: () => void;
  } = $props();
</script>

{#if toast}
  <div class="toast-bar">
    {toast.msg}
    {#if toast.undo}
      <button class="undo" onclick={() => { toast?.undo?.(); onDismissToast(); }}>元に戻す</button>
    {/if}
  </div>
{/if}
{#if error}
  <div class="toast-err">{error} <button onclick={onDismissError}>✕</button></div>
{/if}

<style>
  .toast-bar { position:fixed;bottom:16px;right:16px;padding:10px 16px;background:#1e3a2e;color:var(--green);border:1px solid var(--green);border-radius:8px;font-size:12px;z-index:100;display:flex;gap:12px;align-items:center }
  .undo { background:none;border:none;color:var(--blue);cursor:pointer;font-size:11px;text-decoration:underline }
  .toast-err { position:fixed;bottom:56px;right:16px;padding:10px 16px;background:#3a1e1e;color:var(--red);border:1px solid var(--red);border-radius:8px;font-size:12px;z-index:100 }
  .toast-err button { background:none;border:none;color:var(--red);cursor:pointer;margin-left:8px }
</style>
