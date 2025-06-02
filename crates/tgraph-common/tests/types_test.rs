//! TDD tests for Core Type Definitions in tgraph-common crate.
//!
//! This test suite covers:
//! - Newtype wrappers implementing expected traits (Display, Debug, Serialize, Deserialize)
//! - Type conversions being safe and validated
//! - Domain types enforcing invariants

use std::collections::HashMap;
use tgraph_common::types::*;

#[cfg(test)]
mod newtype_trait_tests {
    use super::*;

    #[test]
    fn test_channel_id_implements_expected_traits() {
        let channel_id = ChannelId(123456789);

        // Test Debug
        let debug_str = format!("{:?}", channel_id);
        assert_eq!(debug_str, "ChannelId(123456789)");

        // Test Display
        let display_str = format!("{}", channel_id);
        assert_eq!(display_str, "123456789");

        // Test Clone and Copy
        let cloned_id = channel_id.clone();
        let copied_id = channel_id;
        assert_eq!(channel_id, cloned_id);
        assert_eq!(channel_id, copied_id);

        // Test PartialEq and Eq
        assert_eq!(channel_id, ChannelId(123456789));
        assert_ne!(channel_id, ChannelId(987654321));

        // Test Hash - can be used in HashMap
        let mut map = HashMap::new();
        map.insert(channel_id, "test_channel");
        assert_eq!(map.get(&channel_id), Some(&"test_channel"));
    }

    #[test]
    fn test_channel_id_serialization() {
        let channel_id = ChannelId(123456789);

        // Test Serialize
        let serialized = serde_json::to_string(&channel_id).unwrap();
        assert_eq!(serialized, "123456789");

        // Test Deserialize
        let deserialized: ChannelId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, channel_id);
    }

    #[test]
    fn test_user_id_implements_expected_traits() {
        let user_id = UserId(987654321);

        // Test Debug
        let debug_str = format!("{:?}", user_id);
        assert_eq!(debug_str, "UserId(987654321)");

        // Test Display
        let display_str = format!("{}", user_id);
        assert_eq!(display_str, "987654321");

        // Test serialization roundtrip
        let serialized = serde_json::to_string(&user_id).unwrap();
        let deserialized: UserId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, user_id);
    }

    #[test]
    fn test_tautulli_user_id_implements_expected_traits() {
        let tautulli_id = TautulliUserId(555);

        // Test Debug
        let debug_str = format!("{:?}", tautulli_id);
        assert_eq!(debug_str, "TautulliUserId(555)");

        // Test Display
        let display_str = format!("{}", tautulli_id);
        assert_eq!(display_str, "555");

        // Test serialization roundtrip
        let serialized = serde_json::to_string(&tautulli_id).unwrap();
        let deserialized: TautulliUserId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, tautulli_id);
    }

    #[test]
    fn test_media_item_id_implements_expected_traits() {
        let media_id = MediaItemId(789);

        // Test Debug
        let debug_str = format!("{:?}", media_id);
        assert_eq!(debug_str, "MediaItemId(789)");

        // Test Display
        let display_str = format!("{}", media_id);
        assert_eq!(display_str, "789");

        // Test serialization roundtrip
        let serialized = serde_json::to_string(&media_id).unwrap();
        let deserialized: MediaItemId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, media_id);
    }

    #[test]
    fn test_session_id_implements_expected_traits() {
        let session_id = SessionId::new("abc123def".to_string());

        // Test Debug
        let debug_str = format!("{:?}", session_id);
        assert!(debug_str.contains("SessionId"));
        assert!(debug_str.contains("abc123def"));

        // Test Display
        let display_str = format!("{}", session_id);
        assert_eq!(display_str, "abc123def");

        // Test serialization roundtrip
        let serialized = serde_json::to_string(&session_id).unwrap();
        let deserialized: SessionId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, session_id);
    }

    #[test]
    fn test_timestamp_implements_expected_traits() {
        let timestamp = Timestamp::now();

        // Test Debug
        let debug_str = format!("{:?}", timestamp);
        assert!(debug_str.contains("Timestamp"));

        // Test Display
        let display_str = format!("{}", timestamp);
        assert!(!display_str.is_empty());

        // Test serialization roundtrip
        let serialized = serde_json::to_string(&timestamp).unwrap();
        let deserialized: Timestamp = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, timestamp);

        // Test Clone
        let cloned = timestamp.clone();
        assert_eq!(timestamp, cloned);
    }
}

#[cfg(test)]
mod type_conversion_tests {
    use super::*;

    #[test]
    fn test_safe_id_conversions() {
        // Test From conversions for IDs
        let raw_id: u64 = 123456789;
        let channel_id = ChannelId::from(raw_id);
        assert_eq!(channel_id.0, raw_id);

        let user_id = UserId::from(raw_id);
        assert_eq!(user_id.0, raw_id);

        let tautulli_id = TautulliUserId::from(raw_id);
        assert_eq!(tautulli_id.0, raw_id);

        let media_id = MediaItemId::from(raw_id);
        assert_eq!(media_id.0, raw_id);
    }

