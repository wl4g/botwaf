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

#[cfg(test)]
mod tests {
    use botwaf_server::{
        cache::{redis::StringRedisCache, ICache},
        config::config::RedisProperties,
    };
    use std::{env, time::Duration};

    fn create_test_cache() -> StringRedisCache {
        let env_nodes = env::var("IT_REDIS_NODES").ok();
        let password = env::var("IT_REDIS_PASSWORD").unwrap_or(String::from("bitnami"));

        let nodes = match env_nodes {
            Some(nodes_str) => nodes_str.split(',').filter_map(|url| url.trim().parse().ok()).collect(),
            None => vec![
                String::from("redis://localhost.com:6379"),
                String::from("redis://localhost.com:6380"),
                String::from("redis://localhost.com:6381"),
                String::from("redis://localhost.com:7379"),
                String::from("redis://localhost.com:7380"),
                String::from("redis://localhost.com:7381"),
            ],
        };

        let config = RedisProperties {
            nodes,
            username: None,
            password: Some(password),
            connection_timeout: None,
            response_timeout: None,
            retries: None,
            max_retry_wait: None,
            min_retry_wait: None,
            read_from_replicas: Some(true),
        };

        StringRedisCache::new(&config)
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let cache = create_test_cache();

        assert!(cache
            .set(String::from("key1"), String::from("value1"), None)
            .await
            .unwrap());

        let result = cache.get(String::from("key1")).await.unwrap().unwrap();
        assert_eq!(result, String::from("value1"));
    }

    #[tokio::test]
    async fn test_keys() {
        let cache = create_test_cache();

        let key1 = String::from("test_keys_key1");
        let key2 = String::from("test_keys_key2");

        assert!(cache.set(key1.clone(), String::from("value1"), None).await.unwrap());
        assert!(cache.set(key2.clone(), String::from("value2"), None).await.unwrap());

        let mut result = cache.keys(String::from("test_keys_key*")).await.unwrap();
        let mut expected = vec![key1.clone(), key2.clone()];
        result.sort();
        expected.sort();

        assert_eq!(result, expected);
    }

    #[allow(unused)]
    #[tokio::test]
    async fn test_set_nx() {
        let cache = create_test_cache();

        let key = String::from("test_setnx_key");

        cache.del(key.clone()).await; // if exists, delete it

        assert!(cache.set_nx(key.clone(), Some(String::from("value2"))).await.unwrap());
        assert!(!cache.set_nx(key.clone(), Some(String::from("value3"))).await.unwrap());
        assert_eq!(cache.get(key.clone()).await.unwrap(), Some(String::from("value2")));
    }

    #[allow(unused)]
    #[tokio::test]
    async fn test_hset_and_hget() {
        let cache = create_test_cache();

        // hset

        let key = String::from("test_hset_hget_key");
        cache.del(key.clone());

        cache
            .hset_nx(key.clone(), String::from("field1"), String::from("value1"))
            .await
            .unwrap();
        cache
            .hset_nx(key.clone(), String::from("field2"), String::from("value2"))
            .await
            .unwrap();
        cache
            .hset_nx(key.clone(), String::from("field3"), String::from("value3"))
            .await
            .unwrap();

        // hget

        let mut result1 = cache
            .hget(key.clone(), Some(String::from("field1")))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result1, String::from("value1"));
    }

    #[tokio::test]
    async fn test_hget_all() {
        let cache = create_test_cache();

        let key = String::from("test_hget_all");
        cache
            .hset_nx(key.clone(), String::from("field1"), String::from("value1"))
            .await
            .unwrap();
        cache
            .hset_nx(key.clone(), String::from("field2"), String::from("value2"))
            .await
            .unwrap();
        cache
            .hset_nx(key.clone(), String::from("field3"), String::from("value3"))
            .await
            .unwrap();

        let result = cache.hget_all(key.clone()).await.unwrap().unwrap();
        assert_eq!(result.get("field1").unwrap().to_owned(), "value1".to_string());
        assert_eq!(result.get("field2").unwrap().to_owned(), "value2".to_string());
        assert_eq!(result.get("field3").unwrap().to_owned(), "value3".to_string());
    }

    #[tokio::test]
    async fn test_hkeys() {
        let cache = create_test_cache();

        let key = String::from("test_hkeys_key");
        cache
            .hset_nx(key.clone(), String::from("field1"), String::from("value1"))
            .await
            .unwrap();
        cache
            .hset_nx(key.clone(), String::from("field2"), String::from("value2"))
            .await
            .unwrap();
        cache
            .hset_nx(key.clone(), String::from("field3"), String::from("value3"))
            .await
            .unwrap();

        let mut result = cache.hkeys(key.clone()).await.unwrap();
        let mut expected = vec![String::from("field1"), String::from("field2"), String::from("field3")];

        result.sort();
        expected.sort();

        assert_eq!(result, expected);
    }

    #[allow(unused)]
    #[tokio::test]
    async fn test_hdel() {
        let cache = create_test_cache();

        let key = String::from("test_hdel_key");
        cache.del(key.clone());

        cache
            .hset_nx(key.clone(), String::from("field1"), String::from("value1"))
            .await
            .unwrap();
        cache
            .hset_nx(key.clone(), String::from("field2"), String::from("value2"))
            .await
            .unwrap();
        cache
            .hset_nx(key.clone(), String::from("field3"), String::from("value3"))
            .await
            .unwrap();

        assert!(cache.hdel(key.clone(), String::from("field1")).await.unwrap());

        let result = cache.hget(key.clone(), Some(String::from("field1"))).await.unwrap();
        assert_eq!(result, None)
    }

    #[tokio::test]
    async fn test_expire() {
        let cache = create_test_cache();

        let key = String::from("test_expire_key1");

        assert!(cache.set(key.clone(), String::from("value1"), None).await.unwrap());
        assert!(cache.expire(key.clone(), 1000).await.unwrap());

        tokio::time::sleep(Duration::from_millis(1100)).await;

        let result = cache.get(key.clone()).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_bit_operations() {
        let cache = create_test_cache();

        assert!(cache.set_bit(String::from("bitkey"), 0, true).await.unwrap());
        assert!(cache.get_bit(String::from("bitkey"), 0).await.unwrap());
        assert!(!cache.get_bit(String::from("bitkey"), 1).await.unwrap());
    }

    #[tokio::test]
    async fn test_del() {
        let cache = create_test_cache();

        let key = String::from("test_del");
        let value = String::from("value1");

        assert!(cache.set(key.clone(), value.clone(), None).await.unwrap());
        assert_eq!(cache.get(key.clone()).await.unwrap().unwrap(), value);

        assert!(cache.del(key.clone()).await.unwrap());
        assert_eq!(cache.get(key.clone()).await.unwrap(), None);
    }
}
