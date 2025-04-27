-- SPDX-License-Identifier: GNU GENERAL PUBLIC LICENSE Version 3
--
-- Copyleft (c) 2024 James Wong. This file is part of James Wong.
-- is free software: you can redistribute it and/or modify it under
-- the terms of the GNU General Public License as published by the
-- Free Software Foundation, either version 3 of the License, or
-- (at your option) any later version.
--
-- James Wong is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY; without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU General Public License for more details.
--
-- You should have received a copy of the GNU General Public License
-- along with James Wong.  If not, see <https://www.gnu.org/licenses/>.
--
-- IMPORTANT: Any software that fully or partially contains or uses materials
-- covered by this license must also be released under the GNU GPL license.
-- This includes modifications and derived works.
--
--
-- Create the sys_user table.
CREATE TABLE IF NOT EXISTS sys_user (
    id BIGINT PRIMARY KEY NOT NULL,
    name VARCHAR(64) NULL,
    -- "账号昵称"
    email VARCHAR(64) NULL,
    -- "邮箱, 可用于登录需唯一"
    phone VARCHAR(64) NULL,
    -- "手机号, 可用于登录需唯一"
    -- [ DEBUG]: echo -n "string" | openssl dgst -sha256 -binary | base64
    -- [OUTPUT]: RzKH+CmNunFjqJeQiVj3wOrnM+JdLgJ5kuou3JvtL6g=
    password VARCHAR(256) NULL,
    -- "静态密码"
    oidc_claims_sub VARCHAR(64) NULL,
    -- '标准 OIDC IdP 授权服务(如:Keycloak)返回的 claims sub 用于绑定唯一标识用户'
    oidc_claims_name VARCHAR(64) NULL,
    -- '标准 OIDC IdP 授权服务(如:Keycloak)返回的 chiams name/preferer_name'
    oidc_claims_email VARCHAR(64) NULL,
    -- '标准 OIDC IdP 授权服务(如:Keycloak)返回的 chimas email'
    github_claims_sub VARCHAR(64) NULL,
    -- 'Github IdP 授权服务返回的 claim sub 用于绑定唯一标识用户'
    github_claims_name VARCHAR(64) NULL,
    -- 'Github IdP 授权服务返回的 claims name/preferer_name'
    github_claims_email VARCHAR(64) NULL,
    -- 'Github IdP 授权服务返回的 claims email'
    google_claims_sub VARCHAR(64) NULL,
    -- 'Google IdP 授权服务返回的 sub claim 用于绑定唯一标识用户'
    google_claims_name VARCHAR(64) NULL,
    -- 'Google IdP 授权服务返回的 claims name/preferer_name'
    google_claims_email VARCHAR(64) NULL,
    -- 'Google IdP 授权服务返回的 claims email'
    ethers_address VARCHAR(64) NULL,
    -- 'Ethers Wallet 地址, 来自签名认证'
    lang VARCHAR(64) NULL,
    status INTEGER NULL default 0,
    create_by VARCHAR(64) NULL,
    create_time TIMESTAMPTZ default current_timestamp,
    update_by VARCHAR(64) NULL,
    update_time TIMESTAMPTZ default current_timestamp,
    del_flag INTEGER NOT NULL default 0,
);