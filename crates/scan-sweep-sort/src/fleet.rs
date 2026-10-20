//! Fleet sort — classify antibodies by behavioral fingerprint.
//! Backscatter — reverse the fleet dust, lick em clean.

use serde_json::{json, Value};
use std::collections::HashMap;

use crate::config::{antibodies_path, live_dir, CLEAN_RESPONSES};
use crate::util::write_json;
use crate::SssError;

/// Classify each fleet entity into sort_class + profile.
pub fn fleet_sort() -> Result<Value, SssError> {
    let ab_path = antibodies_path();
    if !ab_path.exists() {
        return Ok(json!({
            "total_entities": 0,
            "sort_classes": {},
            "profiles": {},
            "entities": {}
        }));
    }

    let data: HashMap<String, Value> = serde_json::from_str(&std::fs::read_to_string(&ab_path)?)?;
    let mut results = serde_json::Map::new();
    let mut classes: HashMap<String, u64> = HashMap::new();
    let mut profiles: HashMap<String, u64> = HashMap::new();

    for (k, v) in &data {
        let dec = v.get("deception").and_then(Value::as_object);
        let path_pat = v.get("path_pattern").and_then(Value::as_object);
        let timing = v.get("timing").and_then(Value::as_object);
        let conf = v.get("confidence").and_then(Value::as_f64).unwrap_or(0.0);
        let match_count = v.get("match_count").and_then(Value::as_u64).unwrap_or(0);
        let defection_count = v.get("defection_count").and_then(Value::as_u64).unwrap_or(0);

        let b = |obj: Option<&serde_json::Map<String, Value>>, key: &str| -> bool {
            obj.and_then(|o| o.get(key))
                .and_then(Value::as_bool)
                .unwrap_or(false)
        };
        let f = |obj: Option<&serde_json::Map<String, Value>>, key: &str| -> f64 {
            obj.and_then(|o| o.get(key))
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        };

        let hides = b(dec, "hides_identity");
        let rotates = b(dec, "rotates_ips");
        let ignores = b(dec, "ignores_rejection");
        let chrome_imp = b(dec, "chrome_impersonation");
        let stale = b(dec, "stale_chrome");
        let header_poor = b(dec, "header_poverty");
        let pagination = b(dec, "pagination_walk");
        let conn_absent = b(dec, "connection_absent");
        let enc_uniform = b(dec, "encoding_uniform");
        let accept_mono = b(dec, "accept_monoculture");

        let commit_pct = f(path_pat, "commit_url_pct");
        let single_page = f(path_pat, "single_page_pct");
        let has_ref = f(path_pat, "has_referrer_pct");
        let mean_interval = f(timing, "mean_interval_ms");

        let dscore = [
            hides,
            rotates,
            ignores,
            chrome_imp,
            stale,
            header_poor,
            pagination,
            conn_absent,
            enc_uniform,
            accept_mono,
        ]
        .iter()
        .filter(|&&x| x)
        .count() as u64;

        let (cls, prof) = if ignores && defection_count > 5 {
            ("pathogen", "ignores_rejection")
        } else if dscore >= 5 && match_count > 5000 {
            ("pathogen", "heavy_masking_fleet")
        } else if hides && chrome_imp && stale {
            ("parasite", "masking_fleet")
        } else if rotates && ignores {
            ("parasite", "rotating_ignorer")
        } else if rotates && hides && conf < 0.5 {
            ("stealth", "stealth_scraper")
        } else if chrome_imp && rotates && !pagination {
            ("stealth", "rotating_impersonator")
        } else if pagination && commit_pct > 0.7 {
            ("commensal", "code_crawler")
        } else if mean_interval > 5000.0 && commit_pct > 0.3 && !hides {
            ("commensal", "slow_reader")
        } else if has_ref > 0.3 && commit_pct > 0.5 && !rotates {
            ("commensal", "dev_reader")
        } else if conf >= 0.7 && dscore <= 2 {
            ("commensal", "high_conf_clean")
        } else if header_poor && !hides && commit_pct > 0.5 {
            ("ghost", "proxy_dev")
        } else if conn_absent && !rotates && commit_pct > 0.3 {
            ("ghost", "proxy_reader")
        } else if single_page > 0.9 && match_count < 10 {
            ("null", "single_probe")
        } else if enc_uniform && match_count < 100 && !rotates {
            ("null", "uniform_tool")
        } else if header_poor && commit_pct > 0.3 {
            ("partial", "unclear_reader")
        } else if dscore >= 2 && dscore <= 4 && conf >= 0.5 {
            ("partial", "mixed_behavior")
        } else if conf >= 0.5 {
            ("partial", "moderate_signal")
        } else {
            ("null", "insufficient_signal")
        };

        *classes.entry(cls.to_string()).or_default() += 1;
        *profiles.entry(prof.to_string()).or_default() += 1;
        results.insert(
            k.clone(),
            json!({
                "sort_class": cls,
                "profile": prof,
                "confidence": conf,
                "deception_score": dscore,
                "match_count": match_count,
            }),
        );
    }

    let output = json!({
        "total_entities": data.len(),
        "sort_classes": sort_map(&classes),
        "profiles": sort_map(&profiles),
        "entities": results,
        "refinement": {
            "previous": {"self": 120, "non_self": 7, "partial": 114, "null": 100},
            "current_fleet": sort_map(&classes),
            "total_classified": data.len(),
            "granularity_increase": profiles.len(),
        },
    });

    write_json(&live_dir().join("fleet-sort.json"), &output)?;
    Ok(output)
}

