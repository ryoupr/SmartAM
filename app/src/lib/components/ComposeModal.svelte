<script lang="ts">
  let { mode = 'new', to = '', subject = '', body = '', signature = '', onClose, onSend, attachmentPaths = $bindable([]) }: {
    mode?: 'new' | 'reply' | 'forward';
    to?: string;
    subject?: string;
    body?: string;
    signature?: string;
    onClose: () => void;
    onSend: (data: { to: string; cc: string; bcc: string; subject: string; body: string }) => void;
    attachmentPaths?: string[];
  } = $props();

  let toField = $state(to);
  let ccField = $state('');
  let bccField = $state('');
  let subjectField = $state(
    mode === 'reply' ? `Re: ${subject}` : mode === 'forward' ? `Fwd: ${subject}` : subject
  );
  let bodyField = $state(body);
  let sending = $state(false);

  const title = mode === 'reply' ? '返信作成' : mode === 'forward' ? '転送' : '新規メール作成';

  function handleSend() {
    sending = true;
    onSend({ to: toField, cc: ccField, bcc: bccField, subject: subjectField, body: bodyField });
  }

  async function addAttachment() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({ multiple: true });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        attachmentPaths = [...attachmentPaths, ...paths.filter((p): p is string => typeof p === 'string')];
      }
    } catch (e) { console.error('file dialog error:', e); }
  }

  function removeAttachment(i: number) {
    attachmentPaths = attachmentPaths.filter((_, idx) => idx !== i);
  }
</script>

<div class="compose-overlay" role="dialog">
  <div class="compose-window">
    <div class="compose-header">{title}</div>
    <div class="fields">
      <label>To: <input bind:value={toField} placeholder="宛先を入力..." /></label>
      <label>CC: <input bind:value={ccField} /></label>
      <label>BCC: <input bind:value={bccField} /></label>
      <label>Subject: <input bind:value={subjectField} /></label>
    </div>
    <textarea class="body-editor" bind:value={bodyField} placeholder="本文を入力..."></textarea>
    {#if signature}
      <div class="signature">{signature}</div>
    {/if}
    <div class="compose-actions">
      <button class="btn-attach" onclick={addAttachment}>📎 添付</button>
      {#each attachmentPaths as path, i}
        <span class="att-tag">📄 {path.split('/').pop()} <button class="att-rm" onclick={() => removeAttachment(i)}>✕</button></span>
      {/each}
      <div class="spacer"></div>
      <button class="btn-cancel" onclick={onClose}>キャンセル</button>
      <button class="btn-send" disabled={sending} onclick={handleSend}>📤 送信</button>
    </div>
  </div>
</div>

<style>
  .compose-overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,0.5);
    display: flex; align-items: center; justify-content: center; z-index: 50;
  }
  .compose-window {
    width: 900px; max-height: 90vh; background: var(--base);
    border: 1px solid var(--surface1); border-radius: 8px;
    display: flex; flex-direction: column;
  }
  .compose-header {
    padding: 10px 16px; font-weight: 700; font-size: 13px;
    border-bottom: 1px solid var(--surface1);
  }
  .fields { padding: 8px 16px; display: flex; flex-direction: column; gap: 4px; }
  .fields label { display: flex; align-items: center; gap: 8px; color: var(--overlay); font-size: 11px; }
  .fields input {
    flex: 1; padding: 5px 8px; border-radius: 4px; border: none;
    background: var(--surface0); color: var(--text); font-size: 11px;
  }
  .body-editor {
    flex: 1; min-height: 200px; margin: 8px 16px; padding: 12px;
    border-radius: 6px; border: 1px solid var(--surface1);
    background: var(--mantle); color: var(--text); font-size: 12px;
    resize: none; font-family: inherit;
  }
  .signature { padding: 0 28px 8px; color: var(--overlay); font-size: 10px; white-space: pre-line; }
  .compose-actions {
    display: flex; align-items: center; padding: 10px 16px; gap: 8px;
    border-top: 1px solid var(--surface1);
  }
  .spacer { flex: 1; }
  .btn-attach { padding: 8px 14px; border-radius: 6px; border: 1px solid var(--surface1); background: var(--surface0); color: var(--text); cursor: pointer; font-size: 11px; }
  .btn-cancel { padding: 8px 20px; border-radius: 6px; border: 1px solid var(--surface1); background: var(--surface0); color: var(--text); cursor: pointer; font-size: 11px; }
  .btn-send { padding: 8px 20px; border-radius: 6px; border: none; background: var(--green); color: var(--base); font-weight: 700; cursor: pointer; font-size: 13px; }
  .btn-send:disabled { opacity: 0.6; }
  .att-tag { display:inline-flex;align-items:center;gap:4px;padding:2px 8px;border-radius:4px;background:var(--surface0);color:var(--text);font-size:10px;border:1px solid var(--surface1) }
  .att-rm { background:none;border:none;color:var(--overlay);cursor:pointer;font-size:10px }
</style>
