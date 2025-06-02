//! # TGraph Config
//!
//! Type-safe configuration management with hot-reloading for TGraph Bot.
//!
//! This crate provides configuration loading, validation, and caching
//! with support for hot-reloading and atomic updates.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod cache;
pub mod defaults;
pub mod loader;
pub mod schema;
pub mod validator;

pub use cache::*;
pub use defaults::*;
pub use loader::*;
pub use schema::*;
pub use validator::*;
