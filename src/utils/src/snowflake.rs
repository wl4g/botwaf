// SPDX-License-Identifier: GNU GENERAL PUBLIC LICENSE Version 3
//
// Copyleft (c) 2024 James Wong. This file is part of James Wong.
// is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// James Wong is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with James Wong.  If not, see <https://www.gnu.org/licenses/>.
//
// IMPORTANT: Any software that fully or partially contains or uses materials
// covered by this license must also be released under the GNU GPL license.
// This includes modifications and derived works.

use std::sync::atomic::{ AtomicI64, Ordering };
use std::sync::Mutex;
use std::time::{ SystemTime, UNIX_EPOCH };
use std::env;
use mac_address::get_mac_address;
use lazy_static::lazy_static;

pub struct SnowflakeIdGenerator {
    epoch: i64,
    datacenter_id: i64,
    worker_id: i64,
    sequence: AtomicI64,
}

lazy_static! {
    static ref SNOWFLAKE_ID_GENERATOR: Mutex<SnowflakeIdGenerator> = Mutex::new(
        SnowflakeIdGenerator::new().expect("Failed to initialize SnowflakeIdGenerator")
    );
}

// Notice: This is an important boundary value in languages ​​like JavaScript because it is the maximum safe integer value.
pub static JS_SAFE_INT_MAX: i64 = (2i64).pow(53) - 1;

impl SnowflakeIdGenerator {
    pub fn default_next() -> i64 {
        SNOWFLAKE_ID_GENERATOR.lock().unwrap().next()
    }

    pub fn default_next_jssafe() -> i64 {
        SNOWFLAKE_ID_GENERATOR.lock().unwrap().next_jssafe()
    }

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let epoch = 1609459200000; // 2021-01-01 00:00:00 UTC
        let datacenter_id = Self::get_datacenter_id()?;
        let worker_id = Self::get_worker_id()?;

        Ok(Self {
            epoch,
            datacenter_id,
            worker_id,
            sequence: AtomicI64::new(0),
        })
    }

    pub fn next(&self) -> i64 {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;

        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst) & 4095;

        // Notice: This generated id is not safe because the length will overflow the int memory in JavaScript.
        ((timestamp - self.epoch) << 22) |
            (self.datacenter_id << 17) |
            (self.worker_id << 12) |
            sequence
    }

    pub fn next_jssafe(&self) -> i64 {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;

        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst) & 0x3f;

        // Notice: This generated id is safe because the length is within the range of JavaScript int.
        ((timestamp - self.epoch) << 14) |
            (self.datacenter_id << 13) |
            (self.worker_id << 9) |
            sequence
    }

    fn get_datacenter_id() -> Result<i64, Box<dyn std::error::Error>> {
        // Try to get datacenter ID from environment variable
        if let Ok(dc_id) = env::var("DATACENTER_ID") {
            return Ok(dc_id.parse()?);
        }

        // If environment variable doesn't exist, consider other methods
        // For example, getting it from Kubernetes labels or annotations
        // Here, we simply return a default value
        Ok(1)
    }

    fn get_worker_id() -> Result<i64, Box<dyn std::error::Error>> {
        // Try to get MAC address
        if let Some(mac) = get_mac_address()? {
            // Use the last 6 bits of MAC address as worker_id
            let mac_bytes = mac.bytes();
            let worker_id = (((mac_bytes[4] as i64) << 8) | (mac_bytes[5] as i64)) & 0x1f;
            return Ok(worker_id);
        }

        // If unable to get MAC address, consider other methods
        // For example, extracting from Pod name
        if let Ok(hostname) = env::var("HOSTNAME") {
            if let Some(last_char) = hostname.chars().last() {
                if let Some(digit) = last_char.to_digit(10) {
                    return Ok(digit as i64);
                }
            }
        }

        // If all above methods fail, return a default value
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snowflake_id_generator() {
        let id1 = SnowflakeIdGenerator::default_next();
        let id2 = SnowflakeIdGenerator::default_next();
        assert!(id1 < id2);
    }

    #[test]
    fn test_snowflake_id_generator_jssafe() {
        let id1 = SnowflakeIdGenerator::default_next_jssafe();
        let id2 = SnowflakeIdGenerator::default_next_jssafe();
        assert!(id1 < id2);
        assert!(id1 >= 0 && id2 <= JS_SAFE_INT_MAX);
    }
}
