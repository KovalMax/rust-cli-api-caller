use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Body, Client, Error, Method, Request, Url};
use serde::{Deserialize, Serialize};

use crate::config::AuthCredentials;

#[derive(Debug, Clone)]
pub struct ApiCaller {
    client: Client,
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

        let mut body_request = Request::new(request.method, Url::parse(&request.url).unwrap());
        *body_request.body_mut() = Some(Body::from(body));
        let headers = body_request.headers_mut();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        self.authorize(body_request).await
    }

    pub async fn api_call(&self, request: ApiRequest) -> Result<ApiResponse, Error> {
        let mut body_request = Request::new(request.method, Url::parse(&request.url).unwrap());
        *body_request.body_mut() = Some(Body::from(request.body));
        let headers = body_request.headers_mut();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(AUTHORIZATION, request.auth_token.parse().unwrap());

        self.request(body_request).await
    }

    async fn request(&self, request: Request) -> Result<ApiResponse, Error> {
        let resp = self.client.execute(request).await;

        match resp {
            Ok(response) => {
                let status = response.status();
                let bytes = response.bytes().await?;
                let body = String::from_utf8(bytes.to_vec()).unwrap();

                Ok(ApiResponse {
                    body,
                    status: status.as_u16(),
                })
            }
            Err(e) => Err(e),
        }
    }

    async fn authorize(&self, request: Request) -> Result<AuthResponse, String> {
        let resp = self.client.execute(request).await;

        match resp {
            Ok(response) => {
                let status = response.status();
                let bytes = response.bytes().await.unwrap();

                if status.as_u16() != 200 {
                    let body = String::from_utf8(bytes.to_vec()).unwrap();

                    return Err(body);
                }

                let body: AuthResponse = serde_json::from_slice(bytes.to_vec().as_slice()).unwrap();

                Ok(body)
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn create_client() -> ApiCaller {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .https_only(false)
        .build()
        .unwrap();

    ApiCaller { client }
}