    #[test]
    fn test_into_conversions() {
        let channel_id = ChannelId(123456789);
        let raw: u64 = channel_id.into();
        assert_eq!(raw, 123456789);
    }

    #[test]
    fn test_session_id_validation() {
        // Valid session IDs
        assert!(SessionId::try_new("abc123".to_string()).is_ok());
        assert!(SessionId::try_new("session_123_abc".to_string()).is_ok());
        assert!(SessionId::try_new("a".repeat(32)).is_ok());

        // Invalid session IDs
        assert!(SessionId::try_new("".to_string()).is_err());
        assert!(SessionId::try_new(" ".to_string()).is_err());
        assert!(SessionId::try_new("session with spaces".to_string()).is_err());
        assert!(SessionId::try_new("session\twith\ttabs".to_string()).is_err());
        assert!(SessionId::try_new("session\nwith\nnewlines".to_string()).is_err());
        assert!(SessionId::try_new("a".repeat(256)).is_err()); // Too long
    }

    #[test]
    fn test_timestamp_conversions() {
        use chrono::{DateTime, Utc};

        // Test from DateTime<Utc>
        let now = Utc::now();
        let timestamp = Timestamp::from(now);

        // Test back to DateTime<Utc>
        let back_to_datetime: DateTime<Utc> = timestamp.into();
        assert_eq!(now.timestamp(), back_to_datetime.timestamp());

        // Test from timestamp
        let unix_timestamp = 1640995200i64; // 2022-01-01 00:00:00 UTC
        let timestamp = Timestamp::from_timestamp(unix_timestamp);
        assert_eq!(timestamp.timestamp(), unix_timestamp);
    }

    #[test]
    fn test_play_count_validation() {
        // Valid play counts
        assert!(PlayCount::try_new(0).is_ok());
        assert!(PlayCount::try_new(1).is_ok());
        assert!(PlayCount::try_new(999999).is_ok());

        // PlayCount should allow zero as it's a valid count
        let zero_count = PlayCount::try_new(0).unwrap();
        assert_eq!(zero_count.value(), 0);
    }

    #[test]
    fn test_duration_validation() {
        // Valid durations
        assert!(Duration::try_new(0).is_ok());
        assert!(Duration::try_new(60).is_ok());
        assert!(Duration::try_new(3600).is_ok());

        // Duration should allow zero for edge cases
        let zero_duration = Duration::try_new(0).unwrap();
        assert_eq!(zero_duration.seconds(), 0);
    }
}

#[cfg(test)]
mod domain_invariant_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_session_id_invariants() {
        // Session IDs must be non-empty and contain valid characters
        let valid_session = SessionId::try_new("valid_session_123".to_string()).unwrap();
        assert!(!valid_session.as_str().is_empty());
        assert!(!valid_session.as_str().contains(' '));
        assert!(!valid_session.as_str().contains('\t'));
        assert!(!valid_session.as_str().contains('\n'));

