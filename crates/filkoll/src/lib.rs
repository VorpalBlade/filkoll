//! This is a bin+lib for technical code organisation reasons, do not use as a
//! library. There is no guarantee of stability or API compatibility.

// Export this for clap_mangen / clap_completions
mod arch;
mod interner;
mod types;

#[doc(hidden)]
pub mod cli;
#[doc(hidden)]
pub mod lookup;
#[doc(hidden)]
pub mod update;

/// Path to the cache directory
pub(crate) const CACHE_PATH: &str = "/var/cache/filkoll";
