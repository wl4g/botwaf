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
    use anyhow::Error;
    use axum::http;
    use hyper::Request;
    // use auth::tests::MockUserProvider;
    // use auth::UserProvider;
    // use http_body::Body;
    // use hyper::StatusCode;
    // use servers::http::authorize::inner_auth;
    // use session::context::QueryContextRef;
    // use std::sync::Arc;

    #[tokio::test]
    async fn test_http_auth() {
        // TODO: Completion the JWT Bearer Authorization Postive and Negetive Cases.

        // // base64encode("username:password") == "dXNlcm5hbWU6cGFzc3dvcmQ="
        // let req = mock_http_request(Some("Basic dXNlcm5hbWU6cGFzc3dvcmQ="), None).unwrap();
        // let req = inner_auth(None, req).await.unwrap();
        // let ctx: &QueryContextRef = req.extensions().get().unwrap();
        // let user_info = ctx.current_user().unwrap();
        // let default = auth::userinfo_by_name(None);
        // assert_eq!(default.username(), user_info.username());

        // // In mock user provider, right username:password == "botwaf:botwaf"
        // let mock_user_provider = Some(Arc::new(MockUserProvider::default()) as Arc<dyn UserProvider>);

        // // base64encode("botwaf:botwaf") == "Z3JlcHRpbWU6Z3JlcHRpbWU="
        // let req = mock_http_request(Some("Basic Z3JlcHRpbWU6Z3JlcHRpbWU="), None).unwrap();
        // let req = inner_auth(mock_user_provider.clone(), req).await.unwrap();
        // let ctx: &QueryContextRef = req.extensions().get().unwrap();
        // let user_info = ctx.current_user().unwrap();
        // let default = auth::userinfo_by_name(None);
        // assert_eq!(default.username(), user_info.username());

        // let req = mock_http_request(None, None).unwrap();
        // let auth_res = inner_auth(mock_user_provider.clone(), req).await;
        // assert!(auth_res.is_err());
        // let mut resp = auth_res.unwrap_err();
        // assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        // assert_eq!(
        //     b"{\"code\":7003,\"error\":\"Not found http or grpc authorization header\",\"execution_time_ms\":0}",
        //     resp.data().await.unwrap().unwrap().as_ref()
        // );

        // // base64encode("username:password") == "dXNlcm5hbWU6cGFzc3dvcmQ="
        // let wrong_req = mock_http_request(Some("Basic dXNlcm5hbWU6cGFzc3dvcmQ="), None).unwrap();
        // let auth_res = inner_auth(mock_user_provider, wrong_req).await;
        // assert!(auth_res.is_err());
        // let mut resp = auth_res.unwrap_err();
        // assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        // assert_eq!(
        //     b"{\"code\":7000,\"error\":\"User not found, username: username\",\"execution_time_ms\":0}",
        //     resp.data().await.unwrap().unwrap().as_ref(),
        // );
    }

    #[allow(unused)]
    fn mock_http_request(auth_header: Option<&str>, uri: Option<&str>) -> Result<Request<()>, Error> {
        let mut req =
            Request::builder().uri(uri.unwrap_or(format!("http://localhost:9999/_/healthz?foo=bar").as_str()));
        if let Some(auth_header) = auth_header {
            req = req.header(http::header::AUTHORIZATION, auth_header);
        }
        Ok(req.body(()).unwrap())
    }
}
