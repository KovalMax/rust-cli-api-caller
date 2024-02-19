use hyper::{Body, Client, Error, Method, Request};
use hyper::client::HttpConnector;
use hyper_rustls::{ConfigBuilderExt, HttpsConnector};
use serde::{Deserialize, Serialize};

use crate::config::AuthCredentials;

#[derive(Debug, Clone)]
pub struct ApiCaller {
    client: Client<HttpsConnector<HttpConnector>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct AuthRequest {
    #[serde(skip_serializing)]
    url: String,
    #[serde(skip_serializing)]
    method: Method,
    username: String,
    password: String,
}

pub struct ApiRequest {
    url: String,
    body: String,
    method: Method,
    auth_token: String,
}

impl ApiRequest {
    pub fn create(url: String, body: String, auth_token: String, method: Option<Method>) -> Self {
        ApiRequest {
            url,
            body,
            auth_token,
            method: method.unwrap_or(Method::PATCH),
        }
    }
}

impl AuthRequest {
    pub fn create(url: String, credentials: AuthCredentials, method: Option<Method>) -> Self {
        AuthRequest {
            url,
            method: method.unwrap_or(Method::POST),
            username: credentials.auth_user,
            password: credentials.auth_password,
        }
    }
}

#[derive(Debug)]
pub struct ApiResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Deserialize, Debug)]
pub struct AuthResponse {
    access_token: String,
    token_type: String,
}

impl AuthResponse {
    pub fn token(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }
}

impl ApiCaller {
    pub async fn auth_call(&self, request: AuthRequest) -> Result<AuthResponse, String> {
        let body = serde_json::to_string(&request).unwrap();

        let body_request = Request::builder()
            .header("Content-Type", "application/json")
            .method(request.method)
            .uri(request.url)
            .body(Body::from(body))
            .unwrap();

        return self.authorize(body_request).await;
    }

    pub async fn api_call(&self, request: ApiRequest) -> Result<ApiResponse, Error> {
        let body_request = Request::builder()
            .header("Content-Type", "application/json")
            .header("Authorization", request.auth_token)
            .method(request.method)
            .uri(request.url)
            .body(Body::from(request.body)).unwrap();

        return self.request(body_request).await;
    }

    async fn request(&self, request: Request<Body>) -> Result<ApiResponse, Error> {
        let resp = self
            .client
            .request(request)
            .await;

        return match resp {
            Ok(response) => {
                let (parts, body) = response.into_parts();
                let bytes = hyper::body::to_bytes(body)
                    .await
                    .unwrap();

                let body = String::from_utf8(bytes.to_vec())
                    .unwrap();

                Ok(ApiResponse { body, status: parts.status.as_u16() })
            }
            Err(e) => Err(e),
        };
    }

    async fn authorize(&self, request: Request<Body>) -> Result<AuthResponse, String> {
        let resp = self
            .client
            .request(request)
            .await;

        return match resp {
            Ok(response) => {
                let (parts, body) = response.into_parts();
                let bytes = hyper::body::to_bytes(body)
                    .await
                    .unwrap();

                if parts.status != 200 {
                    let body = String::from_utf8(bytes.to_vec())
                        .unwrap();

                    return Err(body);
                }

                let body: AuthResponse = serde_json::from_slice(bytes.to_vec().as_slice())
                    .unwrap();

                Ok(body)
            }
            Err(e) => Err(e.to_string()),
        };
    }
}

pub fn create_client() -> ApiCaller {
    let tls = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_native_roots()
        .with_no_client_auth();

    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls)
        .https_or_http()
        .enable_http1()
        .build();

    ApiCaller { client: Client::builder().build(connector) }
}