use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tauri::Manager;

static RE_REGION: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"^[a-z0-9-]+$").unwrap());

mod ai_client;
mod ai_usage;
mod calendar;
mod error;
mod ics_parser;
mod idle_watcher;
mod imap_client;
mod keychain;
mod oauth;
mod smtp_client;
mod tray;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountConfig {
    pub email: String,
    pub auth_type: String,
    pub password: String,
    pub access_token: String,
    pub imap_host: String,
    pub imap_port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmtpConfig {
    pub email: String,
    pub auth_type: String,
    pub password: String,
    pub access_token: String,
    pub smtp_host: String,
    pub smtp_port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MailSummary {
    pub uid: u32,
    pub from: String,
    pub subject: String,
    pub date: String,
    pub seen: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct MailDetail {
    pub uid: u32,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub date: String,
    pub body_text: String,
    pub body_html: String,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub index: usize,
    pub filename: String,
    pub mime_type: String,
    pub size: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Nuance {
    pub icon: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct FolderInfo {
    pub name: String,
    pub count: u32,
}

// IMAP
#[tauri::command]
fn frontend_trace(tag: String, msg: String) {
    log::info!("FE:{}: {}", tag, msg);
}

#[tauri::command]
fn set_log_level(level: String) -> Result<(), String> {
    let filter = match level.as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => return Err(format!("不正なログレベル: {}", level)),
    };
    log::set_max_level(filter);
    log::info!("ログレベルを {} に変更", level);
    Ok(())
}

#[tauri::command]
async fn test_imap_connection(config: AccountConfig) -> Result<String, String> {
    log::debug!("test_imap_connection: {}", config.email);
    imap_client::test_connection(&config).await
}

#[tauri::command]
async fn fetch_mail_list(config: AccountConfig, folder: String, count: u32) -> Result<Vec<MailSummary>, String> {
    log::debug!("fetch_mail_list: folder={} count={}", folder, count);
    match imap_client::fetch_list(&config, &folder, count).await {
        Ok(mails) => { log::debug!("fetch_mail_list: OK {} mails", mails.len()); Ok(mails) }
        Err(e) => { log::debug!("fetch_mail_list: ERROR {}", e); Err(e) }
    }
}

#[tauri::command]
async fn fetch_mail_detail(config: AccountConfig, folder: String, uid: u32) -> Result<MailDetail, String> {
    log::debug!("fetch_mail_detail: uid={}", uid);
    match imap_client::fetch_detail(&config, &folder, uid).await {
        Ok(detail) => { log::debug!("fetch_mail_detail: OK uid={}", uid); Ok(detail) }
        Err(e) => { log::debug!("fetch_mail_detail: ERROR {}", e); Err(e) }
    }
}

#[tauri::command]
async fn search_mails(config: AccountConfig, folder: String, query: String, limit: u32) -> Result<Vec<MailSummary>, String> {
    log::debug!("search_mails: folder={} query={} limit={}", folder, query, limit);
    imap_client::search_mails(&config, &folder, &query, limit).await
}

#[tauri::command]
async fn fetch_mail_page(config: AccountConfig, folder: String, offset: u32, limit: u32) -> Result<(Vec<MailSummary>, u32), String> {
    log::debug!("fetch_mail_page: folder={} offset={} limit={}", folder, offset, limit);
    imap_client::fetch_mail_page(&config, &folder, offset, limit).await
}

#[tauri::command]
async fn fetch_new_mails(config: AccountConfig, folder: String, since_uid: u32) -> Result<Vec<MailSummary>, String> {
    log::debug!("fetch_new_mails: folder={} since_uid={}", folder, since_uid);
    imap_client::fetch_new_mails(&config, &folder, since_uid).await
}

#[tauri::command]
async fn fetch_folders(config: AccountConfig) -> Result<Vec<FolderInfo>, String> {
    log::debug!("fetch_folders");
    imap_client::fetch_folders(&config).await
}

#[tauri::command]
async fn fetch_thread(config: AccountConfig, folder: String, subject: String) -> Result<Vec<MailSummary>, String> {
    log::debug!("fetch_thread: subject={}", subject);
    imap_client::fetch_thread(&config, &folder, &subject).await
}

#[tauri::command]
async fn download_attachment(config: AccountConfig, folder: String, uid: u32, part_index: usize, filename: String) -> Result<String, String> {
    log::debug!("download_attachment: uid={} part={}", uid, part_index);
    imap_client::download_attachment(&config, &folder, uid, part_index, &filename).await
}

#[tauri::command]
async fn fetch_attachment_data(config: AccountConfig, folder: String, uid: u32, part_index: usize) -> Result<String, String> {
    imap_client::fetch_attachment_data(&config, &folder, uid, part_index).await
}

#[tauri::command]
async fn archive_mail(config: AccountConfig, folder: String, uid: u32) -> Result<String, String> {
    log::debug!("archive_mail: uid={}", uid);
    imap_client::archive_mail(&config, &folder, uid).await
}

#[tauri::command]
async fn delete_mail(config: AccountConfig, folder: String, uid: u32) -> Result<String, String> {
    log::debug!("delete_mail: uid={}", uid);
    imap_client::delete_mail(&config, &folder, uid).await
}

#[tauri::command]
async fn toggle_star(config: AccountConfig, folder: String, uid: u32, add: bool) -> Result<String, String> {
    log::debug!("toggle_star: uid={} add={}", uid, add);
    imap_client::toggle_star(&config, &folder, uid, add).await
}

#[tauri::command]
fn set_mail_cache_max(max: usize) {
    imap_client::set_cache_max(max);
}

#[tauri::command]
async fn preload_mails(config: AccountConfig, folder: String, uids: Vec<u32>) -> Result<u32, String> {
    log::debug!("preload_mails: {} uids", uids.len());
    imap_client::preload_mails(&config, &folder, uids).await
}

// SMTP
#[tauri::command]
async fn send_mail(config: SmtpConfig, to: Vec<String>, cc: Vec<String>, bcc: Vec<String>, subject: String, body: String) -> Result<String, String> {
    log::debug!("send_mail: to={:?}", to);
    smtp_client::send_mail(&config, &to, &cc, &bcc, &subject, &body).await
}

#[tauri::command]
async fn send_mail_with_attachments(config: SmtpConfig, to: Vec<String>, cc: Vec<String>, bcc: Vec<String>, subject: String, body: String, attachment_paths: Vec<String>) -> Result<String, String> {
    log::debug!("send_mail_with_attachments: {} files", attachment_paths.len());
    smtp_client::send_mail_with_attachments(&config, &to, &cc, &bcc, &subject, &body, &attachment_paths).await
}

#[tauri::command]
async fn list_bedrock_models(region: String, api_key: String) -> Result<Vec<String>, String> {
    log::debug!("list_bedrock_models");
    if !RE_REGION.is_match(&region) {
        return Err("不正なリージョン名です".into());
    }
    let url = format!("https://bedrock.{region}.amazonaws.com/foundation-models");
    let client = ai_client::http_client();
    let resp = client.get(&url)
        .query(&[("byInferenceType", "ON_DEMAND")])
        .header("Authorization", format!("Bearer {api_key}"))
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("モデル一覧取得失敗: {body}"));
    }
    #[derive(serde::Deserialize)]
    struct Resp { #[serde(rename = "modelSummaries")] model_summaries: Vec<Model> }
    #[derive(serde::Deserialize)]
    struct Model { #[serde(rename = "modelId")] model_id: String }
    let data: Resp = resp.json().await.map_err(|e| format!("{e}"))?;
    let ids: Vec<String> = data.model_summaries.into_iter().map(|m| m.model_id).collect();
    log::debug!("list_bedrock_models: {} models", ids.len());
    Ok(ids)
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("HTTPSのURLのみ開けます".into());
    }
    open::that(&url).map_err(|e| format!("URL open failed: {e}"))
}

#[tauri::command]
fn get_ai_usage() -> ai_usage::UsageSummary {
    ai_usage::get_summary()
}

#[tauri::command]
fn get_ai_usage_months() -> Vec<String> {
    ai_usage::get_available_months()
}

#[tauri::command]
fn get_ai_usage_for_month(month: String) -> Result<ai_usage::UsageSummary, String> {
    ai_usage::get_summary_for_month(&month)
}

#[tauri::command]
fn set_ai_budget(limit_usd: f64) {
    ai_usage::set_budget_limit(limit_usd);
}

#[tauri::command]
fn get_ai_daily_costs(days: u32) -> Vec<ai_usage::DailyCostEntry> {
    ai_usage::get_daily_costs(days)
}

#[tauri::command]
fn get_ai_feature_costs(month: String) -> Vec<ai_usage::FeatureCostEntry> {
    ai_usage::get_feature_costs(&month)
}

#[tauri::command]
fn get_ai_history() -> Vec<ai_usage::UsageLogEntry> {
    ai_usage::get_history()
}

// AI
#[tauri::command]
async fn ai_summarize(llm: LlmConfig, mail_body: String) -> Result<String, String> {
    log::debug!("ai_summarize");
    ai_client::summarize(&llm, &mail_body).await
}

#[tauri::command]
async fn ai_draft_nuances(llm: LlmConfig, mail_body: String) -> Result<Vec<Nuance>, String> {
    log::debug!("ai_draft_nuances");
    ai_client::draft_nuances(&llm, &mail_body).await
}

#[tauri::command]
async fn ai_draft_reply(llm: LlmConfig, mail_body: String, nuance: String, instruction: String) -> Result<String, String> {
    log::debug!("ai_draft_reply: nuance={}", nuance);
    ai_client::draft_reply(&llm, &mail_body, &nuance, &instruction).await
}

#[tauri::command]
async fn ai_translate(llm: LlmConfig, text: String, target_lang: String) -> Result<String, String> {
    log::debug!("ai_translate: lang={}", target_lang);
    ai_client::translate(&llm, &text, &target_lang).await
}

// OAuth
#[tauri::command]
async fn google_oauth_login() -> Result<oauth::OAuthTokens, String> {
    log::debug!("google_oauth_login");
    oauth::start_flow().await
}

#[tauri::command]
async fn google_oauth_refresh(refresh_token: String) -> Result<oauth::OAuthTokens, String> {
    log::debug!("google_oauth_refresh");
    oauth::refresh(&refresh_token).await
}

#[tauri::command]
async fn list_google_calendars(access_token: String) -> Result<Vec<String>, String> {
    log::debug!("list_google_calendars");
    let client = ai_client::http_client();
    let resp = client.get("https://www.googleapis.com/calendar/v3/users/me/calendarList")
        .bearer_auth(&access_token)
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("カレンダー一覧取得失敗: {body}"));
    }
    #[derive(serde::Deserialize)]
    struct Resp { items: Vec<Cal> }
    #[derive(serde::Deserialize)]
    struct Cal { summary: String }
    let data: Resp = resp.json().await.map_err(|e| format!("{e}"))?;
    Ok(data.items.into_iter().map(|c| c.summary).collect())
}

// Calendar
#[tauri::command]
async fn detect_calendar_events(llm: LlmConfig, mail_body: String) -> Result<Vec<calendar::CalendarEvent>, String> {
    log::debug!("detect_calendar_events");
    calendar::detect_events(&llm, &mail_body).await
}

#[tauri::command]
async fn register_calendar_event(event: calendar::CalendarEvent, calendar_name: String, provider: Option<String>, access_token: Option<String>) -> Result<String, String> {
    log::debug!("register_calendar_event: {} (provider={:?})", event.title, provider);
    match provider.as_deref() {
        Some("google") => {
            let token = access_token.ok_or("access_token が必要です")?;
            calendar::register_google_calendar(&event, &token).await
        }
        _ => calendar::register_apple_calendar(&event, &calendar_name).await,
    }
}

// ICS parsing
#[tauri::command]
fn parse_ics_attachment(data: String) -> Result<Vec<ics_parser::CalendarEvent>, String> {
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data)
        .map_err(|e| format!("base64 decode error: {e}"))?;
    ics_parser::parse_ics(&bytes)
}

