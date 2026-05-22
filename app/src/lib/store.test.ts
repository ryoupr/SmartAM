import { describe, it, expect } from 'vitest';
import { getLlmConfig, formatMailDate, DEFAULT_SHORTCUTS, DEFAULTS, type LlmSettings } from '$lib/store';

describe('getLlmConfig', () => {
	it('returns ollama config with localhost base_url', () => {
		const llm: LlmSettings = { ...DEFAULTS.llm, activeProvider: 'ollama' };
		const config = getLlmConfig(llm);
		expect(config.base_url).toBe('http://localhost:11434');
		expect(config.model).toContain('ollama/');
	});

	it('returns openai config with correct base_url', () => {
		const llm: LlmSettings = { ...DEFAULTS.llm, activeProvider: 'openai' };
		const config = getLlmConfig(llm);
		expect(config.base_url).toBe('https://api.openai.com/v1');
		expect(config.model).toBe(llm.openai.model);
	});

	it('returns anthropic config with correct base_url', () => {
		const llm: LlmSettings = { ...DEFAULTS.llm, activeProvider: 'anthropic' };
		const config = getLlmConfig(llm);
		expect(config.base_url).toBe('https://api.anthropic.com/v1');
	});

	it('returns bedrock config with api_key auth', () => {
		const llm: LlmSettings = {
			...DEFAULTS.llm,
			activeProvider: 'bedrock',
			bedrock: { ...DEFAULTS.llm.bedrock, auth_mode: 'api_key', api_key: 'test-key', region: 'us-west-2' },
		};
		const config = getLlmConfig(llm);
		expect(config.base_url).toContain('us-west-2');
		expect(config.api_key).toBe('test-key');
	});

	it('returns gemini config', () => {
		const llm: LlmSettings = { ...DEFAULTS.llm, activeProvider: 'gemini' };
		const config = getLlmConfig(llm);
		expect(config.base_url).toContain('googleapis.com');
	});
});

describe('formatMailDate', () => {
	it('formats today as time only', () => {
		const now = new Date();
		const result = formatMailDate(now.toISOString(), 'YYYY/MM/DD HH:mm:ss', 'Asia/Tokyo');
		expect(result).toMatch(/^\d{2}:\d{2}$/);
	});

	it('formats past date with full format', () => {
		const result = formatMailDate('2024-01-15T10:30:00Z', 'YYYY/MM/DD HH:mm:ss', 'Asia/Tokyo');
		expect(result).toContain('2024');
		expect(result).toContain('01');
		expect(result).toContain('15');
	});

	it('returns raw string for invalid date', () => {
		const result = formatMailDate('not-a-date', 'YYYY/MM/DD HH:mm:ss', 'UTC');
		expect(result).toBe('not-a-date');
	});
});

describe('DEFAULT_SHORTCUTS', () => {
	it('has all expected keys', () => {
		expect(DEFAULT_SHORTCUTS.nextMail).toBe('j');
		expect(DEFAULT_SHORTCUTS.prevMail).toBe('k');
		expect(DEFAULT_SHORTCUTS.aiSummary).toBe('y');
		expect(DEFAULT_SHORTCUTS.aiDraft).toBe('d');
		expect(DEFAULT_SHORTCUTS.compose).toBe('c');
	});
});
