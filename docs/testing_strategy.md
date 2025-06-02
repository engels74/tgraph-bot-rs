# Testing Strategy for TGraph Bot Rust Edition

## Overview

This document outlines the comprehensive testing strategy for the TGraph Bot Rust Edition project, following Test-Driven Development (TDD) principles and Rust best practices.

## Test Organization

### Directory Structure

```
crates/
├── tgraph-common/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── types.rs
│   │   ├── utils.rs
│   │   └── test_utils.rs      # Shared test utilities
│   └── tests/
│       └── integration_test.rs # Integration tests
├── tgraph-config/
│   ├── src/
│   │   └── lib.rs
│   └── tests/
│       └── integration_test.rs
└── [other crates follow same pattern]
```

### Test Types

#### 1. Unit Tests

- **Location**: `src/` files with `#[cfg(test)]` modules
- **Purpose**: Test individual functions and methods in isolation
- **Naming**: `test_[function_name]_[scenario]`
- **Example**:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_channel_id_display() {
          let id = ChannelId(123456789);
          assert_eq!(format!("{}", id), "123456789");
      }
  }
  ```

#### 2. Integration Tests

- **Location**: `tests/` directory
- **Purpose**: Test interactions between modules and crates
- **Naming**: `integration_test.rs` or specific feature tests
- **Example**: Testing Discord command processing end-to-end

#### 3. Property-Based Tests

- **Tool**: `proptest` crate
- **Purpose**: Test properties that should hold for all valid inputs
- **Location**: Both unit and integration tests
- **Example**:
  ```rust
  proptest! {
      #[test]
      fn test_channel_id_roundtrip(id in channel_id_strategy()) {
          let serialized = format!("{}", id);
          let parsed: u64 = serialized.parse().unwrap();
          assert_eq!(id.0, parsed);
      }
  }
  ```

#### 4. Benchmark Tests

- **Tool**: `criterion` crate
- **Purpose**: Performance regression testing
- **Location**: `benches/` directory (to be added)
- **Focus**: Graph generation, data processing, Discord API calls

## Test Utilities (`tgraph-common::test_utils`)

### Core Utilities

#### Logging

```rust
use tgraph_common::test_utils::init_test_logging;

#[test]
fn my_test() {
    init_test_logging(); // Initialize once per test run
    // Test code here
}
```

#### Async Testing

```rust
use tgraph_common::test_utils::create_test_runtime;

#[test]
fn test_async_function() {
    let runtime = create_test_runtime();
    let result = runtime.block_on(async {
        // Async test code
    });
}
```

#### Fixtures

- **Time**: `mock_timestamp(year, month, day, hour, min, sec)`
- **Files**: `create_temp_dir()`, `create_temp_file()`
- **Discord**: `discord_fixtures::test_channel_id()`
- **Graphs**: `graph_fixtures::generate_time_series(count, start_date)`
- **Config**: `config_fixtures::minimal_config_yaml()`

### Property Testing Strategies

#### Discord IDs

```rust
use tgraph_common::test_utils::property_testing::*;

proptest! {
    #[test]
    fn test_user_id_valid(id in user_id_strategy()) {
        assert!(id.0 >= 100000000000000000);
    }
}
```

## Test-Driven Development Workflow

### 1. Red Phase (Write Failing Test)

```rust
#[test]
fn test_config_validation() {
    let config = Config::from_yaml("invalid yaml");
    assert!(config.is_err());
}
```

### 2. Green Phase (Make Test Pass)

```rust
impl Config {
    pub fn from_yaml(yaml: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(yaml)
            .map_err(ConfigError::ParseError)
    }
}
```

### 3. Refactor Phase

- Improve code quality while keeping tests green
- Add more comprehensive tests
- Optimize performance

## Continuous Integration

### GitHub Actions Workflow

The CI pipeline includes:

1. **Test Matrix**: Stable, beta, nightly Rust versions
2. **Feature Matrix**: `--no-default-features`, `--all-features`
3. **Code Quality**: `cargo fmt`, `cargo clippy`
4. **Documentation**: `cargo doc`
5. **Coverage**: `cargo tarpaulin` with 80% threshold
6. **Security**: `cargo audit`
7. **Cross-Platform**: Ubuntu, Windows, macOS

### Code Coverage

- **Tool**: `cargo-tarpaulin`
- **Threshold**: 80% minimum
- **Configuration**: `tarpaulin.toml`
- **Exclusions**: Test files, generated code
- **Reports**: XML (Codecov), HTML (local), JSON

## Testing Best Practices

### 1. Test Naming

- Use descriptive names: `test_[what]_[when]_[expected]`
- Group related tests in modules
- Use `#[should_panic]` for error cases

### 2. Test Independence

- Each test should be independent
- Use fixtures for common setup
- Clean up resources (temp files, etc.)

### 3. Async Testing

- Use `#[tokio::test]` for async tests
- Test timeout scenarios
- Verify cancellation behavior

### 4. Error Testing

- Test both success and failure paths
- Verify error messages and types
- Test error propagation

### 5. Performance Testing

- Benchmark critical paths
- Set performance budgets
- Test with realistic data sizes

## Mock and Stub Strategy

### External Dependencies

- **Discord API**: Mock with `mockall`
- **Tautulli API**: Mock HTTP responses
- **File System**: Use `tempfile` for isolation
- **Time**: Use fixed timestamps in tests

### Example Mock

```rust
use mockall::predicate::*;

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        TautulliClient {}

        #[async_trait]
        impl TautulliApi for TautulliClient {
            async fn get_activity(&self) -> Result<Activity, ApiError>;
        }
    }

    #[tokio::test]
    async fn test_graph_generation() {
        let mut mock_client = MockTautulliClient::new();
        mock_client
            .expect_get_activity()
            .returning(|| Ok(Activity::default()));

        // Test with mock
    }
}
```

## Test Data Management

### Fixtures

- Store test data in `test_utils` module
- Use builders for complex objects
- Provide both minimal and comprehensive examples

### Snapshot Testing

- Use `insta` for graph output verification
- Store expected outputs as snapshots
- Review changes carefully

## Performance Testing

### Benchmarks

- Graph generation performance
- Configuration loading speed
- Discord API response times
- Memory usage patterns

### Profiling

- Use `cargo flamegraph` for CPU profiling
- Monitor memory allocations
- Test under load conditions

## Documentation Testing

### Doc Tests

````rust
/// Formats a timestamp for display
///
/// # Examples
///
/// ```
/// use tgraph_common::format_timestamp;
/// use chrono::{TimeZone, Utc};
///
/// let timestamp = Utc.ymd(2024, 1, 1).and_hms(12, 0, 0);
/// assert_eq!(format_timestamp(timestamp), "2024-01-01 12:00:00 UTC");
/// ```
pub fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
````

## Test Maintenance

### Regular Tasks

- Update test data as features evolve
- Review and update property test strategies
- Maintain benchmark baselines
- Update mock expectations

### Metrics to Track

- Test coverage percentage
- Test execution time
- Number of flaky tests
- Performance regression alerts

## Conclusion

This testing strategy ensures high code quality, reliability, and maintainability for the TGraph Bot Rust Edition. By following TDD principles and leveraging Rust's powerful testing ecosystem, we can build a robust and performant Discord bot.
