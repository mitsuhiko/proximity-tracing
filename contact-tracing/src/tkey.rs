use std::fmt;
use std::str;

use derive_more::{Display, Error};
use rand::{thread_rng, RngCore};
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

/// A compact representation of contact numbers.
#[derive(Default, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct TracingKey {
    bytes: [u8; 32],
}

impl fmt::Debug for TracingKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("TracingKey")
            .field(&self.to_string())
            .finish()
    }
}

impl TracingKey {
    /// Returns a new unique tracing key.
    pub fn unique() -> TracingKey {
        let mut bytes = [0u8; 32];
        let mut rng = thread_rng();
        rng.fill_bytes(&mut bytes[..]);
        TracingKey::from_bytes(&bytes[..]).unwrap()
    }

    /// loads a tracing key from raw bytes.
    pub fn from_bytes(b: &[u8]) -> Result<TracingKey, InvalidTracingKey> {
        if b.len() != 32 {
            return Err(InvalidTracingKey);
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(b);
        Ok(TracingKey { bytes })
    }

    /// Returns the bytes behind the tracing key.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Raised if a tracing key is invalid.
#[derive(Error, Display, Debug)]
#[display(fmt = "invalid tracing key")]
pub struct InvalidTracingKey;

impl str::FromStr for TracingKey {
    type Err = InvalidTracingKey;

    fn from_str(value: &str) -> Result<TracingKey, InvalidTracingKey> {
        let mut bytes = [0u8; 32];
        if value.len() != 43 {
            return Err(InvalidTracingKey);
        }
        base64::decode_config_slice(value, base64::URL_SAFE_NO_PAD, &mut bytes[..])
            .map_err(|_| InvalidTracingKey)?;
        Ok(TracingKey { bytes })
    }
}

impl fmt::Display for TracingKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = [0u8; 50];
        let len = base64::encode_config_slice(self.bytes, base64::URL_SAFE_NO_PAD, &mut buf);
        f.write_str(unsafe { std::str::from_utf8_unchecked(&buf[..len]) })
    }
}

impl Serialize for TracingKey {
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

impl<'de> Deserialize<'de> for TracingKey {
    fn deserialize<D>(deserializer: D) -> Result<TracingKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer).map_err(D::Error::custom)?;
            s.parse().map_err(D::Error::custom)
        } else {
            let buf = Vec::<u8>::deserialize(deserializer).map_err(D::Error::custom)?;
            TracingKey::from_bytes(&buf).map_err(D::Error::custom)
        }
    }
}
