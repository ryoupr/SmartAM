use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{AccountConfig, Attachment, FolderInfo, MailDetail, MailSummary};

type ImapSession = imap::Session<native_tls::TlsStream<std::net::TcpStream>>;

static POOL: LazyLock<Mutex<HashMap<String, ImapSession>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

const CACHE_HARD_LIMIT_BYTES: usize = 1_073_741_824; // 1 GB

/// LRU mail cache with configurable max count + hard memory limit
struct MailCache {
    map: HashMap<u32, MailDetail>,
    order: VecDeque<u32>,
    estimated_bytes: usize,
}

impl MailCache {
    fn new() -> Self { Self { map: HashMap::new(), order: VecDeque::new(), estimated_bytes: 0 } }

    fn estimate_size(detail: &MailDetail) -> usize {
        detail.body_text.len() + detail.body_html.len() + detail.from.len()
            + detail.to.len() + detail.subject.len() + detail.date.len()
            + detail.attachments.len() * 64 + 128 // overhead
    }

    fn get(&mut self, uid: &u32) -> Option<&MailDetail> {
        if self.map.contains_key(uid) {
            self.order.retain(|u| u != uid);
            self.order.push_back(*uid);
            self.map.get(uid)
        } else {
            None
        }
    }

    fn insert(&mut self, uid: u32, detail: MailDetail) {
        let max = CACHE_MAX.load(Ordering::Relaxed);
        let entry_size = Self::estimate_size(&detail);
        if self.map.contains_key(&uid) {
            if let Some(old) = self.map.get(&uid) {
                self.estimated_bytes = self.estimated_bytes.saturating_sub(Self::estimate_size(old));
            }
            self.order.retain(|u| *u != uid);
        }
        self.map.insert(uid, detail);
        self.order.push_back(uid);
        self.estimated_bytes += entry_size;
        self.evict(max);
    }

    fn evict(&mut self, max_count: usize) {
        while (self.map.len() > max_count || self.estimated_bytes > CACHE_HARD_LIMIT_BYTES)
            && !self.order.is_empty()
        {
            if let Some(old) = self.order.pop_front() {
                if let Some(removed) = self.map.remove(&old) {
                    self.estimated_bytes = self.estimated_bytes.saturating_sub(Self::estimate_size(&removed));
                }
            }
        }
    }

    fn contains_key(&self, uid: &u32) -> bool { self.map.contains_key(uid) }
}

static CACHE: LazyLock<Mutex<MailCache>> = LazyLock::new(|| Mutex::new(MailCache::new()));
static CACHE_MAX: AtomicUsize = AtomicUsize::new(100);

/// Cache for ICS attachment data (key: "uid:part_index", value: base64-encoded data)
static ICS_CACHE: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn set_cache_max(max: usize) {
    CACHE_MAX.store(max, Ordering::Relaxed);
    let mut cache = CACHE.lock().unwrap();
    cache.evict(max);
    crate::trace::trace("IMAP", &format!("cache max set to {}, bytes: {}KB", max, cache.estimated_bytes / 1024));
}

static FOLDER_MAP: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn resolve_folder(folder: &str) -> String {
    let map = FOLDER_MAP.lock().unwrap();
    map.get(folder).cloned().unwrap_or_else(|| folder.to_string())
}

/// Discover Gmail special-use folders via IMAP LIST and populate FOLDER_MAP.
fn discover_folders(session: &mut ImapSession) -> Result<(), String> {
    let folders = session.list(Some(""), Some("*"))
        .map_err(|e| format!("フォルダ一覧取得失敗: {e}"))?;

    let mut map = FOLDER_MAP.lock().unwrap();
    map.insert("INBOX".to_string(), "INBOX".to_string());

    for f in folders.iter() {
        let name = f.name().to_string();
        let attrs = format!("{:?}", f.attributes());
        if attrs.contains("All") { map.insert("ALL".to_string(), name.clone()); map.insert("STARRED".to_string(), name.clone()); }
        if attrs.contains("Sent") { map.insert("SENT".to_string(), name.clone()); }
        if attrs.contains("Drafts") { map.insert("DRAFTS".to_string(), name.clone()); }
        if attrs.contains("Junk") { map.insert("SPAM".to_string(), name.clone()); }
        if attrs.contains("Trash") { map.insert("TRASH".to_string(), name.clone()); }
        if attrs.contains("Flagged") { /* STARRED already mapped to All */ }
    }

    crate::trace::trace("IMAP", &format!("discover_folders: {:?}", *map));
    Ok(())
}

