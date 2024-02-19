use std::collections::HashMap;
use std::fs;

use hyper::Method;
use serde::Deserialize;

use crate::cli::Environment;

#[derive(Deserialize, Debug)]
pub struct ApiSettings {
    pub api_uri: String,
    pub auth_uri: String,
    pub use_auth: bool,
    pub auth_method: String,
    pub api_method: String,
}

impl ApiSettings {
    pub fn auth_method(&self) -> Option<Method> {
        return Some(self.map_method(self.auth_method.clone()));
    }
    pub fn api_method(&self) -> Option<Method> {
        return Some(self.map_method(self.api_method.clone()));
    }

    fn map_method(&self, method: String) -> Method {
        return match method.as_str() {
            "POST" => Method::POST,
            "PATCH" => Method::PATCH,
            "PUT" => Method::PUT,
            "GET" => Method::GET,
            _ => Method::POST,
        };
    }
}

#[derive(Deserialize, Debug)]
pub struct AuthCredentials {
    pub auth_user: String,
    pub auth_password: String,
}

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub api_settings: ApiSettings,
    pub auth_credentials: AuthCredentials,
    pub api_url: HashMap<Environment, String>,
    pub auth_url: HashMap<Environment, String>,
}

impl Configuration {
    pub fn api_endpoint(&self, env: Environment) -> String {
        let host = self
            .api_url
            .get(&env)
            .unwrap()
            .clone();

        let path = self
            .api_settings
            .api_uri
            .clone();

        let url = host + "/" + &path;

        return url;
    }

    pub fn auth_endpoint(&self, env: Environment) -> String {
        let host = self
            .auth_url
            .get(&env)
            .unwrap()
            .clone();

        let path = self
            .api_settings
            .auth_uri
            .clone();

        let url = host + "/" + &path;

        return url;
    }
}

pub fn parse_config() -> Configuration {
    let toml_str = fs::read_to_string("config/config.toml")
        .expect("Failed to read config file");

    let config: Configuration = toml::from_str(&toml_str)
        .expect("Failed to parse config file");

    return config;
}
