use regex::Regex;
use std::time::Duration as stdDuration;
use kube::core::Duration as k8sDuration;
use std::fmt;
use std::str::FromStr;
use once_cell::sync::Lazy;

/// gateway_api::Duration is a duration type where parsing and formatting obey
/// GEP-2257. It uses kube::core::Duration for the heavy lifting of parsing
/// but is based on std::time::Duration.
///
/// See https://gateway-api.sigs.k8s.io/geps/gep-2257 for the complete specification.
///
/// Per GEP-2257, when parsing a gateway_api::Duration from a string, the
/// string must match ^([0-9]{1,5}(h|m|s|ms)){1,4}$ and is otherwise parsed
/// the same way that Go's time.ParseDuration parses durations. When
/// formatting a gateway_api::Duration as a string, zero-valued durations must
/// always be formatted as "0s", and non-zero durations must be formatted to
/// with only one instance of each applicable unit, greatest unit first.
///
/// The rules above imply that gateway_api::Duration cannot represent
/// durations with sub-millisecond precision or times greater than
/// 99999h59m59s999ms. Since there's no meaningful way in Rust to allow string
/// formatting to fail, these conditions are checked instead when
/// instantiating gateway_api::Duration.
const GEP2257_PATTERN: &str = r"^([0-9]{1,5}(h|m|s|ms)){1,4}$";

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Duration(stdDuration);

// Maximum duration that can be represented by GEP-2257, in milliseconds.
const MAX_DURATION_MS: u128 = (((99999 * 3600) + (59 * 60) + 59) * 1_000) + 999;

/// is_valid checks if a duration is valid according to GEP-2257. If it's not,
/// it returns an error result explaining why the duration is not GEP-2257 valid.
fn is_valid(duration: stdDuration) -> Result<(), String> {
    // Check nanoseconds to see if we have sub-millisecond precision in
    // this duration.
    if duration.subsec_nanos() % 1_000_000 != 0 {
        return Err("Cannot express sub-millisecond precision in GEP-2257".to_string());
    }

    // Check the duration to see if it's greater than GEP-2257's maximum.
    if duration.as_millis() > MAX_DURATION_MS {
        return Err("Duration exceeds GEP-2257 maximum 99999h59m59s999ms".to_string());
    }

    Ok(())
}

/// Converting from std::time::Duration to gateway_api::Duration is allowed,
/// but we need to make sure that the incoming duration is valid according to
/// GEP-2257.
impl TryFrom<stdDuration> for Duration {
    type Error = String;

    fn try_from(duration: stdDuration) -> Result<Self, Self::Error> {
        // Check validity, and propagate any error if it's not.
        is_valid(duration)?;

        // It's valid, so we can safely convert it to a gateway_api::Duration.
        Ok(Duration(duration))
    }
}

/// Converting from k8s::time::Duration to gateway_api::Duration is allowed,
/// but we need to make sure that the incoming duration is valid according to
/// GEP-2257.
impl TryFrom<k8sDuration> for Duration {
    type Error = String;

    fn try_from(duration: k8sDuration) -> Result<Self, Self::Error> {
        // We can't rely on kube::core::Duration to check validity for
        // gateway_api::Duration, so first we need to make sure that our
        // k8sDuration is not negative...
        if duration.is_negative() {
            return Err("Duration cannot be negative".to_string());
        }

        // Once we know it's not negative, we can safely convert it to a
        // std::time::Duration (which will always succeed) and then check it
        // for validity as in TryFrom<stdDuration>.
        let stddur = stdDuration::from(duration);
        is_valid(stddur)?;
        Ok(Duration(stddur))
    }
}

impl Duration {
    /// gateway_api::Duration::new creates a new gateway_api::Duration from
    /// seconds and nanoseconds, but requires that the resulting Duration be
    /// valid according to GEP-2257.
    pub fn new(secs: u64, nanos: u32) -> Result<Self, String> {
        let stddur = stdDuration::new(secs, nanos);

        // Propagate errors if not valid, or unwrap the new Duration if all's
        // well.
        is_valid(stddur)?;
        Ok(Self(stddur))
    }

