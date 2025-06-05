//! Common utilities and types for TGraph Telegram bot

pub mod error;
pub mod logging;
pub mod macros;
pub mod tautulli;
pub mod types;
pub mod utils;

// Re-export commonly used types
pub use error::{Result, TGraphError};
pub use logging::{
    init_default_logging, init_dev_logging, init_dual_logging, init_logging, init_prod_logging,
    LoggingConfig,
};
pub use tautulli::{
    ActivityResponse, ClientMetrics, HistoryEntry, HistoryResponse, Library, LibrariesResponse,
    ServerInfoResponse, Session, TautulliClient, TautulliConfig, TautulliResponse,
    TautulliResponseData, User, UsersResponse,
};
pub use types::*; 