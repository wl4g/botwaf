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

use super::config::{self, AppConfig};
use crate::llm::route::knowledge_router::__path_handle_knowledge_upload;
use botwaf_types::knowledge::KnowledgeUploadInfo;
use std::collections::BTreeMap;
use utoipa::openapi::{PathItem, Paths};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        version = "1.0.0",
        title = "Botwaf",
        description = "Botwaf - A Mini Open Source AI Bot WAF written in Rust.",
        license(name = "Apache 2.0", url = "https://www.apache.org/licenses/LICENSE-2.0"),
        contact(
            name = "Botwaf",
            url = "https://github.com/wl4g/botwaf",
            email = "jameswong1376@gmail.com"
        )
    ),
    //security((), "my_auth" = ["read:items", "edit:items"], "token_jwt" = []),
    external_docs(url = "https://github.com/wl4g/botwaf", description = "More about our APIs"),
    paths(
        // Knowledge
        handle_knowledge_upload,
    ),
    components(
        schemas(
            // Module of Knowledge
            KnowledgeUploadInfo,
        )
    ),
    modifiers(&ApiPathPrefixer)
)]
struct ApiDoc;

struct ApiPathPrefixer;

impl utoipa::Modify for ApiPathPrefixer {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let ctx_path = &config::get_config().server.context_path;

        let old_paths = std::mem::take(&mut openapi.paths);
        let mut new_paths_map: BTreeMap<String, PathItem> = old_paths
            .paths
            .into_iter()
            .map(|(path, item)| {
                (
                    match ctx_path {
                        Some(cp) => {
                            format!("{}{}", cp, path)
                        } // Add the prefix context path.
                        None => path,
                    },
                    item,
                )
            })
            .collect();

        openapi.paths = Paths::new();
        openapi.paths.paths.append(&mut new_paths_map);
    }
}

pub fn init(config: &AppConfig) -> SwaggerUi {
    // Manual build of OpenAPI.
    // use utoipa::openapi::{ ContactBuilder, InfoBuilder, LicenseBuilder, Paths };
    // let info = InfoBuilder::new()
    //   .title(config.swagger.title.to_string())
    //   .version(config.swagger.version.to_string())
    //   .description(Some(config.swagger.description.to_string()))
    //   .license(
    //       Some(
    //         LicenseBuilder::new()
    //           .name(config.swagger.license_name.to_string())
    //           .url(Some(config.swagger.license_url.to_string()))
    //           .build()
    //       )
    //     )
    //   .contact(
    //       Some(
    //         ContactBuilder::new()
    //           .name(Some(config.swagger.contact_name.to_string()))
    //           .url(Some(config.swagger.contact_url.to_string()))
    //           .email(Some(config.swagger.contact_email.to_string()))
    //           .build()
    //       )
    //     )
    //   .build();
    // let paths = Paths::new();
    // let openapi = utoipa::openapi::OpenApi::new(info, paths);

    // Auto build of OpenAPI.
    let openapi = ApiDoc::openapi();

    let swagger_ui_path = join_context_path(&config, config.swagger.ui_path.to_string());
    let openapi_url = join_context_path(&config, config.swagger.openapi_url.to_string());

    SwaggerUi::new(swagger_ui_path).url(openapi_url, openapi)
}

pub fn join_context_path(config: &AppConfig, path: String) -> String {
    // Absolute URI not needs to join context path.
    let schema = url::Url::parse(path.as_str())
        .map(|uri| uri.scheme().to_lowercase())
        .unwrap_or_default();
    if schema.starts_with("http") {
        return path;
    }
    match &config.server.context_path {
        // Add the prefix context path.
        Some(cp) => format!("{}{}", cp, path),
        None => path,
    }
}
