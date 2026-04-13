use ical::IcalParser;
use serde::Serialize;
use std::io::BufReader;

#[derive(Debug, Serialize, Clone)]
pub struct CalendarEvent {
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