fn decode_rfc2047(raw: &[u8]) -> String {
    let lossy = String::from_utf8_lossy(raw).to_string();
    if lossy.contains("=?") {
        match mailparse::parse_header(format!("Subject: {}", lossy).as_bytes()) {
            Ok((hdr, _)) => hdr.get_value(),
            Err(_) => lossy,
        }
    } else {
        lossy
    }
}

fn connect_sync(config: &AccountConfig) -> Result<ImapSession, String> {
    crate::trace::trace("IMAP", &format!("connect: {}:{}", config.imap_host, config.imap_port));
    let tls = native_tls::TlsConnector::new().map_err(|e| format!("TLS作成失敗: {e}"))?;
    let client = imap::connect(
        (config.imap_host.as_str(), config.imap_port),
        &config.imap_host,
        &tls,
    ).map_err(|e| format!("接続失敗: {e}"))?;

    if config.auth_type == "oauth" {
        crate::trace::trace("IMAP", "connect: XOAUTH2...");
        let auth_str = format!("user={}\x01auth=Bearer {}\x01\x01", config.email, config.access_token);
        let session = client.authenticate("XOAUTH2", &XOAuth2(auth_str))
            .map_err(|e| format!("OAuth認証失敗: {}", e.0))?;
        crate::trace::trace("IMAP", "connect: XOAUTH2 OK");
        Ok(session)
    } else {
        crate::trace::trace("IMAP", "connect: password login...");
        let session = client.login(&config.email, &config.password)
            .map_err(|e| format!("ログイン失敗: {}", e.0))?;
        crate::trace::trace("IMAP", "connect: login OK");
        Ok(session)
    }
}

/// Take a session from pool or create new. On error, auto-reconnect once.
fn with_session<F, R>(config: &AccountConfig, f: F) -> Result<R, String>
where
    F: FnOnce(&mut ImapSession) -> Result<R, String>,
{
    let key = config.email.clone();
    let existing = POOL.lock().unwrap().remove(&key);

    let mut session = match existing {
        Some(mut s) => {
            if s.noop().is_ok() {
                crate::trace::trace("IMAP", "pool: reusing session");
                s
            } else {
                crate::trace::trace("IMAP", "pool: stale, reconnecting");
                connect_sync(config)?
            }
        }
        None => {
            crate::trace::trace("IMAP", "pool: new connection");
            let mut s = connect_sync(config)?;
            if FOLDER_MAP.lock().unwrap().is_empty() {
                let _ = discover_folders(&mut s);
            }
            s
        }
    };

    let result = f(&mut session);

    if result.is_ok() {
        POOL.lock().unwrap().insert(key, session);
    }

    result
}

struct XOAuth2(String);

impl imap::Authenticator for XOAuth2 {
    type Response = String;
    fn process(&self, _challenge: &[u8]) -> Self::Response {
        self.0.clone()
    }
}

fn extract_addr_str(addr: &imap_proto::types::Address) -> String {
    let mb = addr.mailbox.as_ref().map(|m| String::from_utf8_lossy(m).to_string()).unwrap_or_default();
    let host = addr.host.as_ref().map(|h| String::from_utf8_lossy(h).to_string()).unwrap_or_default();
    let name = addr.name.as_ref().map(|n| decode_rfc2047(n));
    match name {
        Some(n) if !n.is_empty() => format!("{n} <{mb}@{host}>"),
        _ => format!("{mb}@{host}"),
    }
}

