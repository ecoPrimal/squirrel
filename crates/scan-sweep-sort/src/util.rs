//! I/O utilities — fetch URL, write JSON atomically, append to journal.

use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::SssError;

/// Fetch JSON from a URL with a 15-second timeout.
pub fn fetch(url: &str) -> Value {
    match fetch_inner(url) {
        Ok(v) => v,
        Err(e) => serde_json::json!({"_error": e.to_string()}),
    }
}

fn fetch_inner(url: &str) -> Result<Value, SssError> {
    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(std::time::Duration::from_secs(15)))
            .build(),
    );
    let body: String = agent
        .get(url)
        .header("User-Agent", "squirrel-scan/0.2-rs")
        .call()?
        .body_mut()
        .read_to_string()?;
    let v: Value = serde_json::from_str(&body)?;
    Ok(v)
}

/// Atomically write JSON to `path` (write tmp + rename).
pub fn write_json(path: &Path, data: &Value) -> Result<(), SssError> {
    let tmp = path.with_extension("json.tmp");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(&tmp)?;
    serde_json::to_writer_pretty(&mut f, data)?;
    f.flush()?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Append a JSONL entry to `journal_dir/subdir/YYYY-MM-DD.jsonl`.
pub fn journal_append(subdir: &str, entry: &Value) -> Result<(), SssError> {
    let base = crate::config::journal_dir().join(subdir);
    fs::create_dir_all(&base)?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let path = base.join(format!("{today}.jsonl"));
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(f, "{}", serde_json::to_string(entry)?)?;
    Ok(())
}
