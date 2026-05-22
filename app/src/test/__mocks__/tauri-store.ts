// Mock for @tauri-apps/plugin-store
const store: Record<string, any> = {};

export async function load(_path: string, _opts?: any) {
	return {
		get: async (key: string) => store[key] ?? null,
		set: async (key: string, value: any) => { store[key] = value; },
		save: async () => {},
	};
}