fn extract_name_or_email(addr: &imap_proto::types::Address) -> String {
    let mb = addr.mailbox.as_ref().map(|m| String::from_utf8_lossy(m).to_string()).unwrap_or_default();
    let host = addr.host.as_ref().map(|h| String::from_utf8_lossy(h).to_string()).unwrap_or_default();
    addr.name.as_ref()
        .map(|n| decode_rfc2047(n))
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| format!("{mb}@{host}"))
}

pub async fn test_connection(config: &AccountConfig) -> Result<String, String> {
    let config = config.clone();
    tokio::task::spawn_blocking(move || {
        let mut session = connect_sync(&config)?;
        let _ = session.logout();
        Ok("接続成功".into())
    }).await.map_err(|e| format!("{e}"))?
}

fn fetch_envelope_list(session: &mut ImapSession, uids: &[u32]) -> Result<Vec<MailSummary>, String> {
    if uids.is_empty() { return Ok(vec![]); }
    let range: String = uids.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(",");
    let messages = session.uid_fetch(&range, "(UID FLAGS ENVELOPE)")
        .map_err(|e| format!("メール取得失敗: {e}"))?;

    let mut results: Vec<MailSummary> = Vec::new();
    for msg in messages.iter() {
        let uid = msg.uid.unwrap_or(0);
        let seen = msg.flags().iter().any(|f| matches!(f, imap::types::Flag::Seen));
        let envelope = match msg.envelope() { Some(e) => e, None => continue };
        let from = envelope.from.as_ref().and_then(|a| a.first()).map(|a| extract_name_or_email(a)).unwrap_or_default();
        let subject = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
        let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
        results.push(MailSummary { uid, from, subject, date, seen });
    }
    Ok(results)
}

pub async fn fetch_list(config: &AccountConfig, folder: &str, count: u32) -> Result<Vec<MailSummary>, String> {
    let config = config.clone();
    let folder = folder.to_string();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            if folder == "STARRED" {
                // STARRED: search FLAGGED across all mail
                session.select(&resolve_folder("ALL")).map_err(|e| format!("フォルダ選択失敗: {e}"))?;
                let uids: Vec<u32> = session.uid_search("FLAGGED")
                    .map_err(|e| format!("検索失敗: {e}"))?
                    .into_iter().collect();
                crate::trace::trace("IMAP", &format!("fetch_list STARRED: {} flagged", uids.len()));
                let take: Vec<u32> = uids.into_iter().rev().take(count as usize).collect();
                let mut results = fetch_envelope_list(session, &take)?;
                results.sort_by(|a, b| b.uid.cmp(&a.uid));
                crate::trace::trace("IMAP", &format!("fetch_list STARRED: done, {} results", results.len()));
                Ok(results)
            } else {
                let mailbox = session.select(&resolve_folder(&folder)).map_err(|e| format!("フォルダ選択失敗: {e}"))?;
                let total = mailbox.exists;
                crate::trace::trace("IMAP", &format!("fetch_list {}: {total} messages", folder));
                if total == 0 { return Ok(vec![]); }
                let start = total.saturating_sub(count) + 1;
                let range = format!("{start}:{total}");

                let messages = session.fetch(&range, "(UID FLAGS ENVELOPE)")
                    .map_err(|e| format!("メール取得失敗: {e}"))?;

                let mut results: Vec<MailSummary> = Vec::new();
                for msg in messages.iter() {
                    let uid = msg.uid.unwrap_or(0);
                    let seen = msg.flags().iter().any(|f| matches!(f, imap::types::Flag::Seen));
                    let envelope = match msg.envelope() { Some(e) => e, None => continue };
                    let from = envelope.from.as_ref().and_then(|a| a.first()).map(|a| extract_name_or_email(a)).unwrap_or_default();
                    let subject = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
                    let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                    results.push(MailSummary { uid, from, subject, date, seen });
                }
                results.reverse();
                crate::trace::trace("IMAP", &format!("fetch_list: done, {} results", results.len()));
                Ok(results)
            }
        })
    }).await.map_err(|e| format!("{e}"))?
}

