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

use crate::config::config::OAuth2Properties;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

// Using unified abstraction as OAuth2Config base class.
pub async fn create_oauth2_client(oauth2_config: &OAuth2Properties) -> Option<BasicClient> {
    if oauth2_config.enabled.unwrap_or(false) {
        Some(
            BasicClient::new(
                ClientId::new(oauth2_config.client_id.as_ref().unwrap().clone()),
                Some(ClientSecret::new(oauth2_config.client_secret.as_ref().unwrap().clone())),
                AuthUrl::new(oauth2_config.auth_url.as_ref().unwrap().clone()).unwrap(),
                Some(TokenUrl::new(oauth2_config.token_url.as_ref().unwrap().clone()).unwrap()),
            )
            .set_redirect_uri(RedirectUrl::new(oauth2_config.redirect_url.as_ref().unwrap().clone()).unwrap()),
        )
    } else {
        None
    }
}
