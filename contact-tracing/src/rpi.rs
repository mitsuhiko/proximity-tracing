use std::fmt;

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};
use derive_more::{Display, Error};

use crate::utils::Base64DebugFmtHelper;

#[cfg(feature = "chrono")]
use crate::utils::tin_for_timestamp;

/// A Rolling Proximity Identifier.
#[derive(Default, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Rpi {
    bytes: [u8; 16],
}

impl fmt::Debug for Rpi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Rpi")
            .field(&Base64DebugFmtHelper(self))
            .finish()
    }
}

impl Rpi {
    /// Returns the RPI for a timestamp directly from a tracing key.
    #[cfg(feature = "chrono")]
    pub fn for_timestamp(tk: &crate::tkey::TracingKey, timestamp: &DateTime<Utc>) -> Rpi {
        let dtkey = crate::dtkey::DailyTracingKey::for_timestamp(tk, timestamp);
        dtkey.get_rpi_for_tin(tin_for_timestamp(timestamp)).unwrap()
    }

    /// Returns the RPI that is for the current time interval.
    #[cfg(feature = "chrono")]
    pub fn for_now(tk: &crate::tkey::TracingKey) -> Rpi {
        Rpi::for_timestamp(tk, &Utc::now())
    }

    /// Creates a RPI from raw bytes.
    pub fn from_bytes(b: &[u8]) -> Result<Rpi, InvalidRpi> {
        if b.len() != 16 {
            return Err(InvalidRpi);
        }
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(b);
        Ok(Rpi { bytes })
    }

    /// Returns the bytes behind the RPI
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Raised if a RPI is invalid.
#[derive(Error, Display, Debug)]
#[display(fmt = "invalid rpi")]
pub struct InvalidRpi;

#[cfg(feature = "base64")]
mod base64_impl {
    use super::*;
    use std::{fmt, str};

    impl str::FromStr for Rpi {
        type Err = InvalidRpi;

        fn from_str(value: &str) -> Result<Rpi, InvalidRpi> {
            let mut bytes = [0u8; 16];
            if value.len() != 22 {
                return Err(InvalidRpi);
            }
            base64_::decode_config_slice(value, base64_::URL_SAFE_NO_PAD, &mut bytes[..])
                .map_err(|_| InvalidRpi)?;
            Ok(Rpi { bytes })
        }
    }

    impl fmt::Display for Rpi {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut buf = [0u8; 50];
            let len = base64_::encode_config_slice(self.bytes, base64_::URL_SAFE_NO_PAD, &mut buf);
            f.write_str(unsafe { std::str::from_utf8_unchecked(&buf[..len]) })
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    pub use super::*;

    use serde_::de::Deserializer;
    use serde_::ser::Serializer;
    use serde_::{Deserialize, Serialize};

    impl Serialize for Rpi {
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

    impl<'de> Deserialize<'de> for Rpi {
        fn deserialize<D>(deserializer: D) -> Result<Rpi, D::Error>
        where
            D: Deserializer<'de>,
        {
            use serde_::de::Error;
            if deserializer.is_human_readable() {
                let s = String::deserialize(deserializer).map_err(D::Error::custom)?;
                s.parse().map_err(D::Error::custom)
            } else {
                let buf = Vec::<u8>::deserialize(deserializer).map_err(D::Error::custom)?;
                Rpi::from_bytes(&buf).map_err(D::Error::custom)
            }
        }
    }
}
