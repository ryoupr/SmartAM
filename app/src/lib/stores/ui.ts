let toast = $state<{ msg: string; undo?: () => void } | null>(null);
let composeMode = $state<'new' | 'reply' | 'forward' | null>(null);
let composeBody = $state('');
let showSettings = $state(false);
let confirmDelete = $state(false);

export function getUiStore() {
  return {
    get toast() { return toast; },
    get composeMode() { return composeMode; },
    get composeBody() { return composeBody; },
    get showSettings() { return showSettings; },
    get confirmDelete() { return confirmDelete; },
    showToast(msg: string, undo?: () => void) {
      toast = { msg, undo };
      setTimeout(() => { if (toast?.msg === msg) toast = null; }, 5000);
    },
    dismissToast() { toast = null; },
    openCompose(mode: 'new' | 'reply' | 'forward', body = '') { composeMode = mode; composeBody = body; },
    closeCompose() { composeMode = null; composeBody = ''; },
    toggleSettings() { showSettings = !showSettings; },
    setConfirmDelete(v: boolean) { confirmDelete = v; },
  };
}
