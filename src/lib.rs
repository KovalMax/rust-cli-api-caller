use hyper::{Response, Result};

use crate::reader::CsvRow;

pub mod cli;
pub mod reader;
pub mod config;
pub mod client;

pub struct ApiResult {
    pub status: u16,
    pub body: String,
    pub succeed: bool,
}

pub async fn handle_api_result(result: Result<Response<String>>, row: CsvRow) -> ApiResult {
    let mut success = format!(
        "Status was changed, MID - {}, status changed to - {}, for market - {}",
        row.mid, row.status, row.market
    );

    if let Some(reason) = row.status_reason {
        success.push_str(format!(", status reason - {reason}").as_str());
    }

    let mut fail = format!("Status wasn't changed, MID - {}, API response - ", row.mid);

    return match result {
        Ok(res) => {
            let (parts, body) = res.into_parts();
            let response_code = parts.status.as_u16();

            match response_code {
                200..=299 => ApiResult { status: response_code, body: success, succeed: true },
                _ => {
                    fail.push_str(body.as_str());

                    ApiResult { status: response_code, body: fail, succeed: false }
                }
            }
        }
        Err(e) => {
            fail.push_str(e.to_string().as_str());

            ApiResult { status: 500, body: fail, succeed: false }
        }
    };
}