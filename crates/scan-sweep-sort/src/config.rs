//! Constants and configuration — FEEDS, CLEAN_RESPONSES, CATEGORY_SITES, DEPTH_ORDER.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

/// π constant.
pub const PI: f64 = std::f64::consts::PI;

/// Base path for live terminal data.
pub fn live_dir() -> PathBuf {
    PathBuf::from("/opt/membrane/live-terminal")
}

/// Squirrel journal root.
pub fn journal_dir() -> PathBuf {
    PathBuf::from("/opt/membrane/squirrel-journal")
}

/// Antibodies file written by skunky-ingest.
pub fn antibodies_path() -> PathBuf {
    PathBuf::from("/opt/membrane/skunkbat-data/antibodies.json")
}

/// HUD public directory.
pub fn hud_public() -> PathBuf {
    PathBuf::from("/opt/ecoPrimals/hud/site/public")
}

/// Feed URLs.
pub static FEEDS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("chipmunk", "https://hud.primals.eco/chipmunk.json"),
        ("human", "https://hud.primals.eco/human.json"),
        ("dashboard", "https://hud.primals.eco/dashboard.json"),
    ])
});

pub const CLASSIFICATION_URL: &str = "https://detroit.primals.eco/classification.json";
pub const GRAPH_URL: &str = "https://detroit.primals.eco/graph.json";

/// Fleet profile → response action mapping.
pub static CLEAN_RESPONSES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("dev_reader", "redirect_clean"),
        ("code_crawler", "allow_clean"),
        ("proxy_dev", "redirect_clean"),
        ("proxy_reader", "redirect_clean"),
        ("high_conf_clean", "allow_clean"),
        ("slow_reader", "allow_clean"),
        ("mixed_behavior", "soft_redirect"),
        ("moderate_signal", "soft_redirect"),
        ("ignores_rejection", "license_hard"),
        ("masking_fleet", "license_hard"),
        ("rotating_impersonator", "license_hard"),
        ("rotating_ignorer", "license_hard"),
        ("heavy_masking_fleet", "license_hard"),
        ("stealth_scraper", "soft_redirect"),
        ("insufficient_signal", "soft_redirect"),
        ("single_probe", "ignore"),
        ("uniform_tool", "ignore"),
        ("unclear_reader", "soft_redirect"),
    ])
});

/// Membrane category → sites mapping.
pub static CATEGORY_SITES: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from([
        ("INVESTIGATION", vec!["detroit", "evidence"]),
        (
            "TOPOLOGY",
            vec!["lansing", "git", "relay", "beacon", "layer2", "sporegate", "admin"],
        ),
        ("SCIENCE", vec!["sporeprint", "hypothesis", "thesis"]),
        (
            "MEMBRANE",
            vec!["hud", "signal", "water", "neutron", "connect"],
        ),
        (
            "IMMUNE",
            vec![
                "tuebor",
                "clutch",
                "thymus",
                "receptor",
                "lysozyme",
                "epitope",
                "interferon",
                "antigen",
                "opsonize",
                "bloom",
                "cytokine",
                "complement",
                "antibody",
                "macrophage",
            ],
        ),
    ])
});

/// Build reverse map: site → category.
pub fn site_category_map() -> HashMap<String, String> {
    let mut m = HashMap::new();
    for (cat, sites) in CATEGORY_SITES.iter() {
        for site in sites {
            m.insert((*site).to_string(), (*cat).to_string());
        }
    }
    m
}

/// Depth ordering for epitope transitions.
pub const DEPTH_ORDER: &[&str] = &[
    "visitor",
    "observer",
    "witness",
    "reader",
    "coder",
    "investigator",
    "explorer",
    "scholar",
];

/// Build depth map: name → index.
pub fn depth_map() -> HashMap<String, usize> {
    DEPTH_ORDER
        .iter()
        .enumerate()
        .map(|(i, &name)| (name.to_string(), i))
        .collect()
}
