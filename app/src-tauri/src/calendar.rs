use serde::{Deserialize, Serialize};
use crate::LlmConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarEvent {
    pub title: String,
    pub start: String,
    pub end: String,
    pub location: String,
}

pub async fn detect_events(llm: &LlmConfig, mail_body: &str) -> Result<Vec<CalendarEvent>, String> {
    let system = "メールから日時を含む予定・イベント・予約・会議・来店・配送などを抽出してJSON配列で返してください。\n\n抽出ルール:\n- 「来店日時」「配送予定日」「会議日時」「開催日」など日時が明記されていれば必ず抽出する\n- 予約確認・リマインドメールは予約そのものがイベント\n- endが不明な場合はstartの1時間後にする\n- locationが不明な場合は空文字\n- titleはイベント内容が分かる具体的な名前にする（例:「CLAPS杉田店 カット」「Q2予算レビュー会議」）\n\n形式: [{\"title\":\"...\",\"start\":\"2026-04-13T14:00:00\",\"end\":\"2026-04-13T15:00:00\",\"location\":\"...\"}]\nイベントがなければ空配列[]を返してください。";
    let raw = crate::ai_client::raw_chat(llm, system, mail_body).await?;
    let start = raw.find('[').unwrap_or(0);
    let end = raw.rfind(']').map(|i| i + 1).unwrap_or(raw.len());
    serde_json::from_str(&raw[start..end]).map_err(|e| format!("JSON解析失敗: {e}"))
}

pub async fn register_apple_calendar(event: &CalendarEvent, calendar_name: &str) -> Result<String, String> {
    let script = format!(
        r#"tell application "Calendar"
    tell calendar "{cal}"
        set startDate to date "{start}"
        set endDate to date "{end}"
        set isDup to false
        repeat with e in (every event whose start date = startDate and end date = endDate)
            if summary of e is "{title}" then
                set isDup to true
                exit repeat
            end if
        end repeat
        if isDup then
            return "DUPLICATE"
        else
            make new event with properties {{summary:"{title}", start date:startDate, end date:endDate, location:"{loc}"}}
            return "OK"
        end if
    end tell
end tell"#,
        cal = calendar_name,
        title = event.title.replace('"', "\\\""),
        start = &event.start,
        end = &event.end,
        loc = event.location.replace('"', "\\\""),
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("osascript実行失敗: {e}"))?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if result.contains("DUPLICATE") {
            Err("同じ予定が既に登録されています".into())
        } else {
            Ok("カレンダーに登録しました".into())
        }
    } else {
        Err(format!("登録失敗: {}", String::from_utf8_lossy(&output.stderr)))
    }
}