    /// gateway_api::Duration::from_secs creates a new gateway_api::Duration
    /// from seconds, but requires that the resulting Duration be valid
    /// according to GEP-2257.
    pub fn from_secs(secs: u64) -> Result<Self, String> {
        Self::new(secs, 0)
    }

    /// gateway_api::Duration::from_micros creates a new gateway_api::Duration
    /// from microseconds, but requires that the resulting Duration be valid
    /// according to GEP-2257.
    pub fn from_micros(micros: u64) -> Result<Self, String> {
        let sec = micros / 1_000_000;
        let ns = ((micros % 1_000_000) * 1_000) as u32;

        Self::new(sec, ns)
    }

    /// gateway_api::Duration::from_millis creates a new gateway_api::Duration
    /// from milliseconds, but requires that the resulting Duration be valid
    /// according to GEP-2257.
    pub fn from_millis(millis: u64) -> Result<Self, String> {
        let sec = millis / 1_000;
        let ns = ((millis % 1_000) * 1_000_000) as u32;

        Self::new(sec, ns)
    }

    /// The number of whole seconds in the duration
    pub fn as_secs(&self) -> u64 {
        self.0.as_secs()
    }

    /// The number of whole milliseconds in the duration
    pub fn as_millis(&self) -> u128 {
        self.0.as_millis()
    }

    /// The number of whole microseconds in the duration
    pub fn as_nanos(&self) -> u128 {
        self.0.as_nanos()
    }

    /// The number of nanoseconds in the duration that are
    /// not part of the whole seconds
    pub fn subsec_nanos(&self) -> u32 {
        self.0.subsec_nanos()
    }

    /// Indicates whether or not the duration is zero
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl FromStr for Duration {
    type Err = String;

    /// Parsing a gateway_api::Duration from a string requires that the string
    /// match the GEP-2257 duration format, and that the resulting duration be
    /// valid according to GEP-2257.
    fn from_str(duration_str: &str) -> Result<Self, Self::Err> {
        // GEP-2257 dictates that string values must match this regex and be
        // parsed the same way that Go's time.ParseDuration parses durations.
        //
        // This Lazy Regex::new should never ever fail, given that the regex
        // is a compile-time constant. But just in case.....
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(GEP2257_PATTERN).expect(
                format!(
                    r#"GEP2257 regex "{}" did not compile (this is a bug!)"#,
                    GEP2257_PATTERN
                )
                .as_str(),
            )
        });

        // If the string doesn't match the regex, it's invalid.
        if ! RE.is_match(duration_str) {
            return Err("Invalid duration format".to_string());
        }

        // We use kube::core::Duration to do the heavy lifting of parsing.
        match k8sDuration::from_str(duration_str) {
            // If the parse fails, return an error immediately...
            Err(err) => Err(err.to_string()),

            // ...otherwise, we need to try to turn the k8sDuration into a
            // gateway_api::Duration (which will check validity).
            Ok(kd) => Duration::try_from(kd),
        }
    }
}

impl fmt::Display for Duration {
    /// Formatting a gateway_api::Duration is defined only for valid
    /// durations, and must follow the GEP-2257 rules for formatting. These
    /// basically say that zero-valued durations must always be formatted as
    /// "0s", and that non-zero durations must be formatted with only one
    /// instance of each applicable unit, greatest unit first.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Short-circuit if the duration is zero, since "0s" is the special
        // case for a zero-valued duration.
        if self.is_zero() {
            return write!(f, "0s");
        }

        // Unfortunately, we can't rely on kube::core::Duration for
        // formatting, since it can happily hand back things like "5400s"
        // instead of "1h30m".
        //
        // So we'll do the formatting ourselves. Start by grabbing the
        // milliseconds part of the Duration (remember, the constructors make
        // sure that we don't have sub-millisecond precision)...
        let ms = self.subsec_nanos() / 1_000_000;

