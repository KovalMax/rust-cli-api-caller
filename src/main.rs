use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use tokio::sync::Semaphore;

use cli_api_caller::api_service::create_api_service;
use cli_api_caller::cli::CliParams;
use cli_api_caller::client::AuthRequest;
use cli_api_caller::config::parse_config;
use cli_api_caller::reader::{create_reader, CsvRow};

#[tokio::main]
async fn main() {
    println!("-- Starting execution --\n");

    let now = Instant::now();
    let args = CliParams::parse();
    let config = parse_config();
    let api_service = create_api_service();

    let api_url = config.api_endpoint(args.environment.clone());
    let auth_url = config.auth_endpoint(args.environment.clone());

    let auth_req = AuthRequest::create(
        auth_url,
        config.auth_credentials,
        config.api_settings.auth_method(),
    );

    let auth = api_service.authenticate(auth_req).await;

    let delimiter = args
        .delimiter
        .as_bytes()
        .first()
        .unwrap();

    let mut join_handles = Vec::new();
    let mut reader = create_reader(args.path, delimiter);
    let semaphore = Arc::new(Semaphore::new(args.limit as usize));

    for row in reader.deserialize::<CsvRow>() {
        let row = match row {
            Ok(r) => r,
            Err(e) => {
                println!("Failed to deserialize row, {}", e);
                continue;
            }
        };

        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .unwrap();

        let url = api_url.clone();
        let token = auth.token().clone();
        let client = api_service.clone();
        let method = config.api_settings.api_method().clone();

        let handle = tokio::spawn(async move {
            drop(permit);
            return client.send_api_call(token, url, row, method).await;
        });

        join_handles.push(handle);
    }

    let (mut successful, mut failed) = (0, 0);
    for handle in join_handles {
        let result = handle.await.unwrap();
        if result.succeed {
            successful += 1;
        } else {
            failed += 1;
        }

        println!("{}", result.body);
    }

    semaphore.close();
    let elapsed = now.elapsed();
    println!(
        "\nDone!\nTotal tasks executed: {}\nSuccessful: {} | Failed: {}\nTime elapsed: {:.1} - seconds",
        successful + failed, successful, failed, elapsed.as_secs_f32()
    );
}