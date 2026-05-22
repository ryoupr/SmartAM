import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
	plugins: [svelte({ hot: false })],
	test: {
		environment: 'jsdom',
		include: ['src/**/*.test.ts'],
		globals: true,
	},
	resolve: {
		alias: {
			'$lib': '/src/lib',
			'@tauri-apps/api/core': '/src/test/__mocks__/tauri.ts',
			'@tauri-apps/plugin-store': '/src/test/__mocks__/tauri-store.ts',
		},
	},
});
