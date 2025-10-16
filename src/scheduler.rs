use anyhow::{Context, Result};
use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;
use tokio::time::Duration;

/// Parse cron expression and calculate next run time
pub fn parse_cron_interval(cron_expr: &str) -> Result<Duration> {
    let schedule = Schedule::from_str(cron_expr).context("Failed to parse cron expression")?;

    let next = schedule
        .upcoming(Utc)
        .next()
        .context("No upcoming scheduled time found")?;

    let now = Utc::now();
    let mut duration = next.signed_duration_since(now);

    if duration.num_seconds() <= 0 {
        // If the next time is in the past, get the one after that
        let next_schedule = schedule
            .upcoming(Utc)
            .nth(1)
            .context("No second upcoming scheduled time found")?;
        duration = next_schedule.signed_duration_since(now);
    }
    Ok(Duration::from_secs(duration.num_seconds() as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cron_every_minute() {
        // Every minute: "0 * * * * *" (with seconds field)
        let result = parse_cron_interval("0 * * * * *");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than 60 seconds (next minute)
        assert!(duration.as_secs() <= 60);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_parse_cron_every_hour() {
        // Every hour at minute 0: "0 0 * * * *" (with seconds field)
        let result = parse_cron_interval("0 0 * * * *");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than or equal to 3600 seconds (1 hour)
        assert!(duration.as_secs() <= 3600);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_parse_cron_every_day_at_midnight() {
        // Every day at midnight: "0 0 0 * * *" (with seconds field)
        let result = parse_cron_interval("0 0 0 * * *");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than or equal to 86400 seconds (24 hours)
        assert!(duration.as_secs() <= 86400);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_parse_cron_every_day_at_noon() {
        // Every day at noon: "0 0 12 * * *" (with seconds field)
        let result = parse_cron_interval("0 0 12 * * *");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than or equal to 86400 seconds (24 hours)
        assert!(duration.as_secs() <= 86400);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_parse_cron_every_monday_at_9am() {
        // Every Monday at 9:00 AM: "0 0 9 * * Mon" (with seconds field)
        let result = parse_cron_interval("0 0 9 * * Mon");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than or equal to 7 days
        assert!(duration.as_secs() <= 7 * 86400);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_parse_cron_every_15_minutes() {
        // Every 15 minutes: "0 */15 * * * *" (with seconds field)
        let result = parse_cron_interval("0 */15 * * * *");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than or equal to 15 minutes (900 seconds)
        assert!(duration.as_secs() <= 900);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_parse_cron_every_30_minutes() {
        // Every 30 minutes: "0 */30 * * * *" (with seconds field)
        let result = parse_cron_interval("0 */30 * * * *");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than or equal to 30 minutes (1800 seconds)
        assert!(duration.as_secs() <= 1800);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_parse_cron_invalid_expression() {
        // Invalid cron expression
        let result = parse_cron_interval("invalid cron");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to parse cron expression"));
    }

    #[test]
    fn test_parse_cron_invalid_format() {
        // Too few fields
        let result = parse_cron_interval("* * *");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cron_invalid_values() {
        // Invalid minute value (60 is not valid, max is 59)
        let result = parse_cron_interval("60 * * * *");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cron_empty_string() {
        // Empty string
        let result = parse_cron_interval("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cron_with_seconds() {
        // Cron with seconds (6 fields): "0 */5 * * * *" - every 5 minutes
        let result = parse_cron_interval("0 */5 * * * *");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than or equal to 5 minutes (300 seconds)
        assert!(duration.as_secs() <= 300);
    }

    #[test]
    fn test_parse_cron_specific_time() {
        // Specific time: "0 30 14 * * *" - every day at 14:30
        let result = parse_cron_interval("0 30 14 * * *");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than 24 hours
        assert!(duration.as_secs() <= 86400);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_parse_cron_first_day_of_month() {
        // First day of every month at midnight: "0 0 0 1 * *"
        let result = parse_cron_interval("0 0 0 1 * *");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than 31 days
        assert!(duration.as_secs() <= 31 * 86400);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_parse_cron_weekdays_only() {
        // Monday to Friday at 9 AM: "0 0 9 * * Mon-Fri"
        let result = parse_cron_interval("0 0 9 * * Mon-Fri");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than 7 days
        assert!(duration.as_secs() <= 7 * 86400);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_parse_cron_multiple_hours() {
        // Every day at 6 AM, noon, and 6 PM: "0 0 6,12,18 * * *"
        let result = parse_cron_interval("0 0 6,12,18 * * *");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than 24 hours
        assert!(duration.as_secs() <= 86400);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_parse_cron_range() {
        // Every minute from 9 AM to 5 PM: "0 * 9-17 * * *"
        let result = parse_cron_interval("0 * 9-17 * * *");
        assert!(result.is_ok());
        let duration = result.unwrap();
        // Should be less than 24 hours
        assert!(duration.as_secs() <= 86400);
        assert!(duration.as_secs() > 0);
    }

    #[test]
    fn test_duration_always_positive() {
        // Test multiple valid cron expressions to ensure duration is always positive
        let expressions = vec![
            "0 * * * * *",    // Every minute
            "0 0 * * * *",    // Every hour
            "0 0 0 * * *",    // Every day at midnight
            "0 */5 * * * *",  // Every 5 minutes
            "0 0 12 * * Mon", // Every Monday at noon
        ];

        for expr in expressions {
            let result = parse_cron_interval(expr);
            assert!(result.is_ok(), "Failed to parse: {}", expr);
            let duration = result.unwrap();
            assert!(
                duration.as_secs() > 0,
                "Duration should be positive for: {}",
                expr
            );
        }
    }
}
