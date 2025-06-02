//! # TGraph Commands
//!
//! Discord command implementations using Poise framework for TGraph Bot.
//!
//! This crate provides all Discord slash commands with type-safe parameter
//! parsing, permission checks, and built-in cooldowns using Poise.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod about;
pub mod commands;
pub mod config;
pub mod framework;
pub mod my_stats;
pub mod update_graphs;
pub mod uptime;

pub use commands::*;
pub use framework::*;
