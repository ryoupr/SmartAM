use ical::IcalParser;
use serde::{Deserialize, Serialize};
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarEvent {
    pub uid: String,
    pub summary: String,
    pub dtstart: String,
    pub dtend: String,
    pub location: String,
    pub organizer: String,
    pub attendees: Vec<String>,
    pub status: String,
    pub description: String,
}

pub fn parse_ics(data: &[u8]) -> Result<Vec<CalendarEvent>, String> {
    let reader = BufReader::new(data);
    let parser = IcalParser::new(reader);
    let mut events = Vec::new();

    for cal in parser {
        let cal = cal.map_err(|e| format!("ics parse error: {e}"))?;
        for ev in cal.events {
            let prop = |name: &str| -> String {
                ev.properties.iter()
                    .find(|p| p.name == name)
                    .and_then(|p| p.value.clone())
                    .unwrap_or_default()
            };

            let organizer_raw = prop("ORGANIZER");
            let organizer = organizer_raw
                .strip_prefix("mailto:")
                .or_else(|| organizer_raw.strip_prefix("MAILTO:"))
                .unwrap_or(&organizer_raw)
                .to_string();

            let attendees: Vec<String> = ev.properties.iter()
                .filter(|p| p.name == "ATTENDEE")
                .filter_map(|p| p.value.as_ref())
                .map(|v| v.strip_prefix("mailto:").or_else(|| v.strip_prefix("MAILTO:")).unwrap_or(v).to_string())
                .collect();

            let status = ev.properties.iter()
                .find(|p| p.name == "ATTENDEE")
                .and_then(|p| p.params.as_ref())
                .and_then(|params| params.iter().find(|(k, _)| k == "PARTSTAT"))
                .and_then(|(_, vals)| vals.first().cloned())
                .unwrap_or_default();

            events.push(CalendarEvent {
                uid: prop("UID"),
                summary: prop("SUMMARY"),
                dtstart: prop("DTSTART"),
                dtend: prop("DTEND"),
                location: prop("LOCATION"),
                organizer,
                attendees,
                status,
                description: prop("DESCRIPTION"),
            });
        }
    }
    Ok(events)
}

pub fn generate_reply_ics(event: &CalendarEvent, my_email: &str, accept: bool) -> String {
    let partstat = if accept { "ACCEPTED" } else { "DECLINED" };
    format!(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID:-//SmartAM//EN\r\n\
         METHOD:REPLY\r\n\
         BEGIN:VEVENT\r\n\
         UID:{uid}\r\n\
         DTSTART:{dtstart}\r\n\
         DTEND:{dtend}\r\n\
         SUMMARY:{summary}\r\n\
         ORGANIZER:mailto:{organizer}\r\n\
         ATTENDEE;PARTSTAT={partstat}:mailto:{attendee}\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
        uid = event.uid,
        dtstart = event.dtstart,
        dtend = event.dtend,
        summary = event.summary,
        organizer = event.organizer,
        partstat = partstat,
        attendee = my_email,
    )
}