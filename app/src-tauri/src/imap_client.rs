use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{LazyLock, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex as TokioMutex;
use tokio::net::TcpStream;
use async_native_tls::TlsStream;
use async_imap::types::Flag;
use futures::TryStreamExt;
use crate::{AccountConfig, Attachment, FolderInfo, MailDetail, MailSummary};

type AsyncImapSession = async_imap::Session<TlsStream<TcpStream>>;

static POOL: LazyLock<TokioMutex<HashMap<String, AsyncImapSession>>> =
    LazyLock::new(|| TokioMutex::new(HashMap::new()));

const CACHE_HARD_LIMIT_BYTES: usize = 1_073_741_824;

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
            + detail.attachments.len() * 64 + 128
    }
    fn get(&mut self, uid: &u32) -> Option<&MailDetail> {
        if self.map.contains_key(uid) {
            self.order.retain(|u| u != uid);
            self.order.push_back(*uid);
            self.map.get(uid)
        } else { None }
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
        while (self.map.len() > max_count || self.estimated_bytes > CACHE_HARD_LIMIT_BYTES) && !self.order.is_empty() {
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
static ICS_CACHE: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static FOLDER_MAP: LazyLock<Mutex<HashMap<String, HashMap<String, String>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn set_cache_max(max: usize) {
    CACHE_MAX.store(max, Ordering::Relaxed);
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    cache.evict(max);
    log::debug!("cache max set to {}, bytes: {}KB", max, cache.estimated_bytes / 1024);
}

fn resolve_folder(email: &str, folder: &str) -> String {
    let map = FOLDER_MAP.lock().unwrap_or_else(|e| e.into_inner());
    map.get(email).and_then(|m| m.get(folder).cloned()).unwrap_or_else(|| folder.to_string())
}

async fn discover_folders(session: &mut AsyncImapSession, email: &str) -> Result<(), String> {
    let folders: Vec<_> = session.list(Some(""), Some("*")).await
        .map_err(|e| format!("フォルダ一覧取得失敗: {e}"))?
        .try_collect().await
        .map_err(|e| format!("フォルダ一覧取得失敗: {e}"))?;
    let mut global_map = FOLDER_MAP.lock().unwrap_or_else(|e| e.into_inner());
    let account_map = global_map.entry(email.to_string()).or_default();
    account_map.insert("INBOX".to_string(), "INBOX".to_string());
    for f in &folders {
        let name = f.name().to_string();
        use imap_proto::types::NameAttribute;
        for attr in f.attributes() {
            match attr {
                NameAttribute::All => { account_map.insert("ALL".to_string(), name.clone()); account_map.insert("STARRED".to_string(), name.clone()); }
                NameAttribute::Sent => { account_map.insert("SENT".to_string(), name.clone()); }
                NameAttribute::Drafts => { account_map.insert("DRAFTS".to_string(), name.clone()); }
                NameAttribute::Junk => { account_map.insert("SPAM".to_string(), name.clone()); }
                NameAttribute::Trash => { account_map.insert("TRASH".to_string(), name.clone()); }
                _ => {}
            }
        }
    }
    log::debug!("discover_folders[{}]: {:?}", email, account_map);
    Ok(())
}

fn decode_rfc2047(raw: &[u8]) -> String {
    let lossy = String::from_utf8_lossy(raw).to_string();
    if lossy.contains("=?") {
        match mailparse::parse_header(format!("Subject: {}", lossy).as_bytes()) {
            Ok((hdr, _)) => hdr.get_value(),
            Err(_) => lossy,
        }
    } else { lossy }
}

struct XOAuth2(String);
impl async_imap::Authenticator for XOAuth2 {
    type Response = String;
    fn process(&mut self, _challenge: &[u8]) -> Self::Response { self.0.clone() }
}

async fn connect_async(config: &AccountConfig) -> Result<AsyncImapSession, String> {
    log::debug!("connect: {}:{}", config.imap_host, config.imap_port);
    let tcp = TcpStream::connect((config.imap_host.as_str(), config.imap_port)).await
        .map_err(|e| format!("TCP接続失敗: {e}"))?;
    let tls = async_native_tls::TlsConnector::new();
    let tls_stream = tls.connect(&config.imap_host, tcp).await
        .map_err(|e| format!("TLS接続失敗: {e}"))?;
    let mut client = async_imap::Client::new(tls_stream);
    let _ = client.read_response().await.map_err(|e| format!("IMAP接続失敗: {e}"))?;
    let session = if config.auth_type == "oauth" {
        log::debug!("connect: XOAUTH2...");
        let auth = XOAuth2(format!("user={}\x01auth=Bearer {}\x01\x01", config.email, config.access_token));
        client.authenticate("XOAUTH2", auth).await.map_err(|e| format!("OAuth認証失敗: {}", e.0))?
    } else {
        log::debug!("connect: password login...");
        client.login(&config.email, &config.password).await.map_err(|e| format!("ログイン失敗: {}", e.0))?
    };
    log::debug!("connect: OK");
    Ok(session)
}

/// Get a session from pool or create new. Returns session that caller must return via `return_session`.
async fn get_session(config: &AccountConfig) -> Result<(String, AsyncImapSession), String> {
    let key = format!("{}:{}", config.email, config.imap_host);
    let existing = POOL.lock().await.remove(&key);
    let had_existing = existing.is_some();
    let mut session = match existing {
        Some(s) => s,
        None => {
            let mut s = connect_async(config).await?;
            if !FOLDER_MAP.lock().unwrap_or_else(|e| e.into_inner()).contains_key(&config.email) {
                let _ = discover_folders(&mut s, &config.email).await;
            }
            s
        }
    };
    if had_existing {
        if session.noop().await.is_err() {
            log::debug!("pool: stale, reconnecting");
            session = connect_async(config).await?;
            if !FOLDER_MAP.lock().unwrap_or_else(|e| e.into_inner()).contains_key(&config.email) {
                let _ = discover_folders(&mut session, &config.email).await;
            }
        }
    }
    Ok((key, session))
}

async fn return_session(key: String, session: AsyncImapSession) {
    POOL.lock().await.insert(key, session);
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
    addr.name.as_ref().map(|n| decode_rfc2047(n)).filter(|n| !n.is_empty())
        .unwrap_or_else(|| format!("{mb}@{host}"))
}

pub async fn test_connection(config: &AccountConfig) -> Result<String, String> {
    let mut session = connect_async(config).await?;
    let _ = session.logout().await;
    Ok("接続成功".into())
}

async fn fetch_envelope_list(session: &mut AsyncImapSession, uids: &[u32]) -> Result<Vec<MailSummary>, String> {
    if uids.is_empty() { return Ok(vec![]); }
    let range: String = uids.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(",");
    let messages: Vec<_> = session.uid_fetch(&range, "(UID FLAGS ENVELOPE)").await
        .map_err(|e| format!("メール取得失敗: {e}"))?
        .try_collect().await
        .map_err(|e| format!("メール取得失敗: {e}"))?;
    let mut results: Vec<MailSummary> = Vec::new();
    for msg in &messages {
        let uid = msg.uid.unwrap_or(0);
        let seen = msg.flags().any(|f| matches!(f, Flag::Seen));
        let envelope = match msg.envelope() { Some(e) => e, None => continue };
        let from = envelope.from.as_ref().and_then(|a| a.first()).map(|a| extract_name_or_email(a)).unwrap_or_default();
        let subject = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
        let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
        results.push(MailSummary { uid, from, subject, date, seen });
    }
    Ok(results)
}

pub async fn fetch_list(config: &AccountConfig, folder: &str, count: u32) -> Result<Vec<MailSummary>, String> {
    let (key, mut session) = get_session(config).await?;
    let result = async {
        if folder == "STARRED" {
            session.select(&resolve_folder(&config.email, "ALL")).await.map_err(|e| format!("フォルダ選択失敗: {e}"))?;
            let uids: Vec<u32> = session.uid_search("FLAGGED").await
                .map_err(|e| format!("検索失敗: {e}"))?.into_iter().collect();
            let take: Vec<u32> = uids.into_iter().rev().take(count as usize).collect();
            let mut results = fetch_envelope_list(&mut session, &take).await?;
            results.sort_by(|a, b| b.uid.cmp(&a.uid));
            Ok(results)
        } else {
            let mailbox = session.select(&resolve_folder(&config.email, folder)).await
                .map_err(|e| format!("フォルダ選択失敗: {e}"))?;
            let total = mailbox.exists;
            if total == 0 { return Ok(vec![]); }
            let start = total.saturating_sub(count) + 1;
            let range = format!("{start}:{total}");
            let messages: Vec<_> = session.fetch(&range, "(UID FLAGS ENVELOPE)").await
                .map_err(|e| format!("メール取得失敗: {e}"))?
                .try_collect().await
                .map_err(|e| format!("メール取得失敗: {e}"))?;
            let mut results: Vec<MailSummary> = Vec::new();
            for msg in &messages {
                let uid = msg.uid.unwrap_or(0);
                let seen = msg.flags().any(|f| matches!(f, Flag::Seen));
                let envelope = match msg.envelope() { Some(e) => e, None => continue };
                let from = envelope.from.as_ref().and_then(|a| a.first()).map(|a| extract_name_or_email(a)).unwrap_or_default();
                let subject = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
                let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                results.push(MailSummary { uid, from, subject, date, seen });
            }
            results.reverse();
            Ok(results)
        }
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

pub async fn fetch_mail_page(config: &AccountConfig, folder: &str, offset: u32, limit: u32) -> Result<(Vec<MailSummary>, u32), String> {
    let (key, mut session) = get_session(config).await?;
    let result = async {
        if folder == "STARRED" {
            session.select(&resolve_folder(&config.email, "ALL")).await.map_err(|e| format!("フォルダ選択失敗: {e}"))?;
            let uids: Vec<u32> = session.uid_search("FLAGGED").await
                .map_err(|e| format!("検索失敗: {e}"))?.into_iter().collect();
            let total = uids.len() as u32;
            if total == 0 { return Ok((vec![], 0)); }
            let page: Vec<u32> = uids.into_iter().rev().skip(offset as usize).take(limit as usize).collect();
            let mut results = fetch_envelope_list(&mut session, &page).await?;
            results.sort_by(|a, b| b.uid.cmp(&a.uid));
            Ok((results, total))
        } else {
            let imap_folder = resolve_folder(&config.email, folder);
            let mailbox = session.select(&imap_folder).await.map_err(|e| format!("フォルダ選択失敗: {e}"))?;
            let total = mailbox.exists;
            if total == 0 { return Ok((vec![], 0)); }
            let end = total.saturating_sub(offset);
            if end == 0 { return Ok((vec![], total)); }
            let start = end.saturating_sub(limit) + 1;
            let range = format!("{start}:{end}");
            let messages: Vec<_> = session.fetch(&range, "(UID FLAGS ENVELOPE)").await
                .map_err(|e| format!("メール取得失敗: {e}"))?
                .try_collect().await
                .map_err(|e| format!("メール取得失敗: {e}"))?;
            let mut results: Vec<MailSummary> = Vec::new();
            for msg in &messages {
                let uid = msg.uid.unwrap_or(0);
                let seen = msg.flags().any(|f| matches!(f, Flag::Seen));
                let envelope = match msg.envelope() { Some(e) => e, None => continue };
                let from = envelope.from.as_ref().and_then(|a| a.first()).map(|a| extract_name_or_email(a)).unwrap_or_default();
                let subject = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
                let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                results.push(MailSummary { uid, from, subject, date, seen });
            }
            results.reverse();
            Ok((results, total))
        }
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

pub async fn search_mails(config: &AccountConfig, folder: &str, query: &str, limit: u32) -> Result<Vec<MailSummary>, String> {
    let (key, mut session) = get_session(config).await?;
    let result = async {
        let imap_folder = resolve_folder(&config.email, folder);
        session.select(&imap_folder).await.map_err(|e| format!("フォルダ選択失敗: {e}"))?;
        let sanitized_query = query.replace(['"', '\r', '\n', '\0'], "");
        let gmail_query = format!("X-GM-RAW \"{}\"", sanitized_query);
        let uids: Vec<u32> = match session.uid_search(&gmail_query).await {
            Ok(set) => set.into_iter().collect(),
            Err(_) => {
                let fallback = format!("OR SUBJECT \"{}\" FROM \"{}\"", sanitized_query, sanitized_query);
                session.uid_search(&fallback).await.map_err(|e| format!("検索失敗: {e}"))?.into_iter().collect()
            }
        };
        let take: Vec<u32> = uids.into_iter().rev().take(limit as usize).collect();
        let mut results = fetch_envelope_list(&mut session, &take).await?;
        results.sort_by(|a, b| b.uid.cmp(&a.uid));
        Ok(results)
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

pub async fn fetch_new_mails(config: &AccountConfig, folder: &str, since_uid: u32) -> Result<Vec<MailSummary>, String> {
    let (key, mut session) = get_session(config).await?;
    let result = async {
        let imap_folder = resolve_folder(&config.email, folder);
        session.select(&imap_folder).await.map_err(|e| format!("フォルダ選択失敗: {e}"))?;
        let range = format!("{}:*", since_uid + 1);
        let messages: Vec<_> = session.uid_fetch(&range, "(UID FLAGS ENVELOPE)").await
            .map_err(|e| format!("差分取得失敗: {e}"))?
            .try_collect().await
            .map_err(|e| format!("差分取得失敗: {e}"))?;
        let mut results: Vec<MailSummary> = Vec::new();
        for msg in &messages {
            let uid = msg.uid.unwrap_or(0);
            if uid <= since_uid { continue; }
            let seen = msg.flags().any(|f| matches!(f, Flag::Seen));
            let envelope = match msg.envelope() { Some(e) => e, None => continue };
            let from = envelope.from.as_ref().and_then(|a| a.first()).map(|a| extract_name_or_email(a)).unwrap_or_default();
            let subject = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
            let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
            results.push(MailSummary { uid, from, subject, date, seen });
        }
        results.reverse();
        Ok(results)
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

fn parse_mail_detail(uid: u32, msg: &async_imap::types::Fetch) -> Result<MailDetail, String> {
    let body_raw = msg.body().unwrap_or_default();
    let parsed = mailparse::parse_mail(body_raw).map_err(|e| format!("パース失敗: {e}"))?;
    let body_text = parsed.subparts.iter()
        .find(|p| p.ctype.mimetype == "text/plain")
        .or_else(|| if parsed.ctype.mimetype == "text/plain" { Some(&parsed) } else { None })
        .and_then(|p| p.get_body().ok()).unwrap_or_default();
    let body_html = parsed.subparts.iter()
        .find(|p| p.ctype.mimetype == "text/html")
        .or_else(|| if parsed.ctype.mimetype == "text/html" { Some(&parsed) } else { None })
        .and_then(|p| p.get_body().ok()).unwrap_or_default();
    let attachments: Vec<Attachment> = parsed.subparts.iter().enumerate()
        .filter(|(_, p)| p.ctype.mimetype != "text/plain" && p.ctype.mimetype != "text/html"
            && p.get_content_disposition().disposition != mailparse::DispositionType::Inline)
        .map(|(i, p)| {
            let filename = p.get_content_disposition().params.get("filename")
                .cloned().unwrap_or_else(|| format!("attachment_{i}"));
            let size = p.get_body_raw().map(|b| b.len()).unwrap_or(0);
            Attachment { index: i, filename, mime_type: p.ctype.mimetype.clone(), size }
        }).collect();
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
    if let Some(detail) = CACHE.lock().unwrap_or_else(|e| e.into_inner()).get(&uid) {
        return Ok(detail.clone());
    }
    let (key, mut session) = get_session(config).await?;
    let result = async {
        session.select(resolve_folder(&config.email, folder)).await.map_err(|e| format!("フォルダ選択失敗: {e}"))?;
        let messages: Vec<_> = session.uid_fetch(uid.to_string(), "(UID ENVELOPE BODY[])").await
            .map_err(|e| format!("メール取得失敗: {e}"))?
            .try_collect().await
            .map_err(|e| format!("メール取得失敗: {e}"))?;
        let msg = messages.first().ok_or("メールが見つかりません".to_string())?;
        let detail = parse_mail_detail(uid, msg)?;
        CACHE.lock().unwrap_or_else(|e| e.into_inner()).insert(uid, detail.clone());
        Ok(detail)
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

pub async fn preload_mails(config: &AccountConfig, folder: &str, uids: Vec<u32>) -> Result<u32, String> {
    let uncached: Vec<u32> = {
        let cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
        uids.into_iter().filter(|uid| !cache.contains_key(uid)).collect()
    };
    if uncached.is_empty() { return Ok(0); }
    let (key, mut session) = get_session(config).await?;
    let result = async {
        session.select(resolve_folder(&config.email, folder)).await.map_err(|e| format!("フォルダ選択失敗: {e}"))?;
        let uid_range: String = uncached.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(",");
        let messages: Vec<_> = session.uid_fetch(&uid_range, "(UID ENVELOPE BODY[])").await
            .map_err(|e| format!("プリロード失敗: {e}"))?
            .try_collect().await
            .map_err(|e| format!("プリロード失敗: {e}"))?;
        let mut count = 0u32;
        let mut ics_entries: Vec<(String, String)> = Vec::new();
        let mut details_to_cache: Vec<(u32, MailDetail)> = Vec::new();
        for msg in &messages {
            let uid = msg.uid.unwrap_or(0);
            if uid == 0 { continue; }
            if let Ok(detail) = parse_mail_detail(uid, msg) {
                if detail.attachments.iter().any(|a| a.filename.ends_with(".ics")) {
                    if let Some(body_raw) = msg.body() {
                        if let Ok(parsed) = mailparse::parse_mail(body_raw) {
                            for att in &detail.attachments {
                                if att.filename.ends_with(".ics") {
                                    if let Some(part) = parsed.subparts.get(att.index) {
                                        if let Ok(data) = part.get_body_raw() {
                                            let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
                                            ics_entries.push((format!("{}:{}", uid, att.index), b64));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                details_to_cache.push((uid, detail));
                count += 1;
            }
        }
        { let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner()); for (uid, detail) in details_to_cache { cache.insert(uid, detail); } }
        if !ics_entries.is_empty() { let mut ics_cache = ICS_CACHE.lock().unwrap_or_else(|e| e.into_inner()); for (k, v) in ics_entries { ics_cache.insert(k, v); } }
        Ok(count)
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

pub async fn archive_mail(config: &AccountConfig, folder: &str, uid: u32) -> Result<String, String> {
    let (key, mut session) = get_session(config).await?;
    let result = async {
        let dest = resolve_folder(&config.email, "ALL");
        session.select(resolve_folder(&config.email, folder)).await.map_err(|e| format!("{e}"))?;
        session.uid_mv(uid.to_string(), &dest).await.map_err(|e| format!("アーカイブ失敗: {e}"))?;
        Ok("アーカイブ完了".into())
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

pub async fn delete_mail(config: &AccountConfig, folder: &str, uid: u32) -> Result<String, String> {
    let (key, mut session) = get_session(config).await?;
    let result = async {
        let dest = resolve_folder(&config.email, "TRASH");
        session.select(resolve_folder(&config.email, folder)).await.map_err(|e| format!("{e}"))?;
        session.uid_mv(uid.to_string(), &dest).await.map_err(|e| format!("削除失敗: {e}"))?;
        Ok("削除完了".into())
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

pub async fn toggle_star(config: &AccountConfig, folder: &str, uid: u32, add: bool) -> Result<String, String> {
    let (key, mut session) = get_session(config).await?;
    let result = async {
        session.select(resolve_folder(&config.email, folder)).await.map_err(|e| format!("{e}"))?;
        let uid_str = uid.to_string();
        if add {
            let _ = session.uid_store(&uid_str, "+FLAGS (\\Flagged)").await.map_err(|e| format!("{e}"))?
                .try_collect::<Vec<_>>().await;
        } else {
            let _ = session.uid_store(&uid_str, "-FLAGS (\\Flagged)").await.map_err(|e| format!("{e}"))?
                .try_collect::<Vec<_>>().await;
        }
        Ok(if add { "スター追加" } else { "スター解除" }.into())
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

pub async fn fetch_folders(config: &AccountConfig) -> Result<Vec<FolderInfo>, String> {
    let (key, mut session) = get_session(config).await?;
    let result = async {
        let folders: Vec<_> = session.list(Some(""), Some("*")).await
            .map_err(|e| format!("フォルダ一覧取得失敗: {e}"))?
            .try_collect().await
            .map_err(|e| format!("フォルダ一覧取得失敗: {e}"))?;
        let mut result = Vec::new();
        for f in &folders {
            let name = f.name().to_string();
            let count = session.examine(&name).await.map(|m| m.exists).unwrap_or(0);
            result.push(FolderInfo { name, count });
        }
        Ok(result)
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

pub async fn fetch_thread(config: &AccountConfig, folder: &str, subject: &str) -> Result<Vec<MailSummary>, String> {
    let (key, mut session) = get_session(config).await?;
    let subject = subject.to_string();
    let result = async {
        session.select(resolve_folder(&config.email, folder)).await.map_err(|e| format!("{e}"))?;
        let clean_subject = subject.trim_start_matches("Re: ").trim_start_matches("Fwd: ");
        let sanitized = clean_subject.replace(['"', '\r', '\n', '\0'], "");
        let query = format!("SUBJECT \"{}\"", sanitized);
        let uids: HashSet<u32> = session.uid_search(&query).await.map_err(|e| format!("検索失敗: {e}"))?;
        if uids.is_empty() { return Ok(vec![]); }
        let range: String = uids.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(",");
        let messages: Vec<_> = session.uid_fetch(&range, "(UID FLAGS ENVELOPE)").await
            .map_err(|e| format!("{e}"))?
            .try_collect().await
            .map_err(|e| format!("{e}"))?;
        let mut results = Vec::new();
        for msg in &messages {
            let uid = msg.uid.unwrap_or(0);
            let seen = msg.flags().any(|f| matches!(f, Flag::Seen));
            if let Some(envelope) = msg.envelope() {
                let from = envelope.from.as_ref().and_then(|a| a.first()).map(|a| extract_name_or_email(a)).unwrap_or_default();
                let subj = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
                let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                results.push(MailSummary { uid, from, subject: subj, date, seen });
            }
        }
        Ok(results)
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

pub async fn download_attachment(config: &AccountConfig, folder: &str, uid: u32, part_index: usize, filename: &str) -> Result<String, String> {
    let data = fetch_attachment_bytes(config, folder, uid, part_index).await?;
    let downloads = dirs::download_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let path = downloads.join(filename);
    std::fs::write(&path, &data).map_err(|e| format!("保存失敗: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

pub async fn fetch_attachment_data(config: &AccountConfig, folder: &str, uid: u32, part_index: usize) -> Result<String, String> {
    let cache_key = format!("{}:{}", uid, part_index);
    if let Some(b64) = ICS_CACHE.lock().unwrap_or_else(|e| e.into_inner()).get(&cache_key) {
        return Ok(b64.clone());
    }
    let data = fetch_attachment_bytes(config, folder, uid, part_index).await?;
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data))
}

pub async fn fetch_attachment_bytes(config: &AccountConfig, folder: &str, uid: u32, part_index: usize) -> Result<Vec<u8>, String> {
    let (key, mut session) = get_session(config).await?;
    let result = async {
        session.select(resolve_folder(&config.email, folder)).await.map_err(|e| format!("{e}"))?;
        let messages: Vec<_> = session.uid_fetch(uid.to_string(), "(UID BODY[])").await
            .map_err(|e| format!("{e}"))?
            .try_collect().await
            .map_err(|e| format!("{e}"))?;
        let msg = messages.first().ok_or("メールが見つかりません".to_string())?;
        let body_raw = msg.body().unwrap_or_default();
        let parsed = mailparse::parse_mail(body_raw).map_err(|e| format!("{e}"))?;
        let part = parsed.subparts.get(part_index).ok_or("パートが見つかりません".to_string())?;
        part.get_body_raw().map_err(|e| format!("{e}"))
    }.await;
    if result.is_ok() { return_session(key, session).await; }
    result
}

// --- Public API for IdleWatcher (separate from pooled sessions) ---

/// Create a new IMAP session for IDLE use (not pooled).
pub async fn connect_idle_session(config: &AccountConfig) -> Result<AsyncImapSession, String> {
    connect_async(config).await
}

/// Fetch new mails using an existing session (for IDLE watcher).
pub async fn fetch_new_mails_with_session(
    session: &mut AsyncImapSession,
    _folder: &str,
    since_uid: u32,
) -> Result<Vec<MailSummary>, String> {
    let range = format!("{}:*", since_uid + 1);
    let messages: Vec<_> = session.uid_fetch(&range, "(UID FLAGS ENVELOPE)").await
        .map_err(|e| format!("差分取得失敗: {e}"))?
        .try_collect().await
        .map_err(|e| format!("差分取得失敗: {e}"))?;
    let mut results = Vec::new();
    for msg in &messages {
        let uid = msg.uid.unwrap_or(0);
        if uid <= since_uid { continue; }
        let seen = msg.flags().any(|f| matches!(f, async_imap::types::Flag::Seen));
        let envelope = match msg.envelope() { Some(e) => e, None => continue };
        let from = envelope.from.as_ref().and_then(|a| a.first()).map(|a| extract_name_or_email(a)).unwrap_or_default();
        let subject = envelope.subject.as_ref().map(|s| decode_rfc2047(s)).unwrap_or_default();
        let date = envelope.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
        results.push(MailSummary { uid, from, subject, date, seen });
    }
    results.reverse();
    Ok(results)
}

/// Public wrapper for resolve_folder (used by idle_watcher).
pub fn resolve_folder_pub(email: &str, folder: &str) -> String {
    resolve_folder(email, folder)
}
