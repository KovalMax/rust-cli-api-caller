use clap::Parser;
use hyper::Method;

use cli_api_caller::{make_api_call, make_auth_call};
use cli_api_caller::cli::CliParams;
use cli_api_caller::client::create_client;
use cli_api_caller::config::parse_config;
use cli_api_caller::reader::{create_reader, CsvCell};

#[tokio::main]
async fn main() {
    let args = CliParams::parse();
    let config = parse_config();
    let client = create_client();

    let api_url = config.api_endpoint(args.environment.clone());
    let auth_url = config.auth_endpoint(args.environment.clone());

    let auth_response = make_auth_call(
        &client,
        auth_url,
        config.api_settings.auth_user,
        config.api_settings.auth_password,
    ).await.unwrap();

    let delimiter = args.delimiter.as_bytes().first().unwrap();
    let mut reader = create_reader(args.path, delimiter);

    for line in reader.deserialize::<CsvCell>() {
        let line = line.unwrap();
        let json = serde_json::to_string(&line).unwrap();

        let replaced_url = api_url.replace("{id}", line.id.as_str());

        let response = make_api_call(
            &client,
            replaced_url,
            json,
            auth_response.token(),
            Method::PATCH,
        ).await.unwrap();

        let (parts, body) = response.into_parts();
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
}