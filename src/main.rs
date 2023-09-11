use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use hyper::Method;
use tokio::sync::Semaphore;

use cli_api_caller::{make_api_call, make_auth_call};
use cli_api_caller::cli::CliParams;
use cli_api_caller::client::create_client;
use cli_api_caller::config::parse_config;
use cli_api_caller::reader::{create_reader, CsvCell};

#[tokio::main]
async fn main() {
    let now = Instant::now();
    let args = CliParams::parse();
    let config = parse_config();
    let client = create_client();

    let api_url = config.api_endpoint(args.environment.clone());
    let semaphore = Arc::new(Semaphore::new(args.limit as usize));

    let auth_response = make_auth_call(
        &client,
        config.auth_endpoint(args.environment.clone()),
        config.api_settings.auth_user,
        config.api_settings.auth_password,
    ).await.unwrap();

    let delimiter = args.delimiter.as_bytes().first().unwrap();
    let mut reader = create_reader(args.path, delimiter);

    let mut total = 0;

    for line in reader.deserialize::<CsvCell>() {
        total += 1;
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let url = api_url.clone();
        let token = auth_response.token().clone();
        let client = client.clone();
        let line = line.unwrap();

        tokio::spawn(async move {
            let line = line.to_owned();
            let client = client.to_owned();
            let url = url.to_owned();
            let token = token.to_owned();

            let replaced_url = url.replace("{id}", line.id.as_str());
            let json = serde_json::to_string(&line).unwrap();

            let response = make_api_call(
                &client,
                replaced_url,
                json,
                token,
                Method::PATCH,
            ).await;

            match response {
                Ok(res) => {
                    let (parts, body) = res.into_parts();
                    if parts.status != 200 {
                        println!("{}",
                                 format!("Status change failed for item with mid {}, response: {}",
                                         line.mid, body)
                        );
                    } else {
                        println!("{}",
                                 format!("Status has been changed to {} for {} region for item with mid {}",
                                         line.status, line.market, line.mid)
                        );
                    }
                }
                Err(e) => {
                    println!("Error happened during API call, {}", e.to_string())
                }
            }
            drop(permit);
        });
    }

    let elapsed = now.elapsed();
    println!("Finished!\nTotal tasks executed: {}\nTime elapsed: {} - seconds",
             total, elapsed.as_secs());
}