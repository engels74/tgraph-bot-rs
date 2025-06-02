//! # TGraph Common
//!
//! Shared types, utilities, and common functionality for TGraph Bot.
//!
//! This crate provides the foundational types and utilities used across
//! all other crates in the TGraph Bot workspace.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod types;
pub mod utils;

#[cfg(any(test, feature = "testing"))]
pub mod test_utils;

pub use types::*;
pub use utils::*;