        // Session IDs have reasonable length limits
        assert!(valid_session.as_str().len() <= 255);
    }

    #[test]
    fn test_timestamp_invariants() {
        let timestamp = Timestamp::now();

        // Timestamps should be within reasonable bounds
        let year = timestamp.year();
        assert!(year >= 2020 && year <= 2100);

        // Timestamps should be comparable
        let earlier = Timestamp::from_timestamp(timestamp.timestamp() - 3600);
        assert!(earlier < timestamp);

        let later = Timestamp::from_timestamp(timestamp.timestamp() + 3600);
        assert!(timestamp < later);
    }

    #[test]
    fn test_play_count_invariants() {
        let count = PlayCount::try_new(42).unwrap();

        // Play counts should never be negative (enforced by u32 type)
        assert!(count.value() >= 0);

        // Play counts should support arithmetic operations safely
        let doubled = count.add(count.value());
        assert_eq!(doubled.value(), 84);
    }

    #[test]
    fn test_duration_invariants() {
        let duration = Duration::try_new(3661).unwrap(); // 1 hour, 1 minute, 1 second

        // Duration should never be negative (enforced by u32 type)
        assert!(duration.seconds() >= 0);

        // Duration should provide convenient accessors
        assert_eq!(duration.hours(), 1);
        assert_eq!(duration.minutes(), 1);
        assert_eq!(duration.remaining_seconds(), 1);
    }

    #[test]
    fn test_media_type_invariants() {
        // Media types should have consistent string representations
        assert_eq!(MediaType::Movie.as_str(), "movie");
        assert_eq!(MediaType::TvShow.as_str(), "show");
        assert_eq!(MediaType::Music.as_str(), "track");

        // Media types should parse from strings consistently
        assert_eq!(MediaType::from_str("movie").unwrap(), MediaType::Movie);
        assert_eq!(MediaType::from_str("show").unwrap(), MediaType::TvShow);
        assert_eq!(MediaType::from_str("track").unwrap(), MediaType::Music);

        // Invalid media types should return errors
        assert!(MediaType::from_str("invalid").is_err());
        assert!(MediaType::from_str("").is_err());
    }

    #[test]
    fn test_graph_type_invariants() {
        // Graph types should have consistent string representations
        assert_eq!(GraphType::DailyPlayCount.as_str(), "daily_play_count");
        assert_eq!(
            GraphType::PlayCountByDayOfWeek.as_str(),
            "play_count_by_dayofweek"
        );
        assert_eq!(
            GraphType::PlayCountByHourOfDay.as_str(),
            "play_count_by_hourofday"
        );
        assert_eq!(GraphType::Top10Platforms.as_str(), "top_10_platforms");
        assert_eq!(GraphType::Top10Users.as_str(), "top_10_users");
        assert_eq!(GraphType::PlayCountByMonth.as_str(), "play_count_by_month");

        // Graph types should implement Display consistently
        assert_eq!(format!("{}", GraphType::DailyPlayCount), "daily_play_count");
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_tgraph_error_display() {
        let config_error = TGraphError::Config("Invalid API key".to_string());
        assert_eq!(
            format!("{}", config_error),
            "Configuration error: Invalid API key"
        );

        let discord_error = TGraphError::Discord("Failed to send message".to_string());
        assert_eq!(
            format!("{}", discord_error),
            "Discord API error: Failed to send message"
        );

        let tautulli_error = TGraphError::Tautulli("API timeout".to_string());
        assert_eq!(
            format!("{}", tautulli_error),
            "Tautulli API error: API timeout"
        );

        let graph_error = TGraphError::Graph("Rendering failed".to_string());
        assert_eq!(
            format!("{}", graph_error),
            "Graph generation error: Rendering failed"
        );

        let serialization_error = TGraphError::Serialization("JSON parse error".to_string());
        assert_eq!(
            format!("{}", serialization_error),
            "Serialization error: JSON parse error"
        );
    }

    #[test]
    fn test_tgraph_error_debug() {
        let error = TGraphError::Config("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_result_type_alias() {
        // Test that our Result type alias works correctly
        fn test_function() -> Result<String> {
            Ok("success".to_string())
        }

        let result = test_function();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_io_error_conversion() {
        use std::io;

        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let tgraph_error: TGraphError = io_error.into();

        match tgraph_error {
            TGraphError::Io(_) => {} // Expected
            _ => panic!("Expected Io error variant"),
        }
    }
}

#[cfg(test)]
mod property_based_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_channel_id_roundtrip_serialization(id in any::<u64>()) {
            let channel_id = ChannelId(id);
            let serialized = serde_json::to_string(&channel_id).unwrap();
            let deserialized: ChannelId = serde_json::from_str(&serialized).unwrap();
            assert_eq!(channel_id, deserialized);
        }

        #[test]
        fn test_user_id_roundtrip_serialization(id in any::<u64>()) {
            let user_id = UserId(id);
            let serialized = serde_json::to_string(&user_id).unwrap();
            let deserialized: UserId = serde_json::from_str(&serialized).unwrap();
            assert_eq!(user_id, deserialized);
        }

        #[test]
        fn test_tautulli_user_id_roundtrip_serialization(id in any::<u64>()) {
            let tautulli_id = TautulliUserId(id);
            let serialized = serde_json::to_string(&tautulli_id).unwrap();
            let deserialized: TautulliUserId = serde_json::from_str(&serialized).unwrap();
            assert_eq!(tautulli_id, deserialized);
        }

        #[test]
        fn test_play_count_invariants(count in any::<u32>()) {
            let play_count = PlayCount::try_new(count).unwrap();
            assert_eq!(play_count.value(), count);
            assert!(play_count.value() >= 0);
        }

        #[test]
        fn test_duration_invariants(seconds in any::<u32>()) {
            let duration = Duration::try_new(seconds).unwrap();
            assert_eq!(duration.seconds(), seconds);
            assert!(duration.seconds() >= 0);

            // Verify time component calculations
            let hours = duration.hours();
            let minutes = duration.minutes();
            let remaining_seconds = duration.remaining_seconds();

            assert_eq!(hours * 3600 + minutes * 60 + remaining_seconds, seconds);
        }

        #[test]
        fn test_valid_session_id_characters(
            s in r"[a-zA-Z0-9_-]{1,255}"
        ) {
            // Property: any string matching this pattern should create a valid SessionId
            let session_id = SessionId::try_new(s.clone());
            assert!(session_id.is_ok());
            assert_eq!(session_id.unwrap().as_str(), s);
        }
    }
}