/// Fetch a page of mails. offset=0 means newest. Returns (mails, total_in_folder).
pub async fn fetch_mail_page(config: &AccountConfig, folder: &str, offset: u32, limit: u32) -> Result<(Vec<MailSummary>, u32), String> {
    let config = config.clone();
    let folder = folder.to_string();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            let imap_folder = resolve_folder(&folder);
            let mailbox = session.select(&imap_folder).map_err(|e| format!("フォルダ選択失敗: {e}"))?;
            let total = mailbox.exists;
            if total == 0 { return Ok((vec![], 0)); }

            // sequence numbers: newest = total, oldest = 1
            let end = total.saturating_sub(offset);
            if end == 0 { return Ok((vec![], total)); }
            let start = end.saturating_sub(limit) + 1;
            let range = format!("{start}:{end}");

            crate::trace::trace("IMAP", &format!("fetch_mail_page: seq {range} (offset={offset}, limit={limit}, total={total})"));
            let messages = session.fetch(&range, "(UID FLAGS ENVELOPE)")
                .map_err(|e| format!("メール取得失敗: {e}"))?;

            let mut results: Vec<MailSummary> = Vec::new();
            for msg in messages.iter() {
                let uid = msg.uid.unwrap_or(0);
                let seen = msg.flags().iter().any(|f| matches!(f, imap::types::Flag::Seen));
                let envelope = match msg.envelope() { Some(e) => e, None => continue };
                let from = envelope.from.as_ref().and_then(|a| a.first()).map(|a| extract_name_or_email(a)).unwrap_or_default();
                let subject = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
                let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                results.push(MailSummary { uid, from, subject, date, seen });
            }
            results.reverse();
            crate::trace::trace("IMAP", &format!("fetch_mail_page: got {} results", results.len()));
            Ok((results, total))
        })
    }).await.map_err(|e| format!("{e}"))?
}

pub async fn search_mails(config: &AccountConfig, folder: &str, query: &str, limit: u32) -> Result<Vec<MailSummary>, String> {
    let config = config.clone();
    let folder = folder.to_string();
    let query = query.to_string();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            let imap_folder = resolve_folder(&folder);
            session.select(&imap_folder).map_err(|e| format!("フォルダ選択失敗: {e}"))?;

            // Try Gmail X-GM-RAW first, fall back to standard IMAP SEARCH
            let uids: Vec<u32> = session.uid_search(format!("X-GM-RAW \"{}\"", query.replace('"', "\\\"")))
                .or_else(|_| {
                    // Fallback: search in subject and from
                    let escaped = query.replace('"', "\\\"");
                    session.uid_search(format!("OR SUBJECT \"{}\" FROM \"{}\"", escaped, escaped))
                })
                .map_err(|e| format!("検索失敗: {e}"))?
                .into_iter().collect();

            crate::trace::trace("IMAP", &format!("search_mails: {} hits for '{}'", uids.len(), query));
            let take: Vec<u32> = uids.into_iter().rev().take(limit as usize).collect();
            let mut results = fetch_envelope_list(session, &take)?;
            results.sort_by(|a, b| b.uid.cmp(&a.uid));
            Ok(results)
        })
    }).await.map_err(|e| format!("{e}"))?
}

