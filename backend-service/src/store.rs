use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fs;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use bytes::{Buf, BufMut, BytesMut};
use chrono::{DateTime, Utc};
use crc::crc32;

use contact_tracing::{day_number_for_timestamp, DailyTracingKey};

const DAYS_WINDOW: u32 = 21;

/// Abstracts over an append only file of CCNs
pub struct DailyTracingKeyStore {
    path: PathBuf,
    buckets: RwLock<BTreeMap<u32, HashSet<DailyTracingKey>>>,
}

impl fmt::Debug for DailyTracingKeyStore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DailyTracingKeyStore")
            .field("path", &self.path)
            .finish()
    }
}

impl DailyTracingKeyStore {
    /// Opens a ccn store
    pub fn open<P: AsRef<Path>>(p: P) -> Result<DailyTracingKeyStore, io::Error> {
        let path = p.as_ref().to_path_buf();
        fs::create_dir_all(&path)?;
        Ok(DailyTracingKeyStore {
            path,
            buckets: RwLock::new(BTreeMap::new()),
        })
    }

    /// Returns the current bucket.
    pub fn current_day(&self) -> u32 {
        day_number_for_timestamp(&Utc::now())
    }

    /// Ensure bucket is loaded from disk.
    fn ensure_day_loaded(&self, bucket: u32) -> Result<bool, io::Error> {
        // we only upsert so if the bucket was already loaded, we don't
        // need to do anything
        if self.buckets.read().unwrap().contains_key(&bucket) {
            return Ok(false);
        }

        let mut buckets = self.buckets.write().unwrap();
        let path = self.path.join(&format!("_{}.bucket", bucket));

        let mut set = HashSet::new();
        if let Ok(mut f) = fs::File::open(path).map(BufReader::new) {
            loop {
                let mut buf = [0u8; 20];
                match f.read(&mut buf)? {
                    0 => break,
                    x if x != buf.len() => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "something went very wrong",
                        ));
                    }
                    _ => {}
                }
                let key = DailyTracingKey::from_bytes(&buf[..16]).unwrap();
                let checksum = crc32::checksum_ieee(key.as_bytes());
                if (&buf[16..]).get_u32_le() != checksum {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "bad checksum, corrupted file",
                    ));
                }
                set.insert(key);
            }
        }

        buckets.insert(bucket, set);

        Ok(true)
    }

    /// Returns all buckets after a certain timestamp
    pub fn fetch_buckets(
        &self,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<DailyTracingKey>, io::Error> {
        let mut rv = vec![];
        let bucket_start = day_number_for_timestamp(&timestamp);
        let bucket_end = self.current_day();

        match bucket_end.checked_sub(bucket_start) {
            None => return Ok(vec![]),
            Some(diff) if diff > 24 * DAYS_WINDOW => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "reading too far into the past",
                ))
            }
            _ => {}
        }

        for bucket in bucket_start..=bucket_end {
            self.ensure_day_loaded(bucket)?;
            if let Some(set) = self.buckets.read().unwrap().get(&bucket) {
                rv.extend(set);
            }
        }

        Ok(rv)
    }

    /// Checks if a tracing key is already known.
    pub fn has_daily_tracing_key(&self, key: DailyTracingKey) -> Result<bool, io::Error> {
        let now = self.current_day();
        for bucket in (now - DAYS_WINDOW)..now {
            self.ensure_day_loaded(bucket)?;
            if let Some(set) = self.buckets.read().unwrap().get(&bucket) {
                if set.contains(&key) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Adds a tracing key at the current timestamp.
    pub fn add_daily_tracing_key(
        &self,
        day_number: u32,
        key: DailyTracingKey,
    ) -> Result<bool, io::Error> {
        // check if this ccn has already been seen in the last 21 days
        if self.has_daily_tracing_key(key)? {
            return Ok(false);
        }

        let path = self.path.join(&format!("_{}.bucket", day_number));
        let mut buckets = self.buckets.write().unwrap();
        let mut file = BufWriter::new(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?,
        );
        let mut msg = BytesMut::new();
        msg.put_slice(key.as_bytes());
        msg.put_u32_le(crc32::checksum_ieee(key.as_bytes()));
        file.write(&msg)?;
        buckets
            .entry(day_number)
            .or_insert_with(Default::default)
            .insert(key);
        Ok(true)
    }
}
