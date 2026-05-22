import { describe, it, expect } from 'vitest';
import { getImapConfig, getSmtpConfig, type Account, DEFAULT_ACCOUNT_EXTRAS } from '$lib/store';

describe('getImapConfig', () => {
	const account: Account = {
		email: 'test@example.com',
		auth_type: 'password',
		password: 'secret',
		access_token: '',
		refresh_token: '',
		token_expires_at: 0,
		imap_host: 'imap.example.com',
		imap_port: 993,
		smtp_host: 'smtp.example.com',
		smtp_port: 587,
		signature: '',
		...DEFAULT_ACCOUNT_EXTRAS,
	};

	it('returns correct IMAP config', () => {
		const config = getImapConfig(account);
		expect(config.email).toBe('test@example.com');
		expect(config.imap_host).toBe('imap.example.com');
		expect(config.imap_port).toBe(993);
		expect(config.auth_type).toBe('password');
		expect(config.password).toBe('secret');
	});

	it('returns oauth access_token when auth_type is oauth', () => {
		const oauthAccount = { ...account, auth_type: 'oauth' as const, access_token: 'token123' };
		const config = getImapConfig(oauthAccount);
		expect(config.auth_type).toBe('oauth');
		expect(config.access_token).toBe('token123');
	});
});

describe('getSmtpConfig', () => {
	const account: Account = {
		email: 'test@example.com',
		auth_type: 'password',
		password: 'secret',
		access_token: '',
		refresh_token: '',
		token_expires_at: 0,
		imap_host: 'imap.example.com',
		imap_port: 993,
		smtp_host: 'smtp.example.com',
		smtp_port: 587,
		signature: '',
		...DEFAULT_ACCOUNT_EXTRAS,
	};

	it('returns correct SMTP config', () => {
		const config = getSmtpConfig(account);
		expect(config.email).toBe('test@example.com');
		expect(config.smtp_host).toBe('smtp.example.com');
		expect(config.smtp_port).toBe(587);
	});
});