        // ...then after that, do the usual div & mod tree to take seconds and
        // get hours, minutes, and seconds from it.
        let mut secs = self.as_secs();

        let hours = secs / 3600;

        if hours > 0 {
            secs -= hours * 3600;
            write!(f, "{}h", hours)?;
        }

        let minutes = secs / 60;
        if minutes > 0 {
            secs -= minutes * 60;
            write!(f, "{}m", minutes)?;
        }

        if secs > 0 {
            write!(f, "{}s", secs)?;
        }

        if ms > 0 {
            write!(f, "{}ms", ms)?;
        }

        Ok(())
    }
}

impl fmt::Debug for Duration {
    /// gateway_api::Duration formats the same for debug as for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Yes, we format GEP-2257 Durations the same in debug and display.
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Test that the validation logic in `Duration`'s constructor
    /// method(s) correctly handles known-good durations
    fn test_gep2257_from_valid_duration() {
        let test_cases = vec![
            Duration::from_secs(0),
            Duration::from_secs(10),
            Duration::from_secs(1800),
            Duration::from_secs(3600),
            Duration::from_secs(9000),
            Duration::from_secs(5410),
            Duration::from_millis(500),
            Duration::from_millis(600),
            Duration::new(7200, 600_000_000),
            Duration::new(7200 + 1800, 600_000_000),
            Duration::new(7200 + 1800 + 10, 600_000_000),
            Duration::from_millis(MAX_DURATION_MS as u64),
        ];

        for (idx, duration) in test_cases.iter().enumerate() {
            assert!(duration.is_ok(), "{:?}: Duration {:?} should be OK", idx, duration);
        }
    }

    #[test]
    /// Test that the validation logic in `Duration`'s constructor
    /// method(s) correctly handles known-bad durations
    fn test_gep2257_from_invalid_duration() {
        let test_cases = vec![
            (Duration::from_micros(100), Err("Cannot express sub-millisecond precision in GEP-2257".to_string())),
            (Duration::from_secs(10000 * 86400), Err("Duration exceeds GEP-2257 maximum 99999h59m59s999ms".to_string())),
            (Duration::from_millis((MAX_DURATION_MS + 1) as u64), Err("Duration exceeds GEP-2257 maximum 99999h59m59s999ms".to_string())),
        ];

        for (idx, (duration, expected)) in test_cases.into_iter().enumerate() {
            assert_eq!(duration, expected, "{:?}: Duration {:?} should be an error", idx, duration);
        }
    }

    #[test]
    /// Test that the TryFrom implementation for k8sDuration correctly converts
    /// to gateway_api::Duration and validates the result.
    fn test_gep2257_from_valid_k8s_duration() {
        let test_cases = vec![
            (k8sDuration::from_str("0s").unwrap(), Duration::from_secs(0).unwrap()),
            (k8sDuration::from_str("1h").unwrap(), Duration::from_secs(3600).unwrap()),
            (k8sDuration::from_str("500ms").unwrap(), Duration::from_millis(500).unwrap()),
            (k8sDuration::from_str("2h600ms").unwrap(), Duration::new(7200, 600_000_000).unwrap()),
        ];

        for (idx, (k8s_duration, expected)) in test_cases.into_iter().enumerate() {
            let duration = Duration::try_from(k8s_duration);

            assert!(duration.as_ref().is_ok_and(|d| *d == expected),
                    "{:?}: Duration {:?} should be {:?}", idx, duration, expected);
        }
    }

