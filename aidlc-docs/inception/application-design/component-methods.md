# Component Methods

## Frontend Store Modules

### stores/mail.ts
```typescript
// State
mailList: MailSummary[]
selectedMail: MailDetail | null
activeFolder: string
loading: boolean
hasMore: boolean

// Actions
fetchMails(folder: string): Promise<void>
loadMore(): Promise<void>
selectMail(uid: number): Promise<void>
archiveMail(uid: number): Promise<void>
deleteMail(uid: number): Promise<void>
starMail(uid: number, add: boolean): Promise<void>
searchMails(query: string): Promise<void>
```

### stores/settings.ts
```typescript
// State
settings: AppSettings
// Actions
load(): Promise<void>
save(): Promise<void>
getActiveAccount(): Account | null
getLlmConfig(): LlmConfig
```

### stores/ui.ts
```typescript
// State
toast: ToastMessage | null
composeMode: 'new' | 'reply' | 'forward' | null
showSettings: boolean
// Actions
showToast(msg: string, undo?: () => void): void
dismissToast(): void
```

---

## Backend Modules

### error.rs
```rust
// Frontend-serializable error
#[derive(Serialize)]
pub struct FrontendError {
    pub code: String,      // e.g. "IMAP_CONNECTION_FAILED"
    pub message: String,   // Human-readable
    pub retryable: bool,
}

impl From<ImapError> for FrontendError { ... }
impl From<AiError> for FrontendError { ... }
impl From<AuthError> for FrontendError { ... }
impl From<CalendarError> for FrontendError { ... }
```

### keychain.rs
```rust
pub fn store_credential(service: &str, account: &str, secret: &str) -> Result<(), AuthError>
pub fn get_credential(service: &str, account: &str) -> Result<String, AuthError>
pub fn delete_credential(service: &str, account: &str) -> Result<(), AuthError>
```

### imap_client.rs (async-imap移行後)
```rust
pub async fn fetch_mails(config: &AccountConfig, folder: &str, offset: u32, limit: u32) -> Result<(Vec<MailSummary>, u32), ImapError>
pub async fn fetch_mail_detail(config: &AccountConfig, folder: &str, uid: u32) -> Result<MailDetail, ImapError>
pub async fn search_mails(config: &AccountConfig, folder: &str, query: &str, limit: u32) -> Result<Vec<MailSummary>, ImapError>
pub async fn archive_mail(config: &AccountConfig, folder: &str, uid: u32) -> Result<(), ImapError>
// ... (既存APIシグネチャ維持、戻り値型のみ変更)
```
