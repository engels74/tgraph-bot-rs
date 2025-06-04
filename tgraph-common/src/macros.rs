//! Convenience macros for error handling and propagation

/// Equivalent to `anyhow::bail!` but for `TGraphError`
/// 
/// This macro allows early returns with custom error messages.
/// 
/// # Examples
/// 
/// ```rust
/// use tgraph_common::bail;
/// use tgraph_common::Result;
/// 
/// fn check_value(value: i32) -> Result<()> {
///     if value < 0 {
///         bail!("Value cannot be negative: {}", value);
///     }
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($crate::TGraphError::new($msg))
    };
    ($err:expr $(,)?) => {
        return Err($crate::TGraphError::new($err))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::TGraphError::new(format!($fmt, $($arg)*)))
    };
}

/// Equivalent to `anyhow::ensure!` but for `TGraphError`
/// 
/// This macro checks a condition and returns an error if it's false.
/// 
/// # Examples
/// 
/// ```rust
/// use tgraph_common::ensure;
/// use tgraph_common::Result;
/// 
/// fn validate_positive(value: i32) -> Result<()> {
///     ensure!(value > 0, "Value must be positive, got: {}", value);
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return Err($crate::TGraphError::new($msg));
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return Err($crate::TGraphError::new($err));
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return Err($crate::TGraphError::new(format!($fmt, $($arg)*)));
        }
    };
}

/// Add context to an error while preserving the error chain
/// 
/// This macro is useful for adding context as errors propagate up the call stack.
/// 
/// # Examples
/// 
/// ```rust
/// use tgraph_common::{with_context, Result};
/// 
/// fn read_config() -> Result<String> {
///     std::fs::read_to_string("config.toml")
///         .map_err(|e| with_context!(e, "Failed to read configuration file"))
/// }
/// ```
#[macro_export]
macro_rules! with_context {
    ($err:expr, $msg:literal $(,)?) => {
        $crate::TGraphError::with_source($msg, $err)
    };
    ($err:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::TGraphError::with_source(format!($fmt, $($arg)*), $err)
    };
}

/// Create a specialized error type for a specific domain
/// 
/// This macro helps create domain-specific error constructors.
/// 
/// # Examples
/// 
/// ```rust
/// use tgraph_common::error_constructor;
/// use tgraph_common::TGraphError;
/// 
/// // Create a constructor for Discord errors
/// error_constructor!(discord_error, Discord);
/// 
/// // Usage
/// let err = discord_error("Failed to send message");
/// ```
#[macro_export]
macro_rules! error_constructor {
    ($name:ident, $variant:ident) => {
        pub fn $name(msg: impl Into<String>) -> $crate::TGraphError {
            match stringify!($variant) {
                "Config" => $crate::TGraphError::config(msg),
                "Network" => $crate::TGraphError::network(msg),
                "Discord" => $crate::TGraphError::discord(msg),
                "Tautulli" => $crate::TGraphError::tautulli(msg),
                "Database" => $crate::TGraphError::database(msg),
                "Graph" => $crate::TGraphError::graph(msg),
                "Localization" => $crate::TGraphError::localization(msg),
                "Auth" => $crate::TGraphError::auth(msg),
                "Validation" => $crate::TGraphError::validation(msg),
                _ => $crate::TGraphError::new(msg),
            }
        }
    };
}

/// Log and return an error
/// 
/// This macro logs an error at the specified level and then returns it.
/// 
/// # Examples
/// 
/// ```rust
/// use tgraph_common::{log_and_bail, Result};
/// 
/// fn risky_operation(should_fail: bool) -> Result<()> {
///     if should_fail {
///         log_and_bail!(error, "Operation failed due to condition");
///     }
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! log_and_bail {
    ($level:ident, $msg:literal $(,)?) => {{
        let error = $crate::TGraphError::new($msg);
        tracing::$level!("{}", error);
        return Err(error);
    }};
    ($level:ident, $fmt:expr, $($arg:tt)*) => {{
        let message = format!($fmt, $($arg)*);
        let error = $crate::TGraphError::new(&message);
        tracing::$level!("{}", error);
        return Err(error);
    }};
}

/// Create a Result with context for a common operation
/// 
/// This macro wraps common operations with appropriate error context.
/// 
/// # Examples
/// 
/// ```rust
/// use tgraph_common::{result_with_context, Result};
/// 
/// fn load_file(path: &str) -> Result<String> {
///     result_with_context!(
///         std::fs::read_to_string(path),
///         "Failed to read file"
///     )
/// }
/// ```
#[macro_export]
macro_rules! result_with_context {
    ($expr:expr, $msg:literal $(,)?) => {
        $expr.map_err(|e| $crate::with_context!(e, $msg))
    };
    ($expr:expr, $fmt:expr, $($arg:tt)*) => {
        $expr.map_err(|e| $crate::with_context!(e, format!($fmt, $($arg)*)))
    };
}

#[cfg(test)]
mod tests {
    use crate::Result;

    #[test]
    fn test_bail_macro() {
        fn test_function() -> Result<()> {
            bail!("Test error message");
        }

        let result = test_function();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Test error message"));
    }

    #[test]
    fn test_ensure_macro() {
        fn test_function(value: i32) -> Result<()> {
            ensure!(value > 0, "Value must be positive: {}", value);
            Ok(())
        }

        // Test success case
        assert!(test_function(5).is_ok());

        // Test failure case
        let result = test_function(-1);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Value must be positive"));
    }

    #[test]
    fn test_with_context_macro() {
        use std::io;

        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let contextual_error = with_context!(io_error, "Failed to read config file");

        assert!(contextual_error.to_string().contains("Failed to read config file"));
    }

    #[test] 
    fn test_result_with_context_macro() {
        use std::fmt;
        
        #[derive(Debug)]
        struct TestError(String);
        
        impl fmt::Display for TestError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        
        impl std::error::Error for TestError {}

        fn might_fail(should_fail: bool) -> std::result::Result<String, TestError> {
            if should_fail {
                Err(TestError("original error".to_string()))
            } else {
                Ok("success".to_string())
            }
        }

        fn wrapper() -> Result<String> {
            result_with_context!(
                might_fail(true),
                "Operation failed with context"
            )
        }

        let result = wrapper();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Operation failed with context"));
    }
} 