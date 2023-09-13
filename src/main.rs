use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use tokio::sync::Semaphore;

use cli_api_caller::cli::CliParams;
use cli_api_caller::client::{ApiRequest, AuthRequest, create_client};
use cli_api_caller::config::parse_config;
use cli_api_caller::handle_api_result;
use cli_api_caller::reader::{create_reader, CsvRow};

#[tokio::main]
async fn main() {
    let now = Instant::now();
    let args = CliParams::parse();
    let config = parse_config();
    let client = create_client();

    let api_url = config.api_endpoint(args.environment.clone());
    let auth_url = config.auth_endpoint(args.environment.clone());

    let auth_req = AuthRequest::create(auth_url, config.auth_credentials, None);
    let auth_response = client
        .auth_call(auth_req)
        .await;
    let token = match auth_response {
        Ok(r) => r.token(),
        Err(e) => {
            panic!("Auth failed with error - {}", e);
        }
    };


    let delimiter = args
        .delimiter
        .as_bytes()
        .first()
        .unwrap();

    let semaphore = Arc::new(Semaphore::new(args.limit as usize));
    let mut reader = create_reader(args.path, delimiter);
    let mut join_handles = Vec::new();

    for row in reader.deserialize::<CsvRow>() {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .unwrap();

        let row = row.unwrap();
        let url = api_url.clone();
        let token = token.clone();
        let client = client.clone();

        join_handles.push(tokio::spawn(async move {
            let replaced_url = url.replace("{id}", row.id.as_str());
            let body_json = serde_json::to_string(&row).unwrap();

            let request = ApiRequest::create(
                replaced_url,
                body_json,
                token,
                None,
            );
            let response = client
                .api_call(request)
                .await;

            let result = handle_api_result(response, row)
                .await;

            drop(permit);

            return result;
        }));
    }

    let total = join_handles.len();
    for handle in join_handles {
        let result = handle.await.unwrap();
        println!("{}", result.body);
    }


    semaphore.close();

    let elapsed = now.elapsed();
    println!(
        "\nDone!\nTotal tasks executed: {}\nTime elapsed: {} - seconds",
        total, elapsed.as_secs()
    );
}