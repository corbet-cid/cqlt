//! Forge-neutral quality policies and subordinate checking backends.
//! Metadata evaluation, prose normalization and semantic replay are pure.
//! Callers own bounded Vale execution and any authorized Jev HTTP transport.
#![forbid(unsafe_code)]

mod model;
mod presentation;
pub mod prose;
pub mod semantic;

pub use model::*;
pub use presentation::{evaluate, valid_login, RULES, RULESET};
