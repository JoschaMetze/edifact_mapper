//! DST timezone helpers for German MESZ/MEZ validation.
//!
//! EU DST rule: MESZ (summer time) starts the last Sunday of March at 01:00 UTC,
//! and ends the last Sunday of October at 01:00 UTC.

use super::evaluator::ConditionResult;

/// Returns `True` if the given CCYYMMDDHHMM value falls within German summer time (MESZ),
/// `False` if it falls in winter time (MEZ), or `Unknown` if the input is invalid/too short.
///
/// Only the first 12 characters are used, so a 15-char value with timezone suffix also works.
pub fn is_mesz_utc(dtm_value: &str) -> ConditionResult {
    let s = dtm_value.trim();
    if s.len() < 12 {
        return ConditionResult::Unknown;
    }
    let s = &s[..12];

    let year: u32 = match s[0..4].parse() {
        Ok(v) => v,
        Err(_) => return ConditionResult::Unknown,
    };
    let month: u32 = match s[4..6].parse() {
        Ok(v) => v,
        Err(_) => return ConditionResult::Unknown,
    };
    let day: u32 = match s[6..8].parse() {
        Ok(v) => v,
        Err(_) => return ConditionResult::Unknown,
    };
    let hour: u32 = match s[8..10].parse() {
        Ok(v) => v,
        Err(_) => return ConditionResult::Unknown,
    };
    let minute: u32 = match s[10..12].parse() {
        Ok(v) => v,
        Err(_) => return ConditionResult::Unknown,
    };

    // Basic validation
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 {
        return ConditionResult::Unknown;
    }

    // Compute transition dates for this year
    let march_last_sunday = last_sunday_of_month(year, 3);
    let october_last_sunday = last_sunday_of_month(year, 10);

    // MESZ transition: last Sunday of March at 01:00 UTC
    // MEZ transition: last Sunday of October at 01:00 UTC
    // MESZ is active when: (month, day, hour, minute) >= March transition AND < October transition

    let dt = (month, day, hour, minute);
    let mesz_start = (3u32, march_last_sunday, 1u32, 0u32);
    let mesz_end = (10u32, october_last_sunday, 1u32, 0u32);

    let in_mesz = dt >= mesz_start && dt < mesz_end;

    ConditionResult::from(in_mesz)
}

/// Returns `True` if the given CCYYMMDDHHMM value falls within German winter time (MEZ).
///
/// This is the complement of [`is_mesz_utc`].
pub fn is_mez_utc(dtm_value: &str) -> ConditionResult {
    match is_mesz_utc(dtm_value) {
        ConditionResult::True => ConditionResult::False,
        ConditionResult::False => ConditionResult::True,
        ConditionResult::Unknown => ConditionResult::Unknown,
    }
}

/// Compute the day-of-month of the last Sunday of the given month and year.
///
/// Uses Tomohiko Sakamoto's algorithm for day-of-week calculation.
fn last_sunday_of_month(year: u32, month: u32) -> u32 {
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => unreachable!("invalid month"),
    };

    // Find the day of the week for the last day of the month (0=Sunday..6=Saturday)
    let dow = day_of_week(year, month, days_in_month);

    // Subtract to get to Sunday
    days_in_month - dow
}

/// Returns the day of the week for a given date (0=Sunday, 1=Monday, ..., 6=Saturday).
///
/// Uses Tomohiko Sakamoto's algorithm.
fn day_of_week(mut year: u32, month: u32, day: u32) -> u32 {
    const T: [u32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    if month < 3 {
        year -= 1;
    }
    (year + year / 4 - year / 100 + year / 400 + T[(month - 1) as usize] + day) % 7
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_mesz_date() {
        // 2026-07-15 12:00 UTC — clearly summer time
        assert_eq!(is_mesz_utc("202607151200"), ConditionResult::True);
    }

    #[test]
    fn test_known_mez_date() {
        // 2026-01-15 12:00 UTC — clearly winter time
        assert_eq!(is_mesz_utc("202601151200"), ConditionResult::False);
    }

    #[test]
    fn test_march_transition_2026() {
        // Last Sunday of March 2026 = March 29
        // Before 01:00 UTC → still MEZ
        assert_eq!(is_mesz_utc("202603290059"), ConditionResult::False);
        // At 01:00 UTC → MESZ starts
        assert_eq!(is_mesz_utc("202603290100"), ConditionResult::True);
    }

    #[test]
    fn test_october_transition_2026() {
        // Last Sunday of October 2026 = October 25
        // Before 01:00 UTC → still MESZ
        assert_eq!(is_mesz_utc("202610250059"), ConditionResult::True);
        // At 01:00 UTC → MEZ starts
        assert_eq!(is_mesz_utc("202610250100"), ConditionResult::False);
    }

    #[test]
    fn test_short_input_returns_unknown() {
        assert_eq!(is_mesz_utc("2026"), ConditionResult::Unknown);
        assert_eq!(is_mesz_utc(""), ConditionResult::Unknown);
        assert_eq!(is_mesz_utc("20260715"), ConditionResult::Unknown);
    }

    #[test]
    fn test_invalid_input_returns_unknown() {
        assert_eq!(is_mesz_utc("abcdefghijkl"), ConditionResult::Unknown);
        // Invalid month
        assert_eq!(is_mesz_utc("202613151200"), ConditionResult::Unknown);
        // Invalid day
        assert_eq!(is_mesz_utc("202601321200"), ConditionResult::Unknown);
    }

    #[test]
    fn test_is_mez_complements_is_mesz() {
        // Summer → MESZ=True, MEZ=False
        assert_eq!(is_mez_utc("202607151200"), ConditionResult::False);
        // Winter → MESZ=False, MEZ=True
        assert_eq!(is_mez_utc("202601151200"), ConditionResult::True);
        // Invalid → both Unknown
        assert_eq!(is_mez_utc("short"), ConditionResult::Unknown);
    }

    #[test]
    fn test_value_with_timezone_suffix() {
        // 15-char value with suffix — only first 12 used
        assert_eq!(is_mesz_utc("202607151200UTC"), ConditionResult::True);
        assert_eq!(is_mesz_utc("202601151200303"), ConditionResult::False);
    }

    #[test]
    fn test_last_sunday_of_march_2026() {
        // March 2026: March 1 is Sunday? Let's verify.
        // March 31 is Tuesday (dow=2), so last Sunday = 31 - 2 = 29
        assert_eq!(last_sunday_of_month(2026, 3), 29);
    }

    #[test]
    fn test_last_sunday_of_october_2026() {
        // October 31, 2026 is Saturday (dow=6), so last Sunday = 31 - 6 = 25
        assert_eq!(last_sunday_of_month(2026, 10), 25);
    }

    #[test]
    fn test_different_years() {
        // 2025: last Sunday of March = March 30, last Sunday of October = October 26
        assert_eq!(last_sunday_of_month(2025, 3), 30);
        assert_eq!(last_sunday_of_month(2025, 10), 26);

        // 2024: last Sunday of March = March 31, last Sunday of October = October 27
        assert_eq!(last_sunday_of_month(2024, 3), 31);
        assert_eq!(last_sunday_of_month(2024, 10), 27);
    }
}