pub async fn fetch_new_mails(config: &AccountConfig, folder: &str, since_uid: u32) -> Result<Vec<MailSummary>, String> {
    let config = config.clone();
    let folder = folder.to_string();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            let imap_folder = resolve_folder(&folder);
            session.select(&imap_folder).map_err(|e| format!("フォルダ選択失敗: {e}"))?;

            let range = format!("{}:*", since_uid + 1);
            let messages = session.uid_fetch(&range, "(UID FLAGS ENVELOPE)")
                .map_err(|e| format!("差分取得失敗: {e}"))?;

            let mut results: Vec<MailSummary> = Vec::new();
            for msg in messages.iter() {
                let uid = msg.uid.unwrap_or(0);
                if uid <= since_uid { continue; } // IMAP may return since_uid itself
                let seen = msg.flags().iter().any(|f| matches!(f, imap::types::Flag::Seen));
                let envelope = match msg.envelope() { Some(e) => e, None => continue };
                let from = envelope.from.as_ref().and_then(|a| a.first()).map(|a| extract_name_or_email(a)).unwrap_or_default();
                let subject = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
                let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                results.push(MailSummary { uid, from, subject, date, seen });
            }
            results.reverse();
            crate::trace::trace("IMAP", &format!("fetch_new_mails: {} new since uid={}", results.len(), since_uid));
            Ok(results)
        })
    }).await.map_err(|e| format!("{e}"))?
}

fn parse_mail_detail(uid: u32, msg: &imap::types::Fetch) -> Result<MailDetail, String> {
    let body_raw = msg.body().unwrap_or_default();
    let parsed = mailparse::parse_mail(body_raw).map_err(|e| format!("パース失敗: {e}"))?;

    let body_text = parsed.subparts.iter()
        .find(|p| p.ctype.mimetype == "text/plain")
        .or_else(|| if parsed.ctype.mimetype == "text/plain" { Some(&parsed) } else { None })
        .and_then(|p| p.get_body().ok())
        .unwrap_or_default();

    let body_html = parsed.subparts.iter()
        .find(|p| p.ctype.mimetype == "text/html")
        .or_else(|| if parsed.ctype.mimetype == "text/html" { Some(&parsed) } else { None })
        .and_then(|p| p.get_body().ok())
        .unwrap_or_default();

    let attachments: Vec<Attachment> = parsed.subparts.iter().enumerate()
        .filter(|(_, p)| {
            p.ctype.mimetype != "text/plain" && p.ctype.mimetype != "text/html"
                && p.get_content_disposition().disposition != mailparse::DispositionType::Inline
        })
        .map(|(i, p)| {
            let filename = p.get_content_disposition().params.get("filename")
                .cloned().unwrap_or_else(|| format!("attachment_{i}"));
            let size = p.get_body_raw().map(|b| b.len()).unwrap_or(0);
            Attachment { index: i, filename, mime_type: p.ctype.mimetype.clone(), size }
        })
        .collect();

    let envelope = msg.envelope().ok_or("エンベロープなし")?;
    let addrs_to_string = |addrs: &Option<Vec<imap_proto::types::Address>>| {
        addrs.as_ref().map(|a| a.iter().map(extract_addr_str).collect::<Vec<_>>().join(", ")).unwrap_or_default()
    };
    let from = addrs_to_string(&envelope.from);
    let to = addrs_to_string(&envelope.to);
    let subject = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
    let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();

    Ok(MailDetail { uid, from, to, subject, date, body_text, body_html, attachments })
}

pub async fn fetch_detail(config: &AccountConfig, folder: &str, uid: u32) -> Result<MailDetail, String> {
    // Check cache first
    if let Some(detail) = CACHE.lock().unwrap().get(&uid) {
        crate::trace::trace("IMAP", &format!("fetch_detail uid={}: cache HIT", uid));
        return Ok(detail.clone());
    }

    let config = config.clone();
    let folder = folder.to_string();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            session.select(resolve_folder(&folder)).map_err(|e| format!("フォルダ選択失敗: {e}"))?;
            let messages = session.uid_fetch(uid.to_string(), "(UID ENVELOPE BODY[])")
                .map_err(|e| format!("メール取得失敗: {e}"))?;
            let msg = messages.iter().next().ok_or("メールが見つかりません".to_string())?;
            let detail = parse_mail_detail(uid, &msg)?;
            CACHE.lock().unwrap().insert(uid, detail.clone());
            crate::trace::trace("IMAP", &format!("fetch_detail uid={}: fetched & cached", uid));
            Ok(detail)
        })
    }).await.map_err(|e| format!("{e}"))?
}

