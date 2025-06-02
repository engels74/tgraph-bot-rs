//! # TGraph Graphs
//!
//! High-performance graph generation and rendering for Tautulli data visualization.
//!
//! This crate handles all graph rendering logic with efficient data processing
//! and native Rust rendering using plotters.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod data_fetcher;
pub mod manager;
pub mod traits;
pub mod user_manager;
pub mod utils;

// Graph implementations (placeholders for now)
pub mod daily_play_count;
pub mod play_count_by_dayofweek;
pub mod play_count_by_hourofday;
pub mod play_count_by_month;
pub mod top_10_platforms;
pub mod top_10_users;

pub use data_fetcher::*;
pub use manager::*;
pub use traits::*;
pub use user_manager::*;
pub use utils::*;
