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

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Error, Ok};
use async_trait::async_trait;
use moka::future::Cache;
use moka::policy::EvictionPolicy;
use regex::Regex;

use crate::config::config::MemoryProperties;

use super::ICache;

pub struct StringMemoryCache {
    cache: Arc<Cache<String, String>>,
}

impl StringMemoryCache {
    pub fn new(config: &MemoryProperties) -> Self {
        let mut builder = Cache::builder();
        if let Some(initial_capacity) = config.initial_capacity {
            builder = builder.initial_capacity(initial_capacity as usize);
        }
        if let Some(max_capacity) = config.max_capacity {
            builder = builder.max_capacity(max_capacity);
        }
        if let Some(ttl) = config.ttl {
            builder = builder.time_to_live(Duration::from_millis(ttl));
        }
        if let Some(eviction_policy) = &config.eviction_policy {
            match eviction_policy.to_uppercase().as_str() {
                "LRU" => {
                    builder = builder.eviction_policy(EvictionPolicy::lru());
                }
                "LFU" | "TINY_LFU" => {
                    builder = builder.eviction_policy(EvictionPolicy::tiny_lfu());
                }
                _ => {
                    builder = builder.eviction_policy(EvictionPolicy::default());
                }
            }
        }
        StringMemoryCache {
            cache: Arc::new(builder.build()),
        }
    }

    fn serialize_hash(hash: &HashMap<String, String>) -> String {
        serde_json::to_string(hash).unwrap_or_default()
    }

    fn deserialize_hash(s: &str) -> HashMap<String, String> {
        serde_json::from_str(s).unwrap_or_default()
    }
}

#[async_trait]
impl ICache<String> for StringMemoryCache {
    async fn get(&self, key: String) -> Result<Option<String>, Error> {
        Ok(self.cache.get(&key).await)
    }

    /// Sets the given key to the specified value.
    ///
    /// # Note
    /// The `milliseconds` parameter is deprecated and will be ignored.
    #[allow(unused_variables)]
    async fn set(&self, key: String, value: String, milliseconds: Option<i32>) -> Result<bool, Error> {
        self.cache.insert(key.clone(), value).await;
        tracing::info!("Inserted to key: {}, expire: {:?}ms", key, milliseconds);
        Ok(true)
    }

    async fn set_nx(&self, key: String, value: Option<String>) -> Result<bool, Error> {
        if let Some(v) = value {
            match self.cache.contains_key(&key) {
                false => {
                    self.cache.insert(key.clone(), v).await;
                    return Ok(true);
                }
                true => {
                    return Ok(false);
                }
            }
        } else {
            Ok(false)
        }
    }

    async fn keys(&self, pattern_str: String) -> Result<Vec<String>, Error> {
        let pattern = Arc::new(Regex::new(&pattern_str).unwrap());
        let keys = self
            .cache
            .into_iter()
            .map(|(k, _)| k.deref().to_string())
            .filter(|k| pattern.clone().is_match(k))
            .collect::<Vec<_>>();
        Ok(keys)
    }

