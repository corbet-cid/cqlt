//! Forge-neutral quality policies and subordinate checking backends.
//! Metadata evaluation and prose normalization are pure. Prose execution uses
//! an explicit caller-supplied runner; cqlt owns its rules and Vale protocol.
#![forbid(unsafe_code)]

mod model;
mod presentation;
pub mod prose;

pub use model::*;
pub use presentation::{evaluate, valid_login, RULES, RULESET};