pub async fn preload_mails(config: &AccountConfig, folder: &str, uids: Vec<u32>) -> Result<u32, String> {
    // Filter out already cached UIDs
    let uncached: Vec<u32> = {
        let cache = CACHE.lock().unwrap();
        uids.into_iter().filter(|uid| !cache.contains_key(uid)).collect()
    };
    if uncached.is_empty() {
        crate::trace::trace("IMAP", "preload: all cached");
        return Ok(0);
    }

    let config = config.clone();
    let folder = folder.to_string();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            session.select(resolve_folder(&folder)).map_err(|e| format!("フォルダ選択失敗: {e}"))?;
            let uid_range: String = uncached.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(",");
            crate::trace::trace("IMAP", &format!("preload: fetching {} mails", uncached.len()));

            let messages = session.uid_fetch(&uid_range, "(UID ENVELOPE BODY[])")
                .map_err(|e| format!("プリロード失敗: {e}"))?;

            let mut count = 0u32;
            let mut cache = CACHE.lock().unwrap();
            for msg in messages.iter() {
                let uid = msg.uid.unwrap_or(0);
                if uid == 0 { continue; }
                if let Ok(detail) = parse_mail_detail(uid, &msg) {
                    // Pre-cache ICS attachment data
                    if detail.attachments.iter().any(|a| a.filename.ends_with(".ics")) {
                        if let Some(body_raw) = msg.body() {
                            if let Ok(parsed) = mailparse::parse_mail(body_raw) {
                                for att in &detail.attachments {
                                    if att.filename.ends_with(".ics") {
                                        if let Some(part) = parsed.subparts.get(att.index) {
                                            if let Ok(data) = part.get_body_raw() {
                                                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
                                                let key = format!("{}:{}", uid, att.index);
                                                ICS_CACHE.lock().unwrap().insert(key, b64);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    cache.insert(uid, detail);
                    count += 1;
                }
            }
            crate::trace::trace("IMAP", &format!("preload: cached {} mails", count));
            Ok(count)
        })
    }).await.map_err(|e| format!("{e}"))?
}

pub async fn archive_mail(config: &AccountConfig, folder: &str, uid: u32) -> Result<String, String> {
    let config = config.clone();
    let folder = folder.to_string();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            let dest = resolve_folder("ALL");
            session.select(resolve_folder(&folder)).map_err(|e| format!("{e}"))?;
            session.uid_mv(uid.to_string(), &dest).map_err(|e| format!("アーカイブ失敗: {e}"))?;
            Ok("アーカイブ完了".into())
        })
    }).await.map_err(|e| format!("{e}"))?
}

pub async fn delete_mail(config: &AccountConfig, folder: &str, uid: u32) -> Result<String, String> {
    let config = config.clone();
    let folder = folder.to_string();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            let dest = resolve_folder("TRASH");
            session.select(resolve_folder(&folder)).map_err(|e| format!("{e}"))?;
            session.uid_mv(uid.to_string(), &dest).map_err(|e| format!("削除失敗: {e}"))?;
            Ok("削除完了".into())
        })
    }).await.map_err(|e| format!("{e}"))?
}

pub async fn toggle_star(config: &AccountConfig, folder: &str, uid: u32, add: bool) -> Result<String, String> {
    let config = config.clone();
    let folder = folder.to_string();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            session.select(resolve_folder(&folder)).map_err(|e| format!("{e}"))?;
            if add {
                session.uid_store(uid.to_string(), "+FLAGS (\\Flagged)").map_err(|e| format!("{e}"))?;
            } else {
                session.uid_store(uid.to_string(), "-FLAGS (\\Flagged)").map_err(|e| format!("{e}"))?;
            }
            Ok(if add { "スター追加" } else { "スター解除" }.into())
        })
    }).await.map_err(|e| format!("{e}"))?
}