    #[test]
    /// Test that the TryFrom implementation for k8sDuration correctly fails
    /// for kube::core::Durations that aren't valid GEP-2257 durations.
    fn test_gep2257_from_invalid_k8s_duration() {
        let test_cases: Vec<(k8sDuration, Result<Duration, String>)> = vec![
            (k8sDuration::from_str("100us").unwrap(),
                Err("Cannot express sub-millisecond precision in GEP-2257".to_string())),
            (k8sDuration::from_str("100000h").unwrap(),
                Err("Duration exceeds GEP-2257 maximum 99999h59m59s999ms".to_string())),
            (k8sDuration::from(stdDuration::from_millis((MAX_DURATION_MS + 1) as u64)),
                Err("Duration exceeds GEP-2257 maximum 99999h59m59s999ms".to_string())),
            (k8sDuration::from_str("-5s").unwrap(),
                Err("Duration cannot be negative".to_string())),
        ];

        for (idx, (k8s_duration, expected)) in test_cases.into_iter().enumerate() {
            assert_eq!(Duration::try_from(k8s_duration), expected, "{:?}: k8sDuration {:?} should be error {:?}", idx, k8s_duration, expected);
        }
    }

    #[test]
    fn test_gep2257_from_str() {
        // Test vectors are mostly taken directly from GEP-2257, but there are
        // some extras thrown in and it's not meaningful to test e.g. "0.5m"
        // in Rust.
        let test_cases = vec![
            ("0h", Duration::from_secs(0)),
            ("0s", Duration::from_secs(0)),
            ("0h0m0s", Duration::from_secs(0)),
            ("1h", Duration::from_secs(3600)),
            ("30m", Duration::from_secs(1800)),
            ("10s", Duration::from_secs(10)),
            ("500ms", Duration::from_millis(500)),
            ("2h30m", Duration::from_secs(9000)),
            ("150m", Duration::from_secs(9000)),
            ("7230s", Duration::from_secs(7230)),
            ("1h30m10s", Duration::from_secs(5410)),
            ("10s30m1h", Duration::from_secs(5410)),
            ("100ms200ms300ms", Duration::from_millis(600)),
            ("100ms200ms300ms", Duration::from_millis(600)),
            ("99999h59m59s999ms", Duration::from_millis(MAX_DURATION_MS as u64)),
            ("1d", Err("Invalid duration format".to_string())),
            ("1", Err("Invalid duration format".to_string())),
            ("1m1", Err("Invalid duration format".to_string())),
            ("1h30m10s20ms50h", Err("Invalid duration format".to_string())),
            ("999999h", Err("Invalid duration format".to_string())),
            ("1.5h", Err("Invalid duration format".to_string())),
            ("-15m", Err("Invalid duration format".to_string())),
            ("99999h59m59s1000ms", Err("Duration exceeds GEP-2257 maximum 99999h59m59s999ms".to_string())),
        ];

        for (idx, (duration_str, expected)) in test_cases.into_iter().enumerate() {
            assert_eq!(Duration::from_str(duration_str), expected, "{:?}: Duration {:?} should be {:?}", idx, duration_str, expected);
        }
    }

    #[test]
    fn test_gep2257_format() {
        // Formatting should always succeed for valid durations, and we've
        // covered invalid durations in the constructor and parse tests.
        let test_cases = vec![
            (Duration::from_secs(0), "0s".to_string()),
            (Duration::from_secs(3600), "1h".to_string()),
            (Duration::from_secs(1800), "30m".to_string()),
            (Duration::from_secs(10), "10s".to_string()),
            (Duration::from_millis(500), "500ms".to_string()),
            (Duration::from_secs(9000), "2h30m".to_string()),
            (Duration::from_secs(5410), "1h30m10s".to_string()),
            (Duration::from_millis(600), "600ms".to_string()),
            (Duration::new(7200, 600_000_000), "2h600ms".to_string()),
            (Duration::new(7200 + 1800, 600_000_000), "2h30m600ms".to_string()),
            (Duration::new(7200 + 1800 + 10, 600_000_000), "2h30m10s600ms".to_string()),
        ];

        for (idx, (duration, expected)) in test_cases.into_iter().enumerate() {
            assert!(duration.as_ref().is_ok_and(|d| format!("{}", d) == expected),
                    "{:?}: Duration {:?} should be {:?}", idx, duration, expected);
        }
    }
}
