<script lang="ts">
  import { check } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';

  let { onError }: { onError: (msg: string) => void } = $props();

  let updateAvailable: { version: string; doUpdate: () => Promise<void> } | null = $state(null);
  let updateProgress: { status: 'downloading' | 'installing'; percent: number } | null = $state(null);

  check().then(update => {
    if (!update) return;
    updateAvailable = {
      version: update.version,
      doUpdate: async () => {
        try {
          updateProgress = { status: 'downloading', percent: 0 };
          let downloaded = 0; let total = 0;
          await update.downloadAndInstall((ev) => {
            if (ev.event === 'Started') total = ev.data.contentLength ?? 0;
            else if (ev.event === 'Progress') { downloaded += ev.data.chunkLength; updateProgress = { status: 'downloading', percent: total > 0 ? Math.round(downloaded / total * 100) : 0 }; }
            else if (ev.event === 'Finished') updateProgress = { status: 'installing', percent: 100 };
          });
          await relaunch();
        } catch (e) { updateProgress = null; onError(`アップデート失敗: ${e}`); }
      }
    };
  }).catch(() => {});
</script>

{#if updateAvailable && !updateProgress}
  <div class="update-bar">
    🚀 v{updateAvailable.version} が利用可能です
    <button class="update-btn" onclick={updateAvailable.doUpdate}>アップデート</button>
    <button class="update-dismiss" onclick={() => updateAvailable = null}>✕</button>
  </div>
{/if}
{#if updateProgress}
  <div class="update-overlay">
    <div class="update-modal">
      <div class="update-icon">⬇️</div>
      <div class="update-title">{updateProgress.status === 'downloading' ? 'アップデートをダウンロード中...' : 'インストール中...'}</div>
      <div class="update-progress-bar"><div class="update-progress-fill" style:width="{updateProgress.percent}%"></div></div>
      <div class="update-percent">{updateProgress.percent}%</div>
    </div>
  </div>
{/if}

<style>
  .update-bar { position:fixed;top:0;left:0;right:0;padding:8px 16px;background:var(--mauve);color:var(--base);font-size:12px;font-weight:700;text-align:center;z-index:200;display:flex;align-items:center;justify-content:center;gap:12px }
  .update-btn { padding:4px 12px;border-radius:4px;border:none;background:var(--base);color:var(--mauve);font-weight:700;font-size:11px;cursor:pointer }
  .update-dismiss { background:none;border:none;color:var(--base);cursor:pointer;font-size:14px;opacity:.7 }
  .update-overlay { position:fixed;inset:0;background:rgba(0,0,0,0.7);z-index:9999;display:flex;align-items:center;justify-content:center }
  .update-modal { background:var(--mantle);border:1px solid var(--surface1);border-radius:12px;padding:32px 48px;text-align:center;min-width:300px }
  .update-icon { font-size:32px;margin-bottom:12px }
  .update-title { font-size:14px;font-weight:700;color:var(--text);margin-bottom:16px }
  .update-progress-bar { height:6px;background:var(--surface0);border-radius:3px;overflow:hidden }
  .update-progress-fill { height:100%;background:var(--mauve);border-radius:3px;transition:width 0.2s }
  .update-percent { font-size:11px;color:var(--overlay);margin-top:8px }
</style>