#[tauri::command]
async fn respond_calendar_invite(smtp: SmtpConfig, event: ics_parser::CalendarEvent, accept: bool) -> Result<String, String> {
    let action = if accept { "ACCEPTED" } else { "DECLINED" };
    log::debug!("respond_calendar_invite: {} {}", event.summary, action);
    let ics = ics_parser::generate_reply_ics(&event, &smtp.email, accept);
    let subject = format!("{}: {}", if accept { "承諾" } else { "辞退" }, event.summary);
    smtp_client::send_calendar_response(&smtp, &event.organizer, &subject, &ics).await
}

#[tauri::command]
async fn check_calendar_conflicts(access_token: String, time_min: String, time_max: String, exclude_uid: String) -> Result<Vec<String>, String> {
    log::debug!("check_calendar_conflicts: {} ~ {}", time_min, time_max);
    let client = ai_client::http_client();
    let resp = client.get("https://www.googleapis.com/calendar/v3/calendars/primary/events")
        .query(&[("timeMin", &time_min), ("timeMax", &time_max), ("singleEvents", &"true".to_string()), ("orderBy", &"startTime".to_string())])
        .bearer_auth(&access_token)
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() { return Ok(vec![]); }

    #[derive(serde::Deserialize)]
    struct EventList { items: Option<Vec<GEvent>> }
    #[derive(serde::Deserialize)]
    struct GEvent { summary: Option<String>, #[serde(rename = "iCalUID")] ical_uid: Option<String>, start: Option<EventTime>, status: Option<String>, attendees: Option<Vec<ConflictAtt>> }
    #[derive(serde::Deserialize)]
    struct EventTime { #[serde(rename = "dateTime")] date_time: Option<String> }
    #[derive(serde::Deserialize)]
    struct ConflictAtt { #[serde(rename = "self", default)] is_self: bool, #[serde(rename = "responseStatus")] response_status: Option<String> }

    let list: EventList = resp.json().await.map_err(|e| format!("{e}"))?;
    let conflicts: Vec<String> = list.items.unwrap_or_default().into_iter()
        .filter(|e| e.ical_uid.as_deref() != Some(&exclude_uid))
        .filter(|e| e.start.as_ref().and_then(|s| s.date_time.as_ref()).is_some()) // exclude all-day
        .filter(|e| e.status.as_deref() != Some("cancelled"))
        .filter(|e| {
            e.attendees.as_ref().and_then(|atts| atts.iter().find(|a| a.is_self))
                .map(|a| a.response_status.as_deref() != Some("declined"))
                .unwrap_or(true)
        })
        .filter_map(|e| e.summary)
        .collect();
    Ok(conflicts)
}

#[tauri::command]
async fn get_calendar_event_status(access_token: String, ics_uid: String, my_email: String) -> Result<String, String> {
    let client = ai_client::http_client();
    let resp = client.get("https://www.googleapis.com/calendar/v3/calendars/primary/events")
        .query(&[("iCalUID", &ics_uid)])
        .bearer_auth(&access_token)
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() { return Ok("unknown".into()); }

    #[derive(serde::Deserialize)]
    struct EventList { items: Vec<GEvent> }
    #[derive(serde::Deserialize)]
    struct GEvent { attendees: Option<Vec<Att>> }
    #[derive(serde::Deserialize)]
    struct Att { email: String, #[serde(rename = "responseStatus")] response_status: String, #[serde(rename = "self", default)] is_self: bool }

    let list: EventList = resp.json().await.map_err(|e| format!("{e}"))?;
    let ev = match list.items.first() { Some(e) => e, None => return Ok("unknown".into()) };
    let my_lower = my_email.to_lowercase();
    let status = ev.attendees.as_ref()
        .and_then(|atts| atts.iter().find(|a| a.email.to_lowercase() == my_lower || a.is_self))
        .map(|a| a.response_status.clone())
        .unwrap_or_else(|| "unknown".into());
    Ok(status)
}

#[tauri::command]
async fn respond_google_calendar_invite(access_token: String, ics_uid: String, my_email: String, accept: bool) -> Result<String, String> {
    let status = if accept { "accepted" } else { "declined" };
    log::debug!("respond_google_calendar_invite: {} {}", ics_uid, status);
    let client = ai_client::http_client();

    // 1. iCalUID でイベントを検索
    let resp = client.get("https://www.googleapis.com/calendar/v3/calendars/primary/events")
        .query(&[("iCalUID", &ics_uid)])
        .bearer_auth(&access_token)
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("イベント検索失敗: {body}"));
    }

    #[derive(serde::Deserialize)]
    struct EventList { items: Vec<GEvent> }
    #[derive(serde::Deserialize, Clone)]
    struct GEvent { id: String, attendees: Option<Vec<Attendee>> }
    #[derive(serde::Deserialize, serde::Serialize, Clone)]
    struct Attendee { email: String, #[serde(rename = "responseStatus")] response_status: String, #[serde(rename = "self", default)] is_self: bool }

    let list: EventList = resp.json().await.map_err(|e| format!("{e}"))?;
    let ev = list.items.first().ok_or("カレンダーにイベントが見つかりません")?;

    // 2. attendees の自分の responseStatus を更新
    let mut attendees = ev.attendees.clone().unwrap_or_default();
    let my_lower = my_email.to_lowercase();
    let found = attendees.iter_mut().find(|a| a.email.to_lowercase() == my_lower || a.is_self);
    match found {
        Some(a) => a.response_status = status.to_string(),
        None => attendees.push(Attendee { email: my_email, response_status: status.to_string(), is_self: true }),
    }

    // 3. PATCH で更新
    let patch_url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/primary/events/{}",
        ev.id
    );
    let patch_resp = client.patch(&patch_url).bearer_auth(&access_token)
        .json(&serde_json::json!({ "attendees": attendees }))
        .send().await.map_err(|e| format!("{e}"))?;
    if !patch_resp.status().is_success() {
        let body = patch_resp.text().await.unwrap_or_default();
        return Err(format!("応答更新失敗: {body}"));
    }

    Ok(if accept { "承諾しました".into() } else { "辞退しました".into() })
}

// Keychain
#[tauri::command]
fn store_keychain(account: String, key_type: String, secret: String) -> Result<(), String> {
    keychain::store_credential(&account, &key_type, &secret)
}

#[tauri::command]
fn get_keychain(account: String, key_type: String) -> Result<String, String> {
    keychain::get_credential(&account, &key_type)
}

#[tauri::command]
fn delete_keychain(account: String, key_type: String) -> Result<(), String> {
    keychain::delete_credential(&account, &key_type)
}

// --- IdleWatcher commands ---

#[tauri::command]
async fn get_idle_status() -> Vec<idle_watcher::WatcherStatus> {
    idle_watcher::get_status().await
}

#[tauri::command]
async fn restart_idle_watcher(app: AppHandle, configs: Vec<idle_watcher::WatcherAccountConfig>) {
    log::debug!("restart_idle_watcher: {} accounts", configs.len());
    idle_watcher::reload(app, configs).await;
}

#[tauri::command]
async fn set_notification_pause(paused: bool) {
    log::debug!("set_notification_pause: {}", paused);
    idle_watcher::set_paused(paused).await;
}

use tauri::AppHandle;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log::info!("=== SmartAM starting ===");

    let builder = tauri::Builder::default();
    log::debug!("Builder created");

    builder
        .setup(|app| {
            log::debug!("setup: begin");
            app.handle().plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Trace).build())?;
            log::set_max_level(log::LevelFilter::Info);
            log::debug!("setup: log plugin ok");
            app.handle().plugin(tauri_plugin_store::Builder::default().build())?;
            log::debug!("setup: store plugin ok");
            app.handle().plugin(tauri_plugin_notification::init())?;
            log::debug!("setup: notification plugin ok");
            app.handle().plugin(tauri_plugin_dialog::init())?;
            log::debug!("setup: dialog plugin ok");
            app.handle().plugin(tauri_plugin_process::init())?;
            log::debug!("setup: process plugin ok");
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
            log::debug!("setup: updater plugin ok");
            log::debug!("setup: complete");
            // Tray icon setup
            tray::setup(app)?;
            log::debug!("setup: tray ok");
            // Fetch Bedrock pricing in background
            tauri::async_runtime::spawn(ai_usage::fetch_pricing());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            frontend_trace, set_log_level,
            test_imap_connection, fetch_mail_list, fetch_mail_detail, fetch_new_mails, fetch_mail_page, search_mails,
            fetch_folders, fetch_thread, download_attachment, fetch_attachment_data,
            archive_mail, delete_mail, toggle_star,
            preload_mails, set_mail_cache_max,
            send_mail, send_mail_with_attachments,
            list_bedrock_models,
            get_ai_usage, get_ai_usage_months, get_ai_usage_for_month, set_ai_budget,
            get_ai_daily_costs, get_ai_feature_costs, get_ai_history,
            ai_summarize, ai_draft_nuances, ai_draft_reply, ai_translate,
            google_oauth_login, google_oauth_refresh, list_google_calendars,
            detect_calendar_events, register_calendar_event,
            open_external_url,
            parse_ics_attachment,
            respond_calendar_invite,
            respond_google_calendar_invite,
            get_calendar_event_status,
            check_calendar_conflicts,
            store_keychain, get_keychain, delete_keychain,
            get_idle_status, restart_idle_watcher, set_notification_pause,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            match event {
                tauri::RunEvent::ExitRequested { api, .. } => {
                    // Prevent exit when all windows are closed — keep running in background
                    api.prevent_exit();
                }
                tauri::RunEvent::WindowEvent { event: tauri::WindowEvent::CloseRequested { api, .. }, .. } => {
                    // Hide window instead of closing
                    api.prevent_close();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                _ => {}
            }
        });

    log::info!("=== SmartAM exited ===");
}
