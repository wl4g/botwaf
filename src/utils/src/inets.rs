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

use local_ip_address::local_ip;
use std::net::{IpAddr, Ipv4Addr};

pub fn get_local_non_loopback_ip_str() -> String {
    get_local_non_loopback_ip().map(|ip| ip.to_string()).unwrap_or_default()
}

pub fn get_local_non_loopback_ip() -> Option<Ipv4Addr> {
    match local_ip() {
        Ok(ip) => {
            match ip {
                IpAddr::V4(ipv4) => {
                    if !ipv4.is_loopback() {
                        Some(ipv4)
                    } else {
                        None
                    }
                }
                IpAddr::V6(_) => None, // Our care the IPv4
            }
        }
        Err(_) => None,
    }
}
