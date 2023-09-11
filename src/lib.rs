use hyper::{Method, Response};

use crate::client::{ApiCaller, ApiRequest, AuthRequest, AuthResponse};

pub mod cli;
pub mod reader;
pub mod config;
pub mod client;


pub async fn make_auth_call(http: &ApiCaller, auth_url: String, username: String, password: String) -> Result<AuthResponse, String> {
    let auth_request = AuthRequest::create(
        auth_url,
        Method::POST,
        username,
        password,
    );

    return http.authorize(auth_request).await;
}

pub async fn make_api_call(http: &ApiCaller, url: String, body: String, auth_token: String, method: Method) -> hyper::Result<Response<String>> {
    let api_request = ApiRequest::create_api_request(url, body, auth_token, method);

    return http.request(api_request).await;
}