use chrono::{DateTime, Duration, LocalResult, NaiveDateTime, NaiveTime, TimeZone, Utc};
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
) -> Result<Vec<SiEventParsed>, String> {
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

            Ok(SiEventParsed {
                event_id,
                title,

                start_time,
                end_time,
                displayed_start,
                displayed_end,

                error_front,
                error_back,
                has_error,

                xml_string,
            })
        })
        .collect()
}

#[tauri::command]
fn parse_file(path: String) -> Result<Vec<SiEventParsed>, String> {
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

    let parsed = parse_si_events(&contents, si_events)?;

    Ok(analyze_si_events(parsed))
}

pub fn analyze_si_events(mut events: Vec<SiEventParsed>) -> Vec<SiEventParsed> {
    for i in 0..events.len() - 1 {
        let end_current = events[i].end_time;
        let start_next = events[i + 1].start_time;

        let diff = (start_next - end_current).num_milliseconds();

        if diff > 0 {
            events[i].error_back = Some(TimeErrorType::Gap);
            events[i + 1].error_front = Some(TimeErrorType::Gap);
        } else if diff < 0 {
            events[i].error_back = Some(TimeErrorType::Overlap);
            events[i + 1].error_front = Some(TimeErrorType::Overlap);
        }
    }

    for event in &mut events {
        event.has_error = event.error_front.is_some() || event.error_back.is_some();
    }

    events
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TimeErrorType {
    Gap,
    Overlap,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiEventParsed {
    pub event_id: String,
    pub title: String,

    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub displayed_start: DateTime<Utc>,
    pub displayed_end: DateTime<Utc>,

    pub error_front: Option<TimeErrorType>,
    pub error_back: Option<TimeErrorType>,
    pub has_error: bool,

    pub xml_string: String,
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
    fn test_parse_and_analyze_logic() -> std::io::Result<()> {
        let mut file = File::open("../../../Downloads/hdplus_20260702_51927.pts")?;
        let mut xml_input = String::new();
        file.read_to_string(&mut xml_input)?;

        let doc = roxmltree::Document::parse(&xml_input).unwrap();
        let si_events: Vec<roxmltree::Node<'_, '_>> = doc
            .descendants()
            .filter(|n| n.has_tag_name("siEvent"))
            .collect();

        let mut parsed = parse_si_events(&xml_input.to_string(), si_events);

        parsed = analyze_si_events(parsed);

        // assert_eq!(parsed.len(), 2);
        // assert_eq!(parsed[0].error_back, Some(TimeErrorType::Gap));
        // assert!(parsed[0].has_error);

        println!("{:#?}", parsed);

        Ok(())
    }
}
