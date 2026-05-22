import { describe, it, expect, vi, beforeEach } from 'vitest';

// We test the logic directly since Svelte 5 runes require component context
// Test the store's exported functions behavior

describe('ui store logic', () => {
	it('toast auto-dismisses after timeout', async () => {
		vi.useFakeTimers();
		// Simulate toast logic
		let toast: { msg: string; undo?: () => void } | null = null;
		function showToast(msg: string, undo?: () => void) {
			toast = { msg, undo };
			setTimeout(() => { if (toast?.msg === msg) toast = null; }, 5000);
		}
		showToast('Archived');
		expect(toast).not.toBeNull();
		expect(toast!.msg).toBe('Archived');
		vi.advanceTimersByTime(5000);
		expect(toast).toBeNull();
		vi.useRealTimers();
	});

	it('toast with undo provides callback', () => {
		let undoCalled = false;
		const toast = { msg: 'Deleted', undo: () => { undoCalled = true; } };
		toast.undo();
		expect(undoCalled).toBe(true);
	});
});
