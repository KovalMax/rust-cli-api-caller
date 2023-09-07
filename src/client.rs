use hyper::{Body, Client, Method, Request, Response};
use hyper::client::HttpConnector;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ApiCaller {
    http: Client<HttpConnector>,
}

#[derive(Serialize, Debug)]
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
    pub fn create_api_request(url: String, body: String, auth_token: String, method: Method) -> Self {
        ApiRequest { url, body, auth_token, method }
    }
}

impl AuthRequest {
    pub fn create(url: String, method: Method, username: String, password: String) -> Self {
        AuthRequest { url, method, username, password }
    }
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
    pub async fn request(&self, request: ApiRequest) -> hyper::Result<Response<String>> {
        let req = Request::builder()
            .header("Content-Type", "application/json")
            .header("Authorization", request.auth_token)
            .method(request.method)
            .uri(request.url)
            .body(Body::from(request.body));

        let resp = self.http.request(req.unwrap()).await;
        return match resp {
            Ok(response) => {
                let (parts, body) = response.into_parts();
                let bytes = hyper::body::to_bytes(body).await;
                let body = String::from_utf8(bytes.unwrap().to_vec());

                return Ok(Response::from_parts(parts, body.unwrap()));
            }
            Err(e) => Err(e),
        };
    }

    pub async fn authorize(&self, request: AuthRequest) -> Result<AuthResponse, hyper::Error> {
        let body = serde_json::to_string(&request).unwrap();

        let req = Request::builder()
            .header("Content-Type", "application/json")
            .method(request.method)
            .uri(request.url)
            .body(Body::from(body));

        let resp = self.http.request(req.unwrap()).await;

        return match resp {
            Ok(response) => {
                let bytes = hyper::body::to_bytes(response).await?;
                let body: AuthResponse = serde_json::from_slice(bytes.to_vec().as_slice()).unwrap();

                return Ok(body);
            }
            Err(e) => Err(e),
        };
    }
}

pub fn create_client() -> ApiCaller {
    ApiCaller { http: Client::new() }
}