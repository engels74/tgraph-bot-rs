//! # TGraph I18n
//!
//! Internationalization support using Fluent localization system for TGraph Bot.
//!
//! This crate provides compile-time validated translations with support for
//! complex pluralization and context-aware messages.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod loader;
pub mod messages;

pub use loader::*;
pub use messages::*;
