use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Param {
    pub name: String,
    pub value: String,
}

const DEFAULT_ENV_TEMPLATE: &str = r#"SES_URL=""
EMAIL=""
PASSWD=""
DATABASE_URL=""
HEADLESS="true"
"#;

fn resolve_env_path(path: Option<String>) -> Result<PathBuf, String> {
    if let Some(p) = path {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
    Ok(PathBuf::from(".env"))
}

fn escape_env_val(val: &str) -> String {
    let escaped = val.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn unescape_env_val(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        let inner = &trimmed[1..trimmed.len() - 1];
        let mut out = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('r') => out.push('\r'),
                    Some('t') => out.push('\t'),
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some(other) => {
                        out.push('\\');
                        out.push(other);
                    }
                    None => out.push('\\'),
                }
            } else {
                out.push(c);
            }
        }
        out
    } else {
        trimmed.to_string()
    }
}

fn set_restrictive_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
}

fn parse_env_file(env_path: &Path) -> Result<Vec<Param>, String> {
    let contents = fs::read_to_string(env_path)
        .map_err(|e| format!("Fehler beim Lesen von {}: {e}", env_path.display()))?;

    let mut params = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let without_export = trimmed.strip_prefix("export ").unwrap_or(trimmed).trim();
        if let Some((key, val)) = without_export.split_once('=') {
            let name = key.trim().to_string();
            if !name.is_empty() {
                params.push(Param {
                    name,
                    value: unescape_env_val(val),
                });
            }
        }
    }
    Ok(params)
}

fn write_env_file(env_path: &Path, params: &[Param], merge_existing: bool) -> Result<(), String> {
    if let Some(parent) = env_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Fehler beim Anlegen des Verzeichnisses {}: {e}",
                    parent.display()
                )
            })?;
        }
    }

    let mut map: BTreeMap<String, String> = BTreeMap::new();

    if merge_existing && env_path.exists() {
        for existing in parse_env_file(env_path)? {
            map.insert(existing.name, escape_env_val(&existing.value));
        }
    }

    for p in params {
        let key = p.name.trim();
        if key.is_empty() {
            continue;
        }
        map.insert(key.to_string(), escape_env_val(&p.value));
    }

    let mut content = String::new();
    for (k, v) in map {
        content.push_str(&format!("{k}={v}\n"));
    }

    let tmp_path = env_path.with_extension("tmp");
    fs::write(&tmp_path, content)
        .map_err(|e| format!("Fehler beim Schreiben von {}: {e}", tmp_path.display()))?;
    set_restrictive_permissions(&tmp_path);

    fs::rename(&tmp_path, env_path)
        .map_err(|e| format!("Fehler beim Ersetzen von {}: {e}", env_path.display()))?;
    set_restrictive_permissions(env_path);
    Ok(())
}

#[tauri::command]
pub fn load_or_create_config(path: Option<String>) -> Result<Vec<Param>, String> {
    let env_path = resolve_env_path(path)?;

    if !env_path.exists() {
        if let Some(parent) = env_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "Fehler beim Anlegen des Verzeichnisses {}: {e}",
                        parent.display()
                    )
                })?;
            }
        }
        fs::write(&env_path, DEFAULT_ENV_TEMPLATE)
            .map_err(|e| format!("Fehler beim Erstellen von {}: {e}", env_path.display()))?;
        set_restrictive_permissions(&env_path);
    }

    parse_env_file(&env_path)
}

#[tauri::command]
pub fn save_config(path: Option<String>, params: Vec<Param>) -> Result<(), String> {
    let env_path = resolve_env_path(path)?;
    write_env_file(&env_path, &params, false)
}
