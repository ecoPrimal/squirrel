//! scan-sweep-sort — unified classification pipeline + genome + worldclock.
//!
//! 6-stage pipeline:
//!   SCAN:      Discover entities (detroit classification, fleet antibodies, braids, traffic)
//!   SWEEP:     Compress, find structure, deduplicate
//!   SORT:      Classify self/non-self/partial/null + fleet + braids
//!   FOSSILIZE: Record what we learned from 8 perspectives
//!   EVOLVE:    Feed learning back — recommendations
//!   GENOME:    Measure the size of the work (when stale — tempo-relative, not wall clock)
//!
//! WORLDCLOCK: time is local to the locals.
//!   No daily cron. No fixed wall-clock times. The membrane's pulse decides
//!   when to refresh. Bounce into tempo, measure, bounce back out.
//!
//! Authorship: Kevin Mok / Tamison / ecoPrimal / attsi, ORCID 0009-0004-2141-0321
//! Credit: Sollimann for bonsai-bt

pub mod braid;
pub mod config;
pub mod fleet;
pub mod pipeline;
pub mod util;
pub mod worldclock;

/// Crate-level error type.
#[derive(Debug, thiserror::Error)]
pub enum SssError {
    /// I/O error.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// JSON error.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// HTTP error.
    #[error("http: {0}")]
    Http(#[from] ureq::Error),
}
