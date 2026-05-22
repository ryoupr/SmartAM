<script lang="ts">
  import type { ShortcutMap } from '$lib/store';

  let { shortcuts, onAction, disabled }: {
    shortcuts: ShortcutMap;
    onAction: (action: string) => void;
    disabled: boolean;
  } = $props();

  let gPending = $state(false);
  let gTimer: ReturnType<typeof setTimeout> | null = null;

  function reverseMap(): Map<string, string> {
    const m = new Map<string, string>();
    for (const [action, key] of Object.entries(shortcuts)) m.set(key, action);
    return m;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (disabled) return;
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

    const key = e.key;

    if (gPending) {
      gPending = false;
      if (gTimer) { clearTimeout(gTimer); gTimer = null; }
      const combo = `g ${key}`;
      const action = reverseMap().get(combo);
      if (action) { e.preventDefault(); onAction(action); }
      return;
    }

    if (key === 'g' && Object.values(shortcuts).some(v => v.startsWith('g '))) {
      gPending = true;
      gTimer = setTimeout(() => { gPending = false; }, 1000);
      return;
    }

    if (key === 'ArrowDown' || key === 'ArrowUp') {
      e.preventDefault();
      onAction(key === 'ArrowDown' ? 'nextMail' : 'prevMail');
      return;
    }

    const action = reverseMap().get(key);
    if (action) { e.preventDefault(); onAction(action); }
  }
</script>

<svelte:window onkeydown={handleKeydown} />
