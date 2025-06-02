//! # TGraph Bot
//!
//! High-performance Discord bot for automated Tautulli graph generation and posting.
//!
//! This is the main binary crate that orchestrates the entire application lifecycle
//! using the Poise framework for Discord interactions.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod bot;
pub mod error;

pub use bot::*;
pub use error::*;