pub async fn fetch_folders(config: &AccountConfig) -> Result<Vec<FolderInfo>, String> {
    let config = config.clone();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            let folders = session.list(Some(""), Some("*"))
                .map_err(|e| format!("フォルダ一覧取得失敗: {e}"))?;
            let mut result = Vec::new();
            for f in folders.iter() {
                let name = f.name().to_string();
                let count = session.examine(&name).map(|m| m.exists).unwrap_or(0);
                result.push(FolderInfo { name, count });
            }
            Ok(result)
        })
    }).await.map_err(|e| format!("{e}"))?
}

pub async fn fetch_thread(config: &AccountConfig, folder: &str, subject: &str) -> Result<Vec<MailSummary>, String> {
    let config = config.clone();
    let folder = folder.to_string();
    let subject = subject.to_string();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            session.select(resolve_folder(&folder)).map_err(|e| format!("{e}"))?;
            let clean_subject = subject.trim_start_matches("Re: ").trim_start_matches("Fwd: ");
            let query = format!("SUBJECT \"{}\"", clean_subject.replace('"', "\\\""));
            let uids = session.uid_search(&query).map_err(|e| format!("検索失敗: {e}"))?;

            if uids.is_empty() { return Ok(vec![]); }
            let range: String = uids.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(",");
            let messages = session.uid_fetch(&range, "(UID FLAGS ENVELOPE)")
                .map_err(|e| format!("{e}"))?;

            let mut results = Vec::new();
            for msg in messages.iter() {
                let uid = msg.uid.unwrap_or(0);
                let seen = msg.flags().iter().any(|f| matches!(f, imap::types::Flag::Seen));
                if let Some(envelope) = msg.envelope() {
                    let from = envelope.from.as_ref().and_then(|a| a.first()).map(|a| extract_name_or_email(a)).unwrap_or_default();
                    let subject = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
                    let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                    results.push(MailSummary { uid, from, subject, date, seen });
                }
            }
            Ok(results)
        })
    }).await.map_err(|e| format!("{e}"))?
}

pub async fn download_attachment(config: &AccountConfig, folder: &str, uid: u32, part_index: usize, filename: &str) -> Result<String, String> {
    let data = fetch_attachment_bytes(config, folder, uid, part_index).await?;
    let downloads = dirs::download_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let path = downloads.join(filename);
    std::fs::write(&path, &data).map_err(|e| format!("保存失敗: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

pub async fn fetch_attachment_data(config: &AccountConfig, folder: &str, uid: u32, part_index: usize) -> Result<String, String> {
    // Check ICS cache first
    let cache_key = format!("{}:{}", uid, part_index);
    if let Some(b64) = ICS_CACHE.lock().unwrap().get(&cache_key) {
        crate::trace::trace("IMAP", &format!("fetch_attachment uid={}:{}: ICS cache HIT", uid, part_index));
        return Ok(b64.clone());
    }
    let data = fetch_attachment_bytes(config, folder, uid, part_index).await?;
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data))
}

async fn fetch_attachment_bytes(config: &AccountConfig, folder: &str, uid: u32, part_index: usize) -> Result<Vec<u8>, String> {
    let config = config.clone();
    let folder = folder.to_string();
    tokio::task::spawn_blocking(move || {
        with_session(&config, |session| {
            session.select(resolve_folder(&folder)).map_err(|e| format!("{e}"))?;
            let messages = session.uid_fetch(uid.to_string(), "(UID BODY[])")
                .map_err(|e| format!("{e}"))?;
            let msg = messages.iter().next().ok_or("メールが見つかりません".to_string())?;
            let body_raw = msg.body().unwrap_or_default();
            let parsed = mailparse::parse_mail(body_raw).map_err(|e| format!("{e}"))?;
            let part = parsed.subparts.get(part_index).ok_or("パートが見つかりません".to_string())?;
            part.get_body_raw().map_err(|e| format!("{e}"))
        })
    }).await.map_err(|e| format!("{e}"))?
}
