//! SQLite store: one row per entry, keyed by `ContentId`, with its `RuntimeMs`.

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

use anyhow::{bail, Result};
use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoredEntry {
    pub content_id: String,
    pub runtime_ms: Option<i64>,
    pub first_seen: String,
}

/// Excel column that uniquely identifies an entry.
pub const KEY_COLUMN: &str = "ContentId";
/// Excel column whose value is stored.
pub const RUNTIME_COLUMN: &str = "RuntimeMs";

/// Bump when the table layout changes.
const SCHEMA_VERSION: i64 = 2;

/// What we keep from one Excel row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub content_id: String,
    pub runtime_ms: Option<i64>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Ingest {
    /// Entries that were not in the DB and have now been inserted.
    pub new_rows: usize,
    /// Entries that were already in the DB from an earlier run.
    pub known_rows: usize,
}

pub struct Store {
    conn: Connection,
    /// Ids inserted during this run. The result list can shift while we page
    /// through it (new entries arrive at the front), so the same entry may
    /// show up on two pages; that must not count as "already in the DB".
    inserted_this_run: HashSet<String>,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    pub fn list_entries(
        &self,
        search: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<StoredEntry>> {
        let lim = limit.min(1000) as i64;
        let off = offset as i64;
        let query = search.unwrap_or("").trim();

        let mut out = Vec::new();
        if query.is_empty() {
            let mut stmt = self.conn.prepare(
                "SELECT content_id, runtime_ms, first_seen
                 FROM items
                 ORDER BY first_seen DESC, content_id ASC
                 LIMIT ?1 OFFSET ?2",
            )?;
            let rows = stmt.query_map(params![lim, off], |r| {
                Ok(StoredEntry {
                    content_id: r.get(0)?,
                    runtime_ms: r.get(1)?,
                    first_seen: r.get(2)?,
                })
            })?;
            for row in rows {
                out.push(row?);
            }
        } else {
            let pattern = format!("%{query}%");
            let mut stmt = self.conn.prepare(
                "SELECT content_id, runtime_ms, first_seen
                 FROM items
                 WHERE content_id LIKE ?1
                 ORDER BY first_seen DESC, content_id ASC
                 LIMIT ?2 OFFSET ?3",
            )?;
            let rows = stmt.query_map(params![pattern, lim, off], |r| {
                Ok(StoredEntry {
                    content_id: r.get(0)?,
                    runtime_ms: r.get(1)?,
                    first_seen: r.get(2)?,
                })
            })?;
            for row in rows {
                out.push(row?);
            }
        }

        Ok(out)
    }

    pub fn get_runtimes_for_ids(&self, ids: &[String]) -> Result<HashMap<String, Option<i64>>> {
        let mut map = HashMap::with_capacity(ids.len());
        if ids.is_empty() {
            return Ok(map);
        }

        let mut stmt = self
            .conn
            .prepare("SELECT runtime_ms FROM items WHERE content_id = ?1")?;

        for id in ids {
            let mut rows = stmt.query(params![id])?;
            if let Some(row) = rows.next()? {
                let rt: Option<i64> = row.get(0)?;
                map.insert(id.clone(), rt);
            }
        }
        Ok(map)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        let has_items: bool = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'items'",
            [],
            |r| r.get::<_, i64>(0),
        )? > 0;
        match (version, has_items) {
            (SCHEMA_VERSION, _) | (0, false) => {}
            _ => bail!(
                "the database has an old format (user_version {version}); \
                 delete the ses.db file and run again"
            ),
        }

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS items (
                 content_id TEXT PRIMARY KEY,
                 runtime_ms INTEGER,
                 first_seen TEXT NOT NULL DEFAULT (datetime('now'))
             );
             CREATE TABLE IF NOT EXISTS meta (
                 key   TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );",
        )?;
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        Ok(Self {
            conn,
            inserted_this_run: HashSet::new(),
        })
    }

    /// Starts a new run: entries inserted from now on count as "this run".
    pub fn begin_run(&mut self) {
        self.inserted_this_run.clear();
    }

    /// Inserts every entry that is not yet in the DB (all entries of the page
    /// are processed, even if some are known) and reports how many were
    /// new/known.
    pub fn ingest(&mut self, entries: &[Entry]) -> Result<Ingest> {
        let tx = self.conn.transaction()?;
        let mut out = Ingest::default();
        {
            let mut stmt =
                tx.prepare("INSERT OR IGNORE INTO items (content_id, runtime_ms) VALUES (?1, ?2)")?;
            for e in entries {
                if stmt.execute(params![e.content_id, e.runtime_ms])? == 1 {
                    out.new_rows += 1;
                    self.inserted_this_run.insert(e.content_id.clone());
                } else if !self.inserted_this_run.contains(&e.content_id) {
                    out.known_rows += 1;
                }
            }
        }
        tx.commit()?;
        Ok(out)
    }

    pub fn count(&self) -> Result<usize> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM items", [], |r| r.get::<_, i64>(0))?
            as usize)
    }

    /// True once a run has reached the end of the list at least once. Until
    /// then the DB may have gaps (e.g. a crashed first run), so early stopping
    /// must not be applied.
    pub fn initial_sync_complete(&self) -> Result<bool> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*) FROM meta WHERE key = 'initial_sync_complete'",
            [],
            |r| r.get::<_, i64>(0),
        )? > 0)
    }

    pub fn mark_initial_sync_complete(&self) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('initial_sync_complete', '1')",
            [],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, ms: i64) -> Entry {
        Entry {
            content_id: id.to_string(),
            runtime_ms: Some(ms),
        }
    }

    #[test]
    fn counts_new_and_known_and_inserts_the_new_ones() {
        let mut store = Store::open_in_memory().unwrap();
        let first = store
            .ingest(&[entry("A", 1), entry("B", 2), entry("C", 3)])
            .unwrap();
        assert_eq!(
            first,
            Ingest {
                new_rows: 3,
                known_rows: 0
            }
        );

        // A later run: "C" is known, "D" is new and must still be inserted.
        store.begin_run();
        let second = store.ingest(&[entry("D", 4), entry("C", 3)]).unwrap();
        assert_eq!(
            second,
            Ingest {
                new_rows: 1,
                known_rows: 1
            }
        );
        assert_eq!(store.count().unwrap(), 4);
    }

    #[test]
    fn a_row_repeated_within_one_run_is_not_counted_as_known() {
        let mut store = Store::open_in_memory().unwrap();
        store.ingest(&[entry("A", 1), entry("B", 2)]).unwrap();
        // The list shifted: "B" shows up again on the next page of the same run.
        let next_page = store.ingest(&[entry("B", 2), entry("C", 3)]).unwrap();
        assert_eq!(
            next_page,
            Ingest {
                new_rows: 1,
                known_rows: 0
            }
        );
    }

    #[test]
    fn stores_the_runtime_value() {
        let mut store = Store::open_in_memory().unwrap();
        store
            .ingest(&[
                entry("A", 61_000),
                Entry {
                    content_id: "B".into(),
                    runtime_ms: None,
                },
            ])
            .unwrap();
        let a: Option<i64> = store
            .conn
            .query_row(
                "SELECT runtime_ms FROM items WHERE content_id = 'A'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let b: Option<i64> = store
            .conn
            .query_row(
                "SELECT runtime_ms FROM items WHERE content_id = 'B'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!((a, b), (Some(61_000), None));
    }

    #[test]
    fn sync_flag_round_trips() {
        let store = Store::open_in_memory().unwrap();
        assert!(!store.initial_sync_complete().unwrap());
        store.mark_initial_sync_complete().unwrap();
        assert!(store.initial_sync_complete().unwrap());
    }

    #[test]
    fn old_database_format_is_rejected_with_a_clear_message() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE items (material_id TEXT PRIMARY KEY, data TEXT);")
            .unwrap();
        let err = Store::init(conn).err().expect("must fail").to_string();
        assert!(err.contains("old format"), "{err}");
    }
}
