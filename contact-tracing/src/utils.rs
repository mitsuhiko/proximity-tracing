use std::fmt;

#[cfg(feature = "chrono")]
use chrono::{Date, DateTime, Utc};

pub(crate) struct Base64DebugFmtHelper<'a, T>(pub &'a T);

#[cfg(feature = "base64")]
impl<'a, T: fmt::Display> fmt::Debug for Base64DebugFmtHelper<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

#[cfg(not(feature = "base64"))]
impl<'a, T: fmt::Debug> fmt::Debug for Base64DebugFmtHelper<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.0, f)
    }
}

/// Returns the day number for a timestamp.
#[cfg(feature = "chrono")]
pub fn day_number_for_timestamp(ts: &DateTime<Utc>) -> u32 {
    (ts.timestamp() / (60 * 60 * 24)) as u32
}

/// Returns the TIN for a timestamp in a day.
///
/// If the TIN does not exist (because it's for a different day) then it
/// returns `None`.
#[cfg(feature = "chrono")]
pub fn tin_for_timestamp_checked(ts: &DateTime<Utc>, day: Date<Utc>) -> Option<u8> {
    let now = ts.timestamp();
    let start_of_day = day.and_hms(0, 0, 0).timestamp();
    let tin = (now - start_of_day) / (60 * 10);
    if tin >= 0 && tin <= 143 {
        Some(tin as u8)
    } else {
        None
    }
}

/// Returns the TIN for a timestamp.
///
/// This does not validate the day.
#[cfg(feature = "chrono")]
pub fn tin_for_timestamp(ts: &DateTime<Utc>) -> u8 {
    tin_for_timestamp_checked(ts, ts.date()).unwrap()
}
