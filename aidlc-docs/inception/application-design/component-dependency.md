# Component Dependencies

## Dependency Diagram

```mermaid
graph TD
    subgraph Frontend
        Page["+page.svelte<br/>(thin orchestrator)"]
        SM[stores/mail.ts]
        SS[stores/settings.ts]
        SA[stores/ai.ts]
        SU[stores/ui.ts]
        Sidebar[Sidebar]
        ML[MailList]
        MD[MailDetail]
        Settings[Settings tabs]
        Compose[ComposeModal]
        Shortcuts[ShortcutManager]
    end

    subgraph Backend
        Lib[lib.rs<br/>(commands)]
        IMAP[imap_client]
        SMTP[smtp_client]
        AI[ai_client]
        Usage[ai_usage]
        OAuth[oauth]
        Cal[calendar]
        KC[keychain]
        Err[error types]
    end

    Page --> SM & SS & SA & SU
    Page --> Sidebar & ML & MD & Settings & Compose & Shortcuts
    ML --> SM
    MD --> SA
    Settings --> SS
    Compose --> SM

    SM -->|invoke| Lib
    SS -->|invoke| Lib
    SA -->|invoke| Lib

    Lib --> IMAP & SMTP & AI & OAuth & Cal
    AI --> Usage
    OAuth --> KC
    Lib --> KC
    IMAP --> Err
    AI --> Err
    OAuth --> Err
    Cal --> Err
```

## Communication Patterns

| From | To | Pattern | Data |
|------|----|---------|------|
| Component | Store | Function call | Action params |
| Store | Backend | `invoke()` IPC | JSON serialized |
| Backend → Frontend | Return | Result<T, FrontendError> | JSON |
| Backend modules | error.rs | `?` operator + From impl | Error conversion |
| keychain.rs | macOS Keychain | `security` CLI or `security-framework` crate | Credential CRUD |

## Data Flow (Refactored)

```
User Action → Component → Store Action → invoke() → Rust Command → Module → External Service
                                                          ↓
                                              Result<T, FrontendError>
                                                          ↓
                                              Store State Update → Reactive UI Update
```

## Key Design Decisions

1. **invoke()はstore内に閉じ込める** — コンポーネントから直接invokeしない
2. **エラーはFrontendError型で統一** — フロントエンドは`code`フィールドでハンドリング分岐
3. **認証情報はKeychain経由** — settings.jsonには認証情報を保存しない
4. **+page.svelteはthin orchestrator** — ロジックはstore、UIはコンポーネントに委譲
