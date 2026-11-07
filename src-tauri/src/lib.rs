use chrono::{DateTime, Duration, LocalResult, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use roxmltree;
use serde::Serialize;
use std::{fs::File, io::Read};

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

            Ok(Event {
                event_id,
                title,

                start_time,
                end_time,
                displayed_start: Some(displayed_start),
                displayed_end: Some(displayed_end),

                error_front,
                error_back,
                has_error,
                slot_start_error,

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

            Ok(Event {
                event_id,
                title,

                start_time,
                end_time,
                displayed_start,
                displayed_end,

                error_front,
                error_back,
                has_error,
                slot_start_error,

                xml_string,
            })
        })
        .collect()
}

pub fn analyze_events(errors: &mut usize, mut events: Vec<Event>) -> Vec<Event> {
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

        events[i].has_error = events[i].error_front.is_some()
            || events[i].error_back.is_some()
            || events[i].slot_start_error;
        if events[i].has_error {
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

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub event_id: String,
    pub title: String,

    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub displayed_start: Option<DateTime<Utc>>,
    pub displayed_end: Option<DateTime<Utc>>,

    pub error_front: Option<TimeErrorType>,
    pub error_back: Option<TimeErrorType>,
    pub has_error: bool,
    pub slot_start_error: bool,

    pub xml_string: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ParsedEvents {
    pub si_events: Vec<Event>,
    pub va_events: Vec<Event>,
    pub si_events_errors: usize,
    pub va_events_errors: usize,
}

#[tauri::command]
fn parse_file(path: String) -> Result<ParsedEvents, String> {
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

    let analyzed_si_events = analyze_events(&mut si_events_errors, parsed_si_events);
    let analyzed_va_events = analyze_events(&mut va_events_errors, parsed_va_events);

    // let si_events_errors: usize = analyzed_si_events
    //     .iter()
    //     .filter(|event| event.has_error)
    //     .collect::<Vec<_>>()
    //     .len();

    // let va_events_errors: usize = analyzed_va_events
    //     .iter()
    //     .filter(|event| event.has_error)
    //     .collect::<Vec<_>>()
    //     .len();

    Ok(ParsedEvents {
        si_events: analyzed_si_events,
        va_events: analyzed_va_events,
        si_events_errors: si_events_errors,
        va_events_errors: va_events_errors,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![parse_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
