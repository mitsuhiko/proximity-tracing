use std::fmt;

use bytes::{BufMut, BytesMut};
use derive_more::{Display, Error};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use sha2::Sha256;

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

use crate::rpi::Rpi;
use crate::tkey::TracingKey;
use crate::utils::Base64DebugFmtHelper;

#[cfg(feature = "chrono")]
use crate::utils::day_number_for_timestamp;

/// A compact representation of contact numbers.
#[derive(Default, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct DailyTracingKey {
    bytes: [u8; 16],
}

impl fmt::Debug for DailyTracingKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("DailyTracingKey")
            .field(&Base64DebugFmtHelper(self))
            .finish()
    }
}

impl DailyTracingKey {
    /// Returns the daily tracing key for a day.
    pub fn for_day(tk: &TracingKey, day: u32) -> DailyTracingKey {
        let h = Hkdf::<Sha256>::new(None, tk.as_bytes());
        let mut info = BytesMut::from(&"CT-DTK"[..]);
        info.put_u32_le(day);
        let mut out = [0u8; 16];
        h.expand(&info, &mut out).unwrap();
        DailyTracingKey::from_bytes(&out[..]).unwrap()
    }

    /// Returns the daily tracing key for today.
    #[cfg(feature = "chrono")]
    pub fn for_today(tk: &TracingKey) -> DailyTracingKey {
        DailyTracingKey::for_timestamp(tk, &Utc::now())
    }

    /// Returns the daily tracing key for a timestamp.
    #[cfg(feature = "chrono")]
    pub fn for_timestamp(tk: &TracingKey, timestamp: &DateTime<Utc>) -> DailyTracingKey {
        DailyTracingKey::for_day(tk, day_number_for_timestamp(timestamp))
    }

    /// Creates a daily tracing key from raw bytes.
    pub fn from_bytes(b: &[u8]) -> Result<DailyTracingKey, InvalidDailyTracingKey> {
        if b.len() != 16 {
            return Err(InvalidDailyTracingKey);
        }
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(b);
        Ok(DailyTracingKey { bytes })
    }

    /// Returns the bytes behind the daily tracing key
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Generates all RPIs for a day.
    ///
    /// If you need the TINs too just use `.enumerate()`.
    pub fn iter_rpis(&self) -> impl Iterator<Item = Rpi> {
        let clone = *self;
        let mut tin = 0;
        std::iter::from_fn(move || {
            clone.get_rpi_for_tin(tin).map(|rv| {
                tin += 1;
                rv
            })
        })
    }

    /// Returns the RPI for a time interval number.
    ///
    /// If the time interval is out of range this returns `None`
    pub fn get_rpi_for_tin(&self, tin: u8) -> Option<Rpi> {
        if tin > 143 {
            return None;
        }

        let mut hmac = Hmac::<Sha256>::new_varkey(&self.as_bytes()).unwrap();
        let mut info = BytesMut::from(&"CT-RPI"[..]);
        info.put_u8(tin);
        hmac.input(&info[..]);
        let result = hmac.result();
        let bytes = &result.code()[..];
        Some(Rpi::from_bytes(&bytes[..16]).unwrap())
    }
}

/// Returned if a daily tracing key is invalid.
#[derive(Error, Display, Debug)]
#[display(fmt = "invalid daily tracing key")]
pub struct InvalidDailyTracingKey;

#[cfg(feature = "base64")]
mod base64_impl {
    use super::*;
    use std::{fmt, str};

    impl str::FromStr for DailyTracingKey {
        type Err = InvalidDailyTracingKey;

        fn from_str(value: &str) -> Result<DailyTracingKey, InvalidDailyTracingKey> {
            let mut bytes = [0u8; 16];
            if value.len() != 22 {
                return Err(InvalidDailyTracingKey);
            }
            base64_::decode_config_slice(value, base64_::URL_SAFE_NO_PAD, &mut bytes[..])
                .map_err(|_| InvalidDailyTracingKey)?;
            Ok(DailyTracingKey { bytes })
        }
    }

    #[cfg(feature = "base64")]
    impl fmt::Display for DailyTracingKey {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut buf = [0u8; 50];
            let len = base64_::encode_config_slice(self.bytes, base64_::URL_SAFE_NO_PAD, &mut buf);
            f.write_str(unsafe { std::str::from_utf8_unchecked(&buf[..len]) })
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;

    use serde_::de::Deserializer;
    use serde_::ser::Serializer;
    use serde_::{Deserialize, Serialize};

    impl Serialize for DailyTracingKey {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if serializer.is_human_readable() {
                serializer.serialize_str(&self.to_string())
            } else {
                serializer.serialize_bytes(self.as_bytes())
            }
        }
    }

    impl<'de> Deserialize<'de> for DailyTracingKey {
        fn deserialize<D>(deserializer: D) -> Result<DailyTracingKey, D::Error>
        where
            D: Deserializer<'de>,
        {
            use serde_::de::Error;
            if deserializer.is_human_readable() {
                let s = String::deserialize(deserializer).map_err(D::Error::custom)?;
                s.parse().map_err(D::Error::custom)
            } else {
                let buf = Vec::<u8>::deserialize(deserializer).map_err(D::Error::custom)?;
                DailyTracingKey::from_bytes(&buf).map_err(D::Error::custom)
            }
        }
    }
}
