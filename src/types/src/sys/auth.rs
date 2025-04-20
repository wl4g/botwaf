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

use serde::{ Deserialize, Serialize };
use validator::Validate;

// ----- Password login types. -----

#[derive(Deserialize, Clone, Debug, Validate, utoipa::ToSchema, utoipa::IntoParams)]
pub struct PasswordPubKeyRequest {
    #[serde(rename = "fpToken")]
    #[validate(length(min = 1, max = 128))]
    pub fingerprint_token: String, // User agent machine fingerprint token.
}

#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct PasswordPubKeyResponse {
    pub pubkey: String,
}

#[derive(Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct PasswordLoginRequest {
    pub username: String,
    pub password: String,
    #[serde(rename = "fpToken")]
    pub fingerprint_token: String,
    //pub seccode: Option<String>, // TODO: SMS/Email security code.
}

// ----- OIDC login types. ------

#[derive(Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct CallbackOidcRequest {
    pub code: Option<String>,
}

// ----- Github OAuth2 login types. -----

#[derive(Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct CallbackGithubRequest {
    pub code: Option<String>,
}

/*
curl -L \
-H "Accept: application/vnd.github+json" \
-H "Authorization: Bearer gho_E5VkThQhOygKDwfGLEsolKxR58uhxxxxxxxx" \
-H "X-GitHub-Api-Version: 2022-11-28" \
https://api.github.com/user
{
    "login": "wl4g",
    "id": 29530154,
    "node_id": "MDQ6VXNlcjI5NTMwMTU0",
    "avatar_url": "https://avatars.githubusercontent.com/u/29530154?v=4",
    "gravatar_id": "",
    "url": "https://api.github.com/users/wl4g",
    "html_url": "https://github.com/wl4g",
    "followers_url": "https://api.github.com/users/wl4g/followers",
    "following_url": "https://api.github.com/users/wl4g/following{/other_user}",
    "gists_url": "https://api.github.com/users/wl4g/gists{/gist_id}",
    "starred_url": "https://api.github.com/users/wl4g/starred{/owner}{/repo}",
    "subscriptions_url": "https://api.github.com/users/wl4g/subscriptions",
    "organizations_url": "https://api.github.com/users/wl4g/orgs",
    "repos_url": "https://api.github.com/users/wl4g/repos",
    "events_url": "https://api.github.com/users/wl4g/events{/privacy}",
    "received_events_url": "https://api.github.com/users/wl4g/received_events",
    "type": "User",
    "site_admin": false,
    "name": "Mr.James Wong",
    "company": "UNASCRIBED",
    "blog": "https://blogs.wl4g.com",
    "location": "China",
    "email": null,
    "hireable": null,
    "bio": "https://gitee.com/wl4g",
    "twitter_username": null,
    "public_repos": 47,
    "public_gists": 0,
    "followers": 19,
    "following": 9,
    "created_at": "2017-06-19T03:34:47Z",
    "updated_at": "2024-03-10T16:30:37Z"
}
*/
#[derive(Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct GithubUserInfo {
    pub id: Option<i64>,
    pub login: Option<String>,
    pub node_id: Option<String>,
    pub avatar_url: Option<String>,
    pub gravatar_id: Option<String>,
    pub url: Option<String>,
    pub html_url: Option<String>,
    pub followers_url: Option<String>,
    pub following_url: Option<String>,
    pub gists_url: Option<String>,
    pub starred_url: Option<String>,
    pub subscriptions_url: Option<String>,
    pub organizations_url: Option<String>,
    pub repos_url: Option<String>,
    pub events_url: Option<String>,
    pub received_events_url: Option<String>,
    #[serde(rename = "type")]
    pub user_type: Option<String>,
    pub site_admin: Option<bool>,
    pub name: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub location: Option<String>,
    pub email: Option<String>,
    pub hireable: Option<String>,
    pub bio: Option<String>,
    pub twitter_username: Option<String>,
    pub public_repos: Option<i32>,
    pub public_gists: Option<i32>,
    pub followers: Option<i32>,
    pub following: Option<i32>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl GithubUserInfo {
    pub fn default(
        id: Option<i64>,
        login: Option<String>,
        email: Option<String>
    ) -> GithubUserInfo {
        GithubUserInfo {
            id,
            login,
            node_id: None,
            avatar_url: None,
            gravatar_id: None,
            url: None,
            html_url: None,
            followers_url: None,
            following_url: None,
            gists_url: None,
            starred_url: None,
            subscriptions_url: None,
            organizations_url: None,
            repos_url: None,
            events_url: None,
            received_events_url: None,
            user_type: None,
            site_admin: None,
            name: None,
            company: None,
            blog: None,
            location: None,
            email,
            hireable: None,
            bio: None,
            twitter_username: None,
            public_repos: None,
            public_gists: None,
            followers: None,
            following: None,
            created_at: None,
            updated_at: None,
        }
    }
}

// ----- Wallet login types. -----

#[derive(Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct EthersWalletLoginRequest {
    pub address: String,
    pub signature: String,
    pub message: String,
}

// ----- Logged types. -----

#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct LoggedResponse {
    #[serde(rename = "errcode")]
    pub errcode: i16,
    #[serde(rename = "errmsg")]
    pub errmsg: String,
    // pub provider: Option<String>,
    #[serde(rename = "redirectUrl")]
    pub redirect_url: Option<String>,
    #[serde(rename = "accessToken")]
    pub access_token: Option<TokenWrapper>,
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<TokenWrapper>,
}

#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct TokenWrapper {
    #[serde(rename = "value")]
    pub value: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: u64,
}

#[derive(Deserialize, Clone, Debug, utoipa::ToSchema, utoipa::IntoParams)]
pub struct LogoutRequest {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}