    async fn hget(&self, key: String, field: Option<String>) -> Result<Option<String>, Error> {
        if let Some(hash_str) = self.cache.get(&key).await {
            let hash = Self::deserialize_hash(&hash_str);
            match field {
                Some(f) => Ok(hash.get(&f).map(|v| v.to_string())),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    async fn hget_all(&self, key: String) -> Result<Option<HashMap<String, String>>, Error> {
        if let Some(hash_str) = self.cache.get(&key).await {
            let hash = Self::deserialize_hash(&hash_str);
            Ok(Some(hash))
        } else {
            Ok(None)
        }
    }

    async fn hset(&self, key: String, field_values: Option<Vec<(String, String)>>) -> Result<bool, Error> {
        if let Some(fv) = field_values {
            let mut hash = if let Some(hash_str) = self.cache.get(&key).await {
                Self::deserialize_hash(&hash_str)
            } else {
                HashMap::new()
            };
            for (field, value) in fv {
                hash.insert(field, value); // override put
            }
            self.cache.insert(key, Self::serialize_hash(&hash)).await;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn hset_nx(&self, key: String, field: String, value: String) -> Result<bool, Error> {
        let mut hash = if let Some(hash_str) = self.cache.get(&key).await {
            Self::deserialize_hash(&hash_str)
        } else {
            HashMap::new()
        };
        if !hash.contains_key(&field) {
            hash.insert(field, value);
            self.cache.insert(key, Self::serialize_hash(&hash)).await;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn hkeys(&self, key: String) -> Result<Vec<String>, Error> {
        if let Some(hash_str) = self.cache.get(&key).await {
            let hash = Self::deserialize_hash(&hash_str);
            let fields: Vec<String> = hash.into_iter().map(|(f, _)| f).collect();
            Ok(fields)
        } else {
            Ok(vec![])
        }
    }

    #[allow(unused)]
    async fn hdel(&self, key: String, field: String) -> Result<bool, Error> {
        let result = self.hget_all(key.clone()).await?;
        match result {
            Some(mut hash) => {
                // Remove the field from the keys vector
                hash.remove(&field);
                // Update to cache.
                self.cache.insert(key, Self::serialize_hash(&hash)).await;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Sets the given key to the specified value.
    ///
    /// # Note
    /// The expire(&self, String, i64) function is deprecated and will be unimplemented in the future.
    #[allow(unused)]
    async fn expire(&self, key: String, milliseconds: i64) -> Result<bool, Error> {
        unimplemented!()
    }

    async fn get_bit(&self, key: String, offset: u64) -> Result<bool, Error> {
        if let Some(value) = self.cache.get(&key).await {
            let byte_offset = (offset / 8) as usize;
            let bit_offset = (offset % 8) as u8;
            if byte_offset < value.len() {
                let byte = value.as_bytes()[byte_offset];
                Ok(((byte >> (7 - bit_offset)) & 1) == 1)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    async fn set_bit(&self, key: String, offset: u64, value: bool) -> Result<bool, Error> {
        let mut bytes = if let Some(existing) = self.cache.get(&key).await {
            existing.into_bytes()
        } else {
            Vec::new()
        };

        let byte_offset = (offset / 8) as usize;
        let bit_offset = (offset % 8) as u8;

        if byte_offset >= bytes.len() {
            bytes.resize(byte_offset + 1, 0);
        }

        let old_byte = bytes[byte_offset];
        let new_byte = if value {
            old_byte | (1 << (7 - bit_offset))
        } else {
            old_byte & !(1 << (7 - bit_offset))
        };

        bytes[byte_offset] = new_byte;
        self.cache.insert(key, String::from_utf8_lossy(&bytes).to_string()).await;

        Ok(((old_byte >> (7 - bit_offset)) & 1) == 1)
    }

    async fn del(&self, key: String) -> Result<bool, Error> {
        self.cache.invalidate(&key).await;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config::MemoryProperties;

    fn create_test_cache() -> StringMemoryCache {
        let config = MemoryProperties {
            initial_capacity: Some(100),
            max_capacity: Some(1000),
            ttl: Some(3600000),
            eviction_policy: Some("LRU".to_string()),
        };
        StringMemoryCache::new(&config)
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let cache = create_test_cache();
        assert!(cache.set("key1".to_string(), "value1".to_string(), None).await.unwrap());
        assert_eq!(cache.get("key1".to_string()).await.unwrap(), Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_set_nx() {
        let cache = create_test_cache();
        assert!(cache.set_nx("key2".to_string(), Some("value2".to_string())).await.unwrap());
        assert!(!cache.set_nx("key2".to_string(), Some("value3".to_string())).await.unwrap());
        assert_eq!(cache.get("key2".to_string()).await.unwrap(), Some("value2".to_string()));
    }

    // keys
    #[tokio::test]
    async fn test_keys() {
        let cache = create_test_cache();

        assert!(cache.set("key1".to_string(), "value1".to_string(), None).await.unwrap());
        assert!(cache.set("key2".to_string(), "value2".to_string(), None).await.unwrap());
        assert!(cache.set("key3".to_string(), "value3".to_string(), None).await.unwrap());

        let mut expected: Vec<String> = vec!["key1".to_string(), "key2".to_string(), "key3".to_string()];
        let mut result: Vec<String> = cache.keys("key*".to_string()).await.unwrap();
        expected.sort();
        result.sort();

        assert_eq!(expected, result);
    }

    #[tokio::test]
    async fn test_hset_and_hget() {
        let cache = create_test_cache();

        let key = String::from("test_hset_and_hget");

        // hset
        assert!(cache
            .hset_nx(key.clone(), String::from("field1"), String::from("value1"))
            .await
            .unwrap());
        assert!(cache
            .hset_nx(key.clone(), String::from("field2"), String::from("value2"))
            .await
            .unwrap());

        // hget
        let result1 = cache.hget(key.clone(), Some("field1".to_string())).await.unwrap().unwrap();
        assert_eq!(result1, "value1".to_string());

        let result2 = cache.hget_all(key.clone()).await.unwrap().unwrap();
        assert_eq!(result2.get("field1").unwrap().to_owned(), "value1".to_string());
        assert_eq!(result2.get("field2").unwrap().to_owned(), "value2".to_string());
    }

    #[tokio::test]
    async fn test_hkeys() {
        let cache = create_test_cache();

        let key = String::from("test_hkeys");

        assert!(cache
            .hset_nx(key.clone(), String::from("field1"), String::from("value1"))
            .await
            .unwrap());
        assert!(cache
            .hset_nx(key.clone(), String::from("field2"), String::from("value2"))
            .await
            .unwrap());

        let result = cache.hget(key.clone(), Some("field1".to_string())).await.unwrap().unwrap();
        assert_eq!(result, "value1".to_string());

        let mut expected = vec!["field1".to_string(), "field2".to_string()];
        let mut keys = cache.hkeys(key.clone()).await.unwrap();
        expected.sort();
        keys.sort();

        assert_eq!(keys, expected);
    }

    // Unimplemented not supported !!!
    // #[tokio::test]
    // async fn test_expire() {
    //     let cache = create_test_cache();
    //     assert!(cache.set("key3".to_string(), "value3".to_string(), None).await.unwrap());
    //     assert!(cache.expire("key3".to_string(), Some(100)).await.unwrap());

    //     tokio::time::sleep(Duration::from_millis(200)).await;

    //     let result = cache.get("key3".to_string()).await.unwrap();
    //     assert_eq!(result, None);
    // }

    #[tokio::test]
    async fn test_bit_operations() {
        let cache = create_test_cache();
        assert!(!cache.set_bit("bitkey".to_string(), 0, true).await.unwrap());
        assert!(cache.get_bit("bitkey".to_string(), 1).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete() {
        let cache = create_test_cache();
        assert!(cache.set("key4".to_string(), "value4".to_string(), None).await.unwrap());
        assert!(cache.del("key4".to_string()).await.unwrap());
        assert_eq!(cache.get("key4".to_string()).await.unwrap(), None);
    }
}
