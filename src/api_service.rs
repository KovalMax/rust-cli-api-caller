use hyper::{Error, Method};

use crate::client::{ApiCaller, ApiRequest, ApiResponse, AuthRequest, AuthResponse, create_client};
use crate::reader::CsvRow;

#[derive(Debug, Clone)]
pub struct ApiService {
    client: ApiCaller,
}

pub struct ApiResult {
    pub status: u16,
    pub succeed: bool,
    pub body: String,
    pub row: CsvRow,
}

impl ApiResult {
    pub fn create(
        status: u16,
        succeed: bool,
        body: String,
        row: CsvRow,
    ) -> Self {
        ApiResult {
            status,
            succeed,
            body,
            row,
        }
    }
}

impl ApiService {
    pub async fn authenticate(&self, request: AuthRequest) -> AuthResponse {
        let mut retry_once = 1;
        let mut last_error = String::new();
        while retry_once > 0 {
            let auth_response = self.client
                .auth_call(request.clone())
                .await;

            if let Ok(response) = auth_response {
                return response;
            }

            retry_once -= 1;
            last_error = auth_response.unwrap_err();
        }
        panic!("Auth failed with error - {}", last_error);
    }

    pub async fn send_api_call(&self, token: String, url: String, csv_row: CsvRow, method: Option<Method>) -> ApiResult {
        let replaced_url = url.replace("{id}", csv_row.id.as_str());
        let body_json = serde_json::to_string(&csv_row).unwrap();

        let request = ApiRequest::create(
            replaced_url,
            body_json,
            token,
            method,
        );

        let api_response = self.client.api_call(request).await;

        return self.unwrap_response(api_response, csv_row).await;
    }

    async fn unwrap_response(&self, result: Result<ApiResponse, Error>, row: CsvRow) -> ApiResult {
        return match result {
            Ok(res) => {
                match res.status {
                    200..=299 => ApiResult::create(res.status, true, res.body, row),
                    _ => ApiResult::create(res.status, false, res.body, row),
                }
            }
            Err(e) => ApiResult::create(500, false, e.to_string(), row),
        };
    }
}

pub fn create_api_service() -> ApiService {
    ApiService { client: create_client() }
}