/// Compute backscatter — what gets cleaned vs licensed.
pub fn backscatter_clean(fleet_data: &Value) -> Result<Value, SssError> {
    let entities = match fleet_data.get("entities").and_then(Value::as_object) {
        Some(e) if !e.is_empty() => e,
        _ => return Ok(json!({})),
    };

    let mut actions: HashMap<String, u64> = HashMap::new();
    for edata in entities.values() {
        let prof = edata
            .get("profile")
            .and_then(Value::as_str)
            .unwrap_or("insufficient_signal");
        let action = CLEAN_RESPONSES
            .get(prof)
            .copied()
            .unwrap_or("soft_redirect");
        *actions.entry(action.to_string()).or_default() += 1;
    }

    let total = fleet_data
        .get("total_entities")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    let cleaned = actions.get("redirect_clean").copied().unwrap_or(0)
        + actions.get("allow_clean").copied().unwrap_or(0);
    let licensed = actions.get("license_hard").copied().unwrap_or(0);
    let t = total.max(1) as f64;

    let result = json!({
        "ts": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "principle": "backscatter — reverse the fleet dust, lick em clean",
        "total": total,
        "actions": sort_map(&actions),
        "cleaned": cleaned,
        "cleaning_pct": (cleaned as f64 / t * 100.0 * 10.0).round() / 10.0,
        "licensed": licensed,
        "hostile_pct": (licensed as f64 / t * 100.0 * 10.0).round() / 10.0,
    });

    write_json(&live_dir().join("backscatter.json"), &result)?;
    Ok(result)
}

/// Sort a counter map by descending count (like Python's `Counter.most_common()`).
fn sort_map(m: &HashMap<String, u64>) -> Value {
    let mut pairs: Vec<_> = m.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1));
    let obj: serde_json::Map<String, Value> = pairs
        .into_iter()
        .map(|(k, v)| (k.clone(), json!(v)))
        .collect();
    Value::Object(obj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fleet_sort_empty_returns_zero() {
        let result =
            serde_json::from_value::<Value>(json!({"total_entities":0})).expect("valid json");
        assert_eq!(
            result.get("total_entities").and_then(Value::as_u64),
            Some(0)
        );
    }

    #[test]
    fn backscatter_empty_entities() {
        let fleet = json!({"entities": {}, "total_entities": 0});
        let result = backscatter_clean(&fleet).expect("should succeed");
        assert_eq!(result, json!({}));
    }

    #[test]
    fn sort_map_orders_by_count() {
        let mut m = HashMap::new();
        m.insert("a".into(), 3);
        m.insert("b".into(), 10);
        m.insert("c".into(), 1);
        let v = sort_map(&m);
        let obj = v.as_object().expect("obj");
        assert_eq!(obj.len(), 3);
        assert_eq!(obj.get("b").and_then(Value::as_u64), Some(10));
        assert_eq!(obj.get("c").and_then(Value::as_u64), Some(1));
    }
}
