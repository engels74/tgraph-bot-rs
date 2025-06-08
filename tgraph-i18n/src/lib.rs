//! Internationalization support for TGraph Telegram bot
//!
//! This crate provides comprehensive internationalization support using the Fluent
//! localization system. It includes:
//!
//! - Locale management and detection
//! - Resource loading and caching
//! - FluentBundle management
//! - Message formatting with context-aware translations
//! - Fallback mechanisms for missing translations
//!
//! # Example
//!
//! ```rust
//! use tgraph_i18n::{I18nManager, Locale};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut manager = I18nManager::new(Locale::English, "locales")?;
//! manager.load_locale(&Locale::Spanish)?;
//!
//! let message = manager.get_message("hello", &Locale::Spanish, None)?;
//! println!("{}", message);
//! # Ok(())
//! # }
//! ```

pub mod bundle;
pub mod context;
pub mod error;
pub mod locale;
pub mod manager;
pub mod pluralization;
pub mod resource;

pub use bundle::{fluent_args, BundleManager};
pub use context::{Gender, TranslationContext};
pub use error::{I18nError, I18nResult};
pub use locale::Locale;
pub use manager::I18nManager;
pub use pluralization::PluralizationHelper;
pub use resource::ResourceManager;

// Re-export commonly used Fluent types
pub use fluent::{FluentArgs, FluentValue};