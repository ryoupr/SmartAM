# API Documentation

## Tauri Commands (Frontend → Backend)

### メール操作
| Command | Purpose | Parameters |
|---------|---------|-----------|
| `fetch_mails` | メール一覧取得 | config, folder, offset, limit |
| `fetch_mail_detail` | メール詳細取得 | config, folder, uid |
| `search_mails` | メール検索 | config, folder, query, limit |
| `archive_mail` | アーカイブ | config, folder, uid |
| `delete_mail` | 削除 | config, folder, uid |
| `star_mail` / `unstar_mail` | スター操作 | config, folder, uid |
| `mark_as_read` | 既読マーク | config, folder, uid |
| `send_mail` | メール送信 | smtp_config, to, cc, bcc, subject, body, attachments |
| `get_folders` | フォルダ一覧 | config |
| `fetch_attachment` | 添付ファイル取得 | config, folder, uid, part_index |

### AI機能
| Command | Purpose | Parameters |
|---------|---------|-----------|
| `ai_summarize` | メール要約 | llm_config, mail_body |
| `ai_draft_nuances` | ニュアンス提案 | llm_config, mail_body |
| `ai_draft_reply` | 返信文生成 | llm_config, mail_body, nuance, instruction |
| `ai_translate` | 翻訳 | llm_config, text, target_lang |
| `ai_detect_events` | 日程検出 | llm_config, mail_body |

### カレンダー
| Command | Purpose | Parameters |
|---------|---------|-----------|
| `register_calendar_event` | イベント登録 | provider, calendar_name, title, start, end, location |
| `list_calendars` | カレンダー一覧 | provider |
| `check_conflicts` | 重複チェック | provider, calendar_name, start, end |

### OAuth
| Command | Purpose | Parameters |
|---------|---------|-----------|
| `start_oauth` | OAuth開始 | — |
| `refresh_token` | トークンリフレッシュ | refresh_token |

### AI使用量
| Command | Purpose | Parameters |
|---------|---------|-----------|
| `get_ai_usage` | 使用量取得 | — |
| `get_ai_usage_monthly` | 月次集計 | year, month |

### その他
| Command | Purpose | Parameters |
|---------|---------|-----------|
| `frontend_trace` | フロントエンドログ | tag, msg |

## Data Models

### MailSummary
- `uid: u32` — メールUID
- `from: String` — 送信者
- `subject: String` — 件名
- `date: String` — 日時
- `seen: bool` — 既読フラグ

### MailDetail
- `uid: u32`, `from`, `to`, `subject`, `date`
- `body_text: String` — プレーンテキスト本文
- `body_html: String` — HTML本文
- `attachments: Vec<Attachment>` — 添付ファイル一覧

### AccountConfig
- `email`, `auth_type`, `password`, `access_token`, `imap_host`, `imap_port`

### LlmConfig
- `base_url`, `model`, `api_key`
