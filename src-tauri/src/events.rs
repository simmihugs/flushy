use chrono::{DateTime, Duration, LocalResult, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use roxmltree;
use serde::Serialize;
use std::collections::HashMap;
use std::{fs::File, io::Read};
use tauri::Manager;

pub fn a_duration_from_string(string: String) -> Result<i64, String> {
    let fmt = "00 %H:%M:%S%.3f";
    let step = NaiveTime::parse_from_str("00 00:00:00.000", "00 %H:%M:%S%.3f").unwrap();
    let naivetime = NaiveTime::parse_from_str(&string, fmt);
    match naivetime {
        Ok(time) => {
            let dur: Duration = time - step;
            Ok(dur.num_milliseconds())
        }
        Err(..) => Err(format!("invalid duration {}", string)),
    }
}

pub fn calculate_endtime(
    duration: i64,
    start_time: DateTime<Utc>,
) -> Result<DateTime<Utc>, String> {
    match chrono::TimeDelta::try_milliseconds(duration) {
        Some(duration) => Ok(start_time + duration),
        None => Err(format!("Invalid duration {}", duration)),
    }
}

pub fn a_starttime_from_str(string: String) -> Result<DateTime<Utc>, String> {
    let dt = NaiveDateTime::parse_from_str(&string, "%Y-%m-%dT%H:%M:%S%.3fZ");
    match dt {
        Ok(dtt) => {
            let dt2: LocalResult<DateTime<Utc>> = Utc.from_local_datetime(&dtt);
            Ok(dt2.unwrap())
        }
        Err(..) => Err(format!("invalid timestring {}", string)),
    }
}

pub fn parse_si_events(
    contents: &String,
    si_events: Vec<roxmltree::Node<'_, '_>>,
) -> Result<Vec<Event>, String> {
    si_events
        .iter()
        .map(|node| {
            let xml_string = String::from(&contents[node.range()]);
            let event_id = node
                .attribute("eventId")
                .ok_or("Fehler: EventId fehlt")?
                .to_string();
            let content_id = None;
            let title = node
                .attribute("title")
                .ok_or("Fehler: Title fehlt")?
                .to_string();

            let duration: i64 = a_duration_from_string(
                node.attribute("duration")
                    .ok_or("Fehler: duration fehlt")?
                    .to_string(),
            )?;
            let start_time = a_starttime_from_str(
                node.attribute("startTime")
                    .ok_or("Fehler: startTime fehlt")?
                    .to_string(),
            )?;
            let end_time = calculate_endtime(duration, start_time)?;
            let mut displayed_start: DateTime<chrono::Utc> = start_time;
            let mut displayed_end: DateTime<chrono::Utc> = end_time;

            for child in node.descendants().filter(|c| c.has_tag_name("siStandard")) {
                let dd = a_duration_from_string(
                    child
                        .attribute("displayedDuration")
                        .ok_or("Fehler: displayedDuration fehlt")?
                        .to_string(),
                )?;
                displayed_start = a_starttime_from_str(
                    child
                        .attribute("displayedStart")
                        .ok_or("Fehler: displayedStart fehlt")?
                        .to_string(),
                )?;
                displayed_end = calculate_endtime(dd, displayed_start)?;
            }

            let error_front = None;
            let error_back = None;
            let has_error = false;
            let slot_start_error = false;
            let duration_error = DurationCheckStatus::No;
            let db_duration = 0;

            Ok(Event {
                event_id,
                content_id,
                title,

                start_time,
                end_time,
                duration,
                db_duration,
                displayed_start: Some(displayed_start),
                displayed_end: Some(displayed_end),

                error_front,
                error_back,
                has_error,
                slot_start_error,
                duration_error,

                xml_string,
            })
        })
        .collect()
}

pub fn parse_va_events(
    contents: &String,
    va_events: Vec<roxmltree::Node<'_, '_>>,
) -> Result<Vec<Event>, String> {
    va_events
        .iter()
        .map(|node| {
            let xml_string = String::from(&contents[node.range()]);
            let event_id = node
                .attribute("eventId")
                .ok_or("Fehler: EventId fehlt")?
                .to_string();
            let content_id = Some(
                node.attribute("contentId")
                    .ok_or("Fehler: ContentId fehlt")?
                    .to_string(),
            );

            let title = node
                .attribute("title")
                .ok_or("Fehler: Title fehlt")?
                .to_string();

            let duration: i64 = a_duration_from_string(
                node.attribute("duration")
                    .ok_or("Fehler: duration fehlt")?
                    .to_string(),
            )?;
            let start_time = a_starttime_from_str(
                node.attribute("startTime")
                    .ok_or("Fehler: startTime fehlt")?
                    .to_string(),
            )?;
            let end_time = calculate_endtime(duration, start_time)?;
            let displayed_start = None;
            let displayed_end = None;

            let error_front = None;
            let error_back = None;
            let has_error = false;
            let slot_start_error = false;
            let duration_error = DurationCheckStatus::Maybe;
            let db_duration = 0;

            Ok(Event {
                event_id,
                content_id,
                title,

                start_time,
                end_time,
                duration,
                db_duration,
                displayed_start,
                displayed_end,

                error_front,
                error_back,
                has_error,
                slot_start_error,
                duration_error,

                xml_string,
            })
        })
        .collect()
}

pub fn analyze_events(
    errors: &mut usize,
    mut events: Vec<Event>,
    runtimes: &HashMap<String, Option<i64>>,
    maybe_ids: &mut Vec<String>,
) -> Vec<Event> {
    if events.is_empty() {
        return vec![];
    }

    for i in 0..events.len() - 1 {
        let end_current = events[i].end_time;
        let start_next = events[i + 1].start_time;
        let diff = (start_next - end_current).num_milliseconds();

        if diff > 0 {
            events[i].error_back = Some(TimeErrorType::Gap(diff));
            events[i + 1].error_front = Some(TimeErrorType::Gap(diff));
        } else if diff < 0 {
            events[i].error_back = Some(TimeErrorType::Overlap(diff));
            events[i + 1].error_front = Some(TimeErrorType::Overlap(diff));
        }

        if events[i].title.contains("Sendepause") {
            let st = events[i].start_time;
            let minutes = st.minute();
            let seconds = st.second();
            let nanoseconds = st.nanosecond();

            if seconds == 0 && nanoseconds == 0 && minutes % 5 != 0 {
                events[i].slot_start_error = true;
                events[i].title = "P7S1 - Slot".to_string();
            }
        }

        if let Some(cid) = events[i].content_id.clone() {
            let db_entry = runtimes.get(&cid).copied();
            (events[i].duration_error, events[i].db_duration) =
                evaluate_duration(events[i].duration, db_entry);

            if events[i].duration_error == DurationCheckStatus::Maybe && !maybe_ids.contains(&cid) {
                maybe_ids.push(cid);
            }
        }
        if events[i].duration_error == DurationCheckStatus::Yes {
            crate::log_warn!(
                "duration",
                "Duration Error: event_id={} content_id={:?} title={:?} db_ms={} < event_ms={}",
                events[i].event_id,
                events[i].content_id,
                events[i].title,
                events[i].db_duration,
                events[i].duration
            );
        }

        events[i].has_error = events[i].error_front.is_some()
            || events[i].error_back.is_some()
            || events[i].slot_start_error
            || events[i].duration_error == DurationCheckStatus::Yes;

        if events[i].has_error {
            *errors += 1;
        }
    }

    if let Some(last) = events.last_mut() {
        if let Some(ref cid) = last.content_id {
            let db_entry = runtimes.get(cid).copied();
            (last.duration_error, _) = evaluate_duration(last.duration, db_entry);
            if last.duration_error == DurationCheckStatus::Maybe && !maybe_ids.contains(cid) {
                maybe_ids.push(cid.clone());
            }
        }
        last.has_error = last.error_front.is_some()
            || last.slot_start_error
            || last.duration_error == DurationCheckStatus::Yes;
        if last.has_error {
            *errors += 1;
        }
    }

    events
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TimeErrorType {
    Gap(i64),
    Overlap(i64),
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DurationCheckStatus {
    No,
    Yes,
    Maybe,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub event_id: String,
    pub content_id: Option<String>,
    pub title: String,

    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration: i64,
    pub db_duration: i64,
    pub displayed_start: Option<DateTime<Utc>>,
    pub displayed_end: Option<DateTime<Utc>>,

    pub error_front: Option<TimeErrorType>,
    pub error_back: Option<TimeErrorType>,
    pub has_error: bool,
    pub slot_start_error: bool,
    pub duration_error: DurationCheckStatus,

    pub xml_string: String,
}

pub fn evaluate_duration(
    event_duration_ms: i64,
    db_runtime_ms: Option<Option<i64>>,
) -> (DurationCheckStatus, i64) {
    match db_runtime_ms {
        Some(Some(db_ms)) => {
            if db_ms >= event_duration_ms {
                (DurationCheckStatus::No, 0)
            } else {
                (DurationCheckStatus::Yes, db_ms)
            }
        }
        None | Some(None) => (DurationCheckStatus::Maybe, 0),
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ParsedEvents {
    pub si_events: Vec<Event>,
    pub va_events: Vec<Event>,
    pub si_events_errors: usize,
    pub va_events_errors: usize,
    pub maybe_event_ids: Vec<String>,
}

#[tauri::command]
pub fn parse_file(
    app: tauri::AppHandle,
    path: String,
    db: Option<String>,
) -> Result<ParsedEvents, String> {
    let runtimes: HashMap<String, Option<i64>> = app
        .path()
        .app_data_dir()
        .ok()
        .and_then(|dir| {
            let db_file = db
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("ses.db");
            let full_path = dir.join(db_file);
            if !full_path.exists() {
                return None;
            }
            let store = ses_downloader_pw::store::Store::open(&full_path).ok()?;
            let mut file = File::open(&path).ok()?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).ok()?;
            let doc = roxmltree::Document::parse(&contents).ok()?;
            let mut ids: Vec<String> = doc
                .descendants()
                .filter(|n| n.has_tag_name("vaEvent"))
                .filter_map(|n| n.attribute("contentId"))
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            ids.sort();
            ids.dedup();
            store.get_runtimes_for_ids(&ids).ok()
        })
        .unwrap_or_default();

    parse_file_with_runtimes(path, &runtimes)
}

fn parse_file_with_runtimes(
    path: String,
    runtimes: &HashMap<String, Option<i64>>,
) -> Result<ParsedEvents, String> {
    let mut file = File::open(&path).map_err(|e| format!("Fehler beim Öffnen der Datei: {}", e))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Fehler beim Lesen der Datei: {}", e))?;

    let doc = roxmltree::Document::parse(&contents)
        .map_err(|e| format!("Ungültiges XML-Format: {}", e))?;

    let si_events: Vec<roxmltree::Node<'_, '_>> = doc
        .descendants()
        .filter(|n| n.has_tag_name("siEvent"))
        .collect();

    let va_events: Vec<roxmltree::Node<'_, '_>> = doc
        .descendants()
        .filter(|n| n.has_tag_name("vaEvent"))
        .collect();

    let parsed_si_events = parse_si_events(&contents, si_events)?;
    let parsed_va_events = parse_va_events(&contents, va_events)?;

    let mut si_events_errors: usize = 0;
    let mut va_events_errors: usize = 0;

    let mut maybe_event_ids = vec![];

    let analyzed_si_events = analyze_events(
        &mut si_events_errors,
        parsed_si_events,
        runtimes,
        &mut maybe_event_ids,
    );
    let analyzed_va_events = analyze_events(
        &mut va_events_errors,
        parsed_va_events,
        runtimes,
        &mut maybe_event_ids,
    );

    crate::log_info!(
        "events",
        "Geparst: {path} (SI: {}, VA: {}, SI-Errors: {}, VA-Errors: {}, Maybes: {})",
        analyzed_si_events.len(),
        analyzed_va_events.len(),
        si_events_errors,
        va_events_errors,
        maybe_event_ids.len()
    );

    Ok(ParsedEvents {
        si_events: analyzed_si_events,
        va_events: analyzed_va_events,
        si_events_errors: si_events_errors,
        va_events_errors: va_events_errors,
        maybe_event_ids: maybe_event_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Die Testdaten sind eingebettet, damit die Tests weder von Dateien im
    // Download-Ordner noch von Tauri (AppHandle) abhängen. Getestet wird die
    // reine Logik über `parse_file_with_runtimes`.

    /// Schreibt `xml` in eine temporäre Datei, parst sie und räumt wieder auf.
    fn parse(
        name: &str,
        xml: &str,
        runtimes: &HashMap<String, Option<i64>>,
    ) -> Result<ParsedEvents, String> {
        let path =
            std::env::temp_dir().join(format!("events-test-{}-{name}.pts", std::process::id()));
        std::fs::write(&path, xml).unwrap();
        let result = parse_file_with_runtimes(path.to_string_lossy().into_owned(), runtimes);
        let _ = std::fs::remove_file(&path);
        result
    }

    fn no_runtimes() -> HashMap<String, Option<i64>> {
        HashMap::new()
    }

    #[test]
    fn parses_durations_in_milliseconds() {
        let ms = |s: &str| a_duration_from_string(s.to_string()).unwrap();
        assert_eq!(ms("00 00:34:00.000"), 2_040_000);
        assert_eq!(ms("00 00:32:46.040"), 1_966_040);
        assert_eq!(ms("00 02:45:46.360"), 9_946_360);
        assert!(a_duration_from_string("00:34:00".to_string()).is_err());
        assert!(a_duration_from_string("quatsch".to_string()).is_err());
    }

    #[test]
    fn parses_start_times_as_utc() {
        let t = a_starttime_from_str("2026-11-07T19:00:00.000Z".to_string()).unwrap();
        assert_eq!(t.to_rfc3339(), "2026-11-07T19:00:00+00:00");
        assert!(a_starttime_from_str("2026-11-07 19:00".to_string()).is_err());
    }

    #[test]
    fn evaluate_duration_compares_db_runtime_with_planned_duration() {
        use DurationCheckStatus::*;
        assert_eq!(evaluate_duration(2_040_000, Some(Some(2_040_000))), (No, 0));
        assert_eq!(evaluate_duration(2_040_000, Some(Some(2_100_000))), (No, 0));
        assert_eq!(
            evaluate_duration(2_040_000, Some(Some(1_966_040))),
            (Yes, 1_966_040)
        );
        assert_eq!(evaluate_duration(2_040_000, Some(None)), (Maybe, 0));
        assert_eq!(evaluate_duration(2_040_000, None), (Maybe, 0));
    }

    const CLEAN: &str = r#"<?xml version="1.0"?>
<root>
  <siEvent eventId="S1" title="Show" duration="00 00:30:00.000" startTime="2026-11-07T19:00:00.000Z">
    <siStandard displayedDuration="00 00:29:00.000" displayedStart="2026-11-07T19:01:00.000Z"/>
  </siEvent>
  <siEvent eventId="S2" title="Show 2" duration="00 00:05:00.000" startTime="2026-11-07T19:30:00.000Z"/>
  <vaEvent eventId="V1" contentId="C1" title="Film" duration="00 00:34:00.000" startTime="2026-11-07T19:00:00.000Z"/>
  <vaEvent eventId="V2" contentId="C2" title="Serie" duration="00 00:10:00.000" startTime="2026-11-07T19:34:00.000Z"/>
</root>"#;

    #[test]
    fn clean_file_has_no_errors() {
        let runtimes = HashMap::from([
            ("C1".to_string(), Some(2_100_000)),
            ("C2".to_string(), Some(600_000)),
        ]);
        let p = parse("clean", CLEAN, &runtimes).unwrap();

        assert_eq!(p.si_events.len(), 2);
        assert_eq!(p.va_events.len(), 2);
        assert_eq!((p.si_events_errors, p.va_events_errors), (0, 0));
        assert!(p.maybe_event_ids.is_empty());
        assert_eq!(p.si_events[0].duration, 1_800_000);
        assert_eq!(
            p.si_events[0].displayed_start.unwrap().to_rfc3339(),
            "2026-11-07T19:01:00+00:00"
        );
    }

    #[test]
    fn too_long_scheduled_video_is_an_error_and_unknown_runtime_is_a_maybe() {
        // C1 ist laut DB kürzer (32:46,040) als geplant (34:00); C2 fehlt in der DB.
        let runtimes = HashMap::from([("C1".to_string(), Some(1_966_040))]);
        let p = parse("too_long", CLEAN, &runtimes).unwrap();

        let v1 = &p.va_events[0];
        assert_eq!(v1.duration_error, DurationCheckStatus::Yes);
        assert_eq!(v1.db_duration, 1_966_040);
        assert!(v1.has_error);
        assert_eq!(p.va_events_errors, 1);

        assert_eq!(p.va_events[1].duration_error, DurationCheckStatus::Maybe);
        assert_eq!(p.maybe_event_ids, vec!["C2".to_string()]);
    }

    #[test]
    fn gap_between_events_is_reported_on_both_sides() {
        let xml = r#"<root>
  <vaEvent eventId="V1" contentId="C1" title="A" duration="00 00:10:00.000" startTime="2026-11-07T19:00:00.000Z"/>
  <vaEvent eventId="V2" contentId="C2" title="B" duration="00 00:10:00.000" startTime="2026-11-07T19:12:00.000Z"/>
</root>"#;
        let p = parse("gap", xml, &no_runtimes()).unwrap();
        assert_eq!(p.va_events[0].error_back, Some(TimeErrorType::Gap(120_000)));
        assert_eq!(
            p.va_events[1].error_front,
            Some(TimeErrorType::Gap(120_000))
        );
        assert_eq!(p.va_events_errors, 2);
    }

    #[test]
    fn overlap_between_events_is_negative() {
        let xml = r#"<root>
  <vaEvent eventId="V1" contentId="C1" title="A" duration="00 00:10:00.000" startTime="2026-11-07T19:00:00.000Z"/>
  <vaEvent eventId="V2" contentId="C2" title="B" duration="00 00:10:00.000" startTime="2026-11-07T19:08:00.000Z"/>
</root>"#;
        let p = parse("overlap", xml, &no_runtimes()).unwrap();
        assert_eq!(
            p.va_events[0].error_back,
            Some(TimeErrorType::Overlap(-120_000))
        );
    }

    #[test]
    fn sendepause_not_on_a_five_minute_slot_is_flagged() {
        let xml = r#"<root>
  <siEvent eventId="S1" title="Sendepause" duration="00 00:02:00.000" startTime="2026-11-07T19:03:00.000Z"/>
  <siEvent eventId="S2" title="Show" duration="00 00:10:00.000" startTime="2026-11-07T19:05:00.000Z"/>
</root>"#;
        let p = parse("slot", xml, &no_runtimes()).unwrap();
        assert!(p.si_events[0].slot_start_error);
        assert_eq!(p.si_events[0].title, "P7S1 - Slot");
        assert_eq!(p.si_events_errors, 1);
    }

    #[test]
    fn missing_attribute_gives_a_readable_error() {
        let xml = r#"<root>
  <vaEvent eventId="V1" title="A" duration="00 00:10:00.000" startTime="2026-11-07T19:00:00.000Z"/>
</root>"#;
        let err = parse("missing", xml, &no_runtimes()).unwrap_err();
        assert_eq!(err, "Fehler: ContentId fehlt");
    }

    #[test]
    fn broken_xml_and_missing_file_are_errors_not_panics() {
        let err = parse("broken", "<root><vaEvent>", &no_runtimes()).unwrap_err();
        assert!(err.starts_with("Ungültiges XML-Format"), "{err}");

        let err =
            parse_file_with_runtimes("/gibt/es/nicht.pts".to_string(), &no_runtimes()).unwrap_err();
        assert!(err.starts_with("Fehler beim Öffnen der Datei"), "{err}");
    }

    /// Optionaler Test mit einer echten Datei, ohne festen Pfad im Code:
    ///   PTS_TEST_FILE=/pfad/zu/hdplus_20261107_54526.pts cargo test real_file
    /// Ohne die Variable wird der Test übersprungen.
    #[test]
    fn real_file_smoke_test() {
        let Ok(path) = std::env::var("PTS_TEST_FILE") else {
            eprintln!("PTS_TEST_FILE nicht gesetzt, übersprungen");
            return;
        };
        let p = parse_file_with_runtimes(path, &no_runtimes()).expect("Datei muss parsebar sein");
        println!(
            "{} si_events, {} va_events",
            p.si_events.len(),
            p.va_events.len()
        );
    }
}
