// Mock for @tauri-apps/api/core
const handlers: Record<string, (...args: any[]) => any> = {};

export function invoke(cmd: string, args?: any): Promise<any> {
	if (handlers[cmd]) return Promise.resolve(handlers[cmd](args));
	return Promise.resolve(null);
}

// Test helper: register mock command handlers
export function __mockCommand(cmd: string, handler: (...args: any[]) => any) {
	handlers[cmd] = handler;
}

export function __clearMocks() {
	Object.keys(handlers).forEach(k => delete handlers[k]);
}
