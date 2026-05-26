<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import type { AppSettings } from '$lib/store';

  let { settings = $bindable() }: { settings: AppSettings } = $props();

  let permissionStatus = $state('確認中...');
  let permissionChecking = $state(false);
  let testResult = $state('');

  const SYSTEM_SOUNDS = ['default', 'Basso', 'Blow', 'Bottle', 'Frog', 'Funk', 'Glass', 'Hero', 'Morse', 'Ping', 'Pop', 'Purr', 'Sosumi', 'Submarine', 'Tink'];

  const isCustom = $derived(
    settings.notificationSoundName !== '' && !SYSTEM_SOUNDS.includes(settings.notificationSoundName ?? 'default')
  );

  async function checkPermission() {
    permissionChecking = true;
    permissionStatus = '🔄 確認中...';
    try {
      const { isPermissionGranted } = await import('@tauri-apps/plugin-notification');
      const granted = await isPermissionGranted();
      permissionStatus = granted ? '✅ 許可済み' : '❌ 未許可';
    } catch (e) { permissionStatus = `⚠️ 確認失敗: ${e}`; }
    permissionChecking = false;
  }

  async function requestPermission() {
    try {
      const { requestPermission } = await import('@tauri-apps/plugin-notification');
      await requestPermission();
      await checkPermission();
    } catch (e) { permissionStatus = `⚠️ リクエスト失敗: ${e}`; }
  }

  async function sendTestNotification() {
    testResult = '⏳ 送信中...';
    try {
      await invoke('send_test_notification', { sound: settings.notificationSoundName ?? 'default' });
      testResult = '✅ 送信しました';
    } catch (e) { testResult = `❌ エラー: ${e}`; }
    setTimeout(() => { testResult = ''; }, 5000);
  }

  async function previewSound() {
    const sound = settings.notificationSoundName ?? 'default';
    if (!sound) return;
    await invoke('preview_sound', { name: sound });
  }

  async function pickCustomFile() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const path = await open({ filters: [{ name: 'Audio', extensions: ['aiff', 'wav', 'mp3', 'caf', 'm4a'] }] });
      if (path) {
        settings.notificationSoundName = path as string;
        await invoke('preview_sound', { name: path as string });
      }
    } catch {}
  }

  function onSoundSelect(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    if (val === '__custom__') {
      pickCustomFile();
    } else {
      settings.notificationSoundName = val;
      previewSound();
    }
  }

  onMount(() => { checkPermission(); });
</script>

<h3>通知</h3>

<section class="section">
  <h4>通知権限</h4>
  <div class="row">
    <span class="status">{permissionStatus}</span>
    {#if permissionStatus === '❌ 未許可'}
      <button class="bd" onclick={requestPermission}>権限をリクエスト</button>
    {/if}
    <button class="bd" onclick={checkPermission} disabled={permissionChecking}>再確認</button>
  </div>
  <p class="hint">macOSの「システム設定 &gt; 通知」からも変更できます</p>
</section>

<section class="section">
  <h4>通知音</h4>
  <div class="row">
    <select value={isCustom ? '__custom__' : (settings.notificationSoundName ?? 'default')} onchange={onSoundSelect}>
      {#each SYSTEM_SOUNDS as s}
        <option value={s}>{s === 'default' ? 'デフォルト' : s}</option>
      {/each}
      <option value="">なし（サイレント）</option>
      <option value="__custom__">カスタム...</option>
    </select>
    {#if isCustom}
      <span class="custom-path" title={settings.notificationSoundName}>🎵 {settings.notificationSoundName?.split('/').pop()}</span>
      <button class="bd" onclick={pickCustomFile}>変更</button>
      <button class="bd" onclick={previewSound}>▶</button>
    {/if}
  </div>
</section>

<section class="section">
  <h4>通知テスト</h4>
  <div class="row">
    <button class="bd primary" onclick={sendTestNotification}>テスト通知を送信</button>
    {#if testResult}<span class="status">{testResult}</span>{/if}
  </div>
</section>

<style>
  h3 { margin:0 0 16px }
  h4 { margin:0 0 8px;font-size:12px;color:var(--overlay) }
  .section { margin-bottom:20px;padding:12px;background:var(--surface0);border-radius:8px }
  .row { display:flex;align-items:center;gap:8px;flex-wrap:wrap }
  .status { font-size:12px }
  .hint { font-size:10px;color:var(--overlay);margin:6px 0 0 }
  .bd { font-size:11px;padding:4px 10px;border-radius:4px;border:1px solid var(--surface1);background:var(--mantle);color:var(--text);cursor:pointer }
  .bd:disabled { opacity:0.5;cursor:default }
  .bd.primary { background:var(--blue);color:#fff;border-color:var(--blue) }
  select { font-size:11px;padding:4px 8px;border-radius:4px;border:1px solid var(--surface1);background:var(--mantle);color:var(--text) }
  .custom-path { font-size:10px;color:var(--overlay);max-width:150px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap }
</style>
