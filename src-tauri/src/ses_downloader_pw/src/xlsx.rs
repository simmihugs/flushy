//! Reads a downloaded export: first sheet, header in row 1, one entry per row.
//! Only the `ContentId` and `RuntimeMs` columns are used.

use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use calamine::{open_workbook_auto, Data, Reader};
use chrono::NaiveTime;

use crate::store::{Entry, KEY_COLUMN, RUNTIME_COLUMN};

fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::Float(f) if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", *f as i64),
        other => other.to_string().trim().to_string(),
    }
}

pub fn a_duration_from_string(string: String) -> Option<i64> {
    let fmt = "%H:%M:%S.0%3f";
    let step = NaiveTime::from_hms_opt(0, 0, 0).unwrap();

    NaiveTime::parse_from_str(&string, fmt)
        .ok()
        .map(|time| (time - step).num_milliseconds())
}

fn runtime_ms(cell: &Data) -> Result<Option<i64>> {
    match cell {
        Data::Empty => Ok(None),
        Data::Int(i) => Ok(Some(*i)),
        Data::Float(f) => Ok(Some(f.round() as i64)),
        other => {
            let text = other.to_string();
            return Ok(a_duration_from_string(text));
        }
    }
}

pub fn read_xlsx(path: &Path) -> Result<Vec<Entry>> {
    let mut workbook =
        open_workbook_auto(path).with_context(|| format!("opening {}", path.display()))?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| anyhow!("{} has no sheets", path.display()))??;

    let mut lines = range.rows();
    let header: Vec<String> = lines
        .next()
        .ok_or_else(|| anyhow!("{} is empty", path.display()))?
        .iter()
        .map(cell_text)
        .collect();
    let find = |name: &str| -> Result<usize> {
        header.iter().position(|h| h == name).ok_or_else(|| {
            anyhow!(
                "{}: no '{name}' column; found headers: {:?}",
                path.display(),
                header
            )
        })
    };
    let key_idx = find(KEY_COLUMN)?;
    let runtime_idx = find(RUNTIME_COLUMN)?;

    let mut entries = Vec::new();
    for (i, line) in lines.enumerate() {
        let content_id = line.get(key_idx).map(cell_text).unwrap_or_default();
        if content_id.is_empty() {
            // Completely blank lines (e.g. trailing) are fine, anything else is not.
            if line
                .iter()
                .all(|c| matches!(c, Data::Empty) || cell_text(c).is_empty())
            {
                continue;
            }
            bail!("{}: row {} has no {KEY_COLUMN}", path.display(), i + 2);
        }
        let runtime = match line.get(runtime_idx) {
            Some(cell) => runtime_ms(cell)
                .with_context(|| format!("{}: row {} ({content_id})", path.display(), i + 2))?,
            None => None,
        };
        entries.push(Entry {
            content_id,
            runtime_ms: runtime,
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_xlsxwriter::Workbook;

    fn temp_file(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("xlsx-test-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("page.xlsx")
    }

    #[test]
    fn reads_content_id_and_runtime_and_ignores_other_columns() {
        let path = temp_file("ok");
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();
        for (c, h) in ["Class", KEY_COLUMN, "Filename", RUNTIME_COLUMN]
            .iter()
            .enumerate()
        {
            ws.write_string(0, c as u16, *h).unwrap();
        }
        ws.write_string(1, 0, "PROMO").unwrap();
        ws.write_string(1, 1, "1806520-1067636").unwrap();
        ws.write_string(1, 2, "a.mov").unwrap();
        ws.write_number(1, 3, 61000.0).unwrap(); // numeric runtime
        ws.write_string(2, 1, "X-9").unwrap();
        ws.write_string(2, 3, " 45000 ").unwrap(); // runtime as text
        ws.write_string(3, 1, "Y-1").unwrap(); // no runtime
        ws.write_string(5, 2, "").unwrap(); // blank line
        wb.save(&path).unwrap();

        let entries = read_xlsx(&path).unwrap();
        assert_eq!(
            entries,
            vec![
                Entry {
                    content_id: "1806520-1067636".into(),
                    runtime_ms: Some(61000)
                },
                Entry {
                    content_id: "X-9".into(),
                    runtime_ms: Some(45000)
                },
                Entry {
                    content_id: "Y-1".into(),
                    runtime_ms: None
                },
            ]
        );
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[test]
    fn missing_column_lists_the_headers() {
        let path = temp_file("missing");
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();
        ws.write_string(0, 0, "Filename").unwrap();
        ws.write_string(1, 0, "a.mov").unwrap();
        wb.save(&path).unwrap();

        let err = read_xlsx(&path).unwrap_err().to_string();
        assert!(
            err.contains(KEY_COLUMN) && err.contains("Filename"),
            "{err}"
        );
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[test]
    fn non_numeric_runtime_is_an_error_with_the_row() {
        let path = temp_file("bad");
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();
        ws.write_string(0, 0, KEY_COLUMN).unwrap();
        ws.write_string(0, 1, RUNTIME_COLUMN).unwrap();
        ws.write_string(1, 0, "A").unwrap();
        ws.write_string(1, 1, "abc").unwrap();
        wb.save(&path).unwrap();

        let err = format!("{:#}", read_xlsx(&path).unwrap_err());
        assert!(err.contains("abc") && err.contains("row 2"), "{err}");
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }
}
