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

use super::ipfilter::IPFilter;
use crate::cache::{redis::StringRedisCache, ICache};
use anyhow::{Error, Ok, Result};
use botwaf_types::forwarder::HttpIncomingRequest;
use std::{net::IpAddr, str::FromStr, sync::Arc};

pub struct RedisIPFilter {
    redis_cache: Arc<StringRedisCache>,
    redis_key: String,
}

impl RedisIPFilter {
    pub const NAME: &'static str = "REDIS";

    pub fn new(redis_cache: Arc<StringRedisCache>, redis_key: String) -> Arc<RedisIPFilter> {
        Arc::new(Self { redis_cache, redis_key })
    }

    fn get_client_ip(&self, incoming: Arc<HttpIncomingRequest>) -> Result<String, Error> {
        incoming
            .client_ip
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Client IP not found"))
    }

    /// Converts an IP address to a bitmap offset.
    fn get_ip_bitmap_offset(&self, incoming: Arc<HttpIncomingRequest>) -> Result<u64, Error> {
        let ip = IpAddr::from_str(self.get_client_ip(incoming)?.as_ref())?;
        match ip {
            IpAddr::V4(ipv4) => Ok(u32::from(ipv4) as u64),
            IpAddr::V6(ipv6) => {
                // For IPv6, we use a simplified mapping approach
                let octets = ipv6.octets();
                let mut result: u64 = 0;
                for i in 0..8 {
                    result = (result << 8) | ((((octets[i * 2] as u16) << 8) | (octets[i * 2 + 1] as u16)) as u64);
                }
                let offset = result % (u32::MAX as u64); // Limit to 32-bit range
                Ok(offset)
            }
        }
    }
}

#[async_trait::async_trait]
impl IPFilter for RedisIPFilter {
    async fn init(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn is_blocked(&self, incoming: Arc<HttpIncomingRequest>) -> Result<bool, Error> {
        let offset = self.get_ip_bitmap_offset(incoming)?;
        self.redis_cache.get_bit(self.redis_key.clone(), offset).await
    }

    async fn block_ip(&self, incoming: Arc<HttpIncomingRequest>) -> Result<bool, Error> {
        let offset = self.get_ip_bitmap_offset(incoming)?;
        self.redis_cache.set_bit(self.redis_key.clone(), offset, true).await
    }

    async fn unblock_ip(&self, incoming: Arc<HttpIncomingRequest>) -> Result<bool, Error> {
        let offset = self.get_ip_bitmap_offset(incoming)?;
        self.redis_cache.set_bit(self.redis_key.clone(), offset, false).await
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use axum::{ body::Body, http::{ Request, StatusCode } };
    // use std::net::{ IpAddr, Ipv4Addr };
    // use mockall::predicate::*;
    // use mockall::mock;

    // // Mock for StringRedisCache
    // mock! {
    //     pub StringRedisCache {
    //         async fn get_bit(&self, key: String, offset: u64) -> Result<bool, Error>;
    //         async fn set_bit(&self, key: String, offset: u64, value: bool) -> Result<bool, Error>;
    //     }

    //     impl Clone for StringRedisCache {
    //         fn clone(&self) -> Self;
    //     }
    // }

    // // Mock for IIPFilterHandler
    // mock! {
    //     pub IIPFilterHandlerTrait {}

    //     #[async_trait::async_trait]
    //     impl IIPFilterHandler for IIPFilterHandlerTrait {
    //         async fn is_ip_blocked(&self, ip: IpAddr) -> Result<bool, Error>;
    //         async fn block_ip(&self, ip: IpAddr) -> Result<bool, Error>;
    //         async fn unblock_ip(&self, ip: IpAddr) -> Result<bool, Error>;
    //         fn with_filter_middleware(&self, router: Router) -> Router;
    //     }
    // }

    // #[tokio::test]
    // async fn test_block_ip() {
    //     let mut mock_cache = MockStringRedisCache::new();
    //     mock_cache.expect_clone().returning(|| {
    //         let mut clone = MockStringRedisCache::new();
    //         clone.expect_set_bit().returning(|_, _, _| Ok(true));
    //         clone
    //     });

    //     mock_cache
    //         .expect_set_bit()
    //         .with(eq("ip_blacklist".to_string()), eq(167772160), eq(true))
    //         .returning(|_, _, _| Ok(true));

    //     let handler = RedisIPFilterHandler::new(mock_cache, "ip_blacklist".to_string());

    //     let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0));
    //     let result = handler.block_ip(ip).await;
    //     assert!(result.is_ok());
    //     assert!(result.unwrap());
    // }

    // #[tokio::test]
    // async fn test_is_ip_blocked() {
    //     let mut mock_cache = MockStringRedisCache::new();
    //     mock_cache.expect_clone().returning(|| {
    //         let mut clone = MockStringRedisCache::new();
    //         clone.expect_get_bit().returning(|_, _| Ok(true));
    //         clone
    //     });

    //     mock_cache
    //         .expect_get_bit()
    //         .with(eq("ip_blacklist".to_string()), eq(167772160))
    //         .returning(|_, _| Ok(true));

    //     let handler = RedisIPFilterHandler::new(mock_cache, "ip_blacklist".to_string());

    //     let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0));
    //     let result = handler.is_ip_blocked(ip).await;
    //     assert!(result.is_ok());
    //     assert!(result.unwrap());
    // }

    // #[tokio::test]
    // async fn test_filter_request() {
    //     let mut mock_cache = MockStringRedisCache::new();
    //     mock_cache.expect_clone().returning(|| {
    //         let mut clone = MockStringRedisCache::new();
    //         clone.expect_get_bit().returning(|_, _| Ok(true));
    //         clone
    //     });

    //     mock_cache.expect_get_bit().returning(|_, _| Ok(true));

    //     let handler = RedisIPFilterHandler::new(mock_cache, "ip_blacklist".to_string());

    //     let req = Request::builder().body(Body::empty()).unwrap();
    //     let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)), 8080);

    //     let next = |_req: Request<Body>| async {
    //         Ok(Response::builder().status(StatusCode::OK).body(Body::empty()).unwrap())
    //     };

    //     let result = RedisIPFilterHandler::do_filter_request(
    //         ConnectInfo(addr),
    //         req,
    //         Next::new(next),
    //         handler.instance.clone()
    //     ).await;

    //     assert!(result.is_err());
    //     assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    // }
}
