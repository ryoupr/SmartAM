<script lang="ts">
  let { subject, onConfirm, onCancel }: {
    subject: string;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();
</script>

<div class="dialog-overlay">
  <div class="dialog">
    <div class="dialog-title">メールを削除</div>
    <p class="dialog-msg">「{subject}」を削除しますか？<br/>この操作はゴミ箱に移動します。</p>
    <div class="dialog-actions">
      <button class="btn-cancel" onclick={onCancel}>キャンセル</button>
      <button class="btn-delete" onclick={onConfirm}>削除する</button>
    </div>
  </div>
</div>

<style>
  .dialog-overlay { position:fixed;inset:0;background:rgba(0,0,0,.5);display:flex;align-items:center;justify-content:center;z-index:60 }
  .dialog { background:var(--base);border:1px solid var(--red);border-radius:8px;padding:24px;width:380px;text-align:center }
  .dialog-title { font-size:14px;font-weight:700;margin-bottom:12px }
  .dialog-msg { font-size:12px;color:var(--text);margin-bottom:16px;line-height:1.5 }
  .dialog-actions { display:flex;gap:8px;justify-content:center }
  .btn-cancel { padding:8px 20px;border-radius:6px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);cursor:pointer }
  .btn-delete { padding:8px 20px;border-radius:6px;border:none;background:var(--red);color:var(--base);font-weight:700;cursor:pointer }
</style>
