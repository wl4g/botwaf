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

use std::sync::Arc;

use anyhow::Error;

use crate::types::server::HttpIncomingRequest;

#[async_trait::async_trait]
pub trait IPFilterHandler {
    /// Checks if an IP address is in the blacklist
    async fn is_blocked(&self, incoming: Arc<HttpIncomingRequest>) -> Result<bool, Error>;

    /// Adds an IP address to the blacklist
    async fn block_ip(&self, incoming: Arc<HttpIncomingRequest>) -> Result<bool, Error>;

    /// Removes an IP address from the blacklist
    async fn unblock_ip(&self, incoming: Arc<HttpIncomingRequest>) -> Result<bool, Error>;
}
