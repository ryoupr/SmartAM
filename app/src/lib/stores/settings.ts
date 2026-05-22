import { loadSettings as _load, saveSettings as _save, type AppSettings, DEFAULTS, getLlmConfig } from '$lib/store';

let settings = $state<AppSettings>(structuredClone(DEFAULTS));

export function getSettingsStore() {
  return {
    get settings() { return settings; },
    get activeAccount() { return settings.accounts?.[settings.activeAccountIndex] ?? null; },
    get llmConfig() { return getLlmConfig(settings.llm); },
    async load() { settings = await _load(); },
    async save() { await _save(settings); },
    update(fn: (s: AppSettings) => void) { fn(settings); },
  };
}
