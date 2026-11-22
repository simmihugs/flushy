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

    #[test]
    fn test_si_parsing() -> std::io::Result<()> {
        match parse_file("../../../Downloads/6.11/hdplus_20261107_54526.pts".to_string()) {
            Err(..) => (),
            Ok(events) => println!("{} si_events", events.si_events.len()),
        }

        Ok(())
    }

    #[test]
    fn test_va_parsing() -> std::io::Result<()> {
        match parse_file("../../../Downloads/6.11/hdplus_20261107_54526.pts".to_string()) {
            Err(..) => (),
            Ok(events) => println!("{} va_events", events.va_events.len()),
        }

        Ok(())
    }
}
