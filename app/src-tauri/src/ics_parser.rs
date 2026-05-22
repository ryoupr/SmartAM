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


#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ICS: &str = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
BEGIN:VEVENT\r\n\
UID:test-uid-123@example.com\r\n\
DTSTART:20260615T100000Z\r\n\
DTEND:20260615T110000Z\r\n\
SUMMARY:Team Meeting\r\n\
LOCATION:Room 101\r\n\
ORGANIZER:mailto:boss@example.com\r\n\
ATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:user@example.com\r\n\
DESCRIPTION:Weekly sync\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    #[test]
    fn parse_basic_event() {
        let events = parse_ics(SAMPLE_ICS.as_bytes()).unwrap();
        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert_eq!(ev.uid, "test-uid-123@example.com");
        assert_eq!(ev.summary, "Team Meeting");
        assert_eq!(ev.dtstart, "20260615T100000Z");
        assert_eq!(ev.dtend, "20260615T110000Z");
        assert_eq!(ev.location, "Room 101");
        assert_eq!(ev.organizer, "boss@example.com");
        assert_eq!(ev.description, "Weekly sync");
    }

    #[test]
    fn parse_strips_mailto_prefix() {
        let events = parse_ics(SAMPLE_ICS.as_bytes()).unwrap();
        assert_eq!(events[0].organizer, "boss@example.com");
        assert!(events[0].attendees.contains(&"user@example.com".to_string()));
    }

    #[test]
    fn parse_empty_ics() {
        let data = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";
        let events = parse_ics(data.as_bytes()).unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn parse_invalid_ics() {
        let result = parse_ics(b"not valid ics data");
        // Parser returns empty vec for non-ICS data
        match result {
            Ok(events) => assert!(events.is_empty()),
            Err(_) => {} // Also acceptable
        }
    }

    #[test]
    fn generate_reply_accept() {
        let event = CalendarEvent {
            uid: "uid-1".into(),
            summary: "Meeting".into(),
            dtstart: "20260615T100000Z".into(),
            dtend: "20260615T110000Z".into(),
            location: "".into(),
            organizer: "boss@example.com".into(),
            attendees: vec![],
            status: "".into(),
            description: "".into(),
        };
        let reply = generate_reply_ics(&event, "me@example.com", true);
        assert!(reply.contains("METHOD:REPLY"));
        assert!(reply.contains("PARTSTAT=ACCEPTED"));
        assert!(reply.contains("mailto:me@example.com"));
        assert!(reply.contains("UID:uid-1"));
    }

    #[test]
    fn generate_reply_decline() {
        let event = CalendarEvent {
            uid: "uid-2".into(),
            summary: "Party".into(),
            dtstart: "20260620T180000Z".into(),
            dtend: "20260620T220000Z".into(),
            location: "".into(),
            organizer: "host@example.com".into(),
            attendees: vec![],
            status: "".into(),
            description: "".into(),
        };
        let reply = generate_reply_ics(&event, "me@example.com", false);
        assert!(reply.contains("PARTSTAT=DECLINED"));
    }

    #[test]
    fn parse_multiple_events() {
        let data = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n\
BEGIN:VEVENT\r\nUID:ev1\r\nDTSTART:20260601T090000Z\r\nDTEND:20260601T100000Z\r\nSUMMARY:Event 1\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:ev2\r\nDTSTART:20260602T090000Z\r\nDTEND:20260602T100000Z\r\nSUMMARY:Event 2\r\nEND:VEVENT\r\n\
END:VCALENDAR\r\n";
        let events = parse_ics(data.as_bytes()).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].uid, "ev1");
        assert_eq!(events[1].uid, "ev2");
    }
}
