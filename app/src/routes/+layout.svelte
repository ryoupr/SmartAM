<script>
  import '../app.css';
  import { onMount } from 'svelte';
  import { loadSettings } from '$lib/store';

  let { children } = $props();

  onMount(async () => {
    const s = await loadSettings();
    if (s.theme) document.documentElement.setAttribute('data-theme', s.theme);
    else document.documentElement.setAttribute('data-theme', 'light');
  });
</script>

<div class="app">
  {@render children()}
</div>

<style>
  .app { height: 100vh; display: flex; flex-direction: column; }
</style>
