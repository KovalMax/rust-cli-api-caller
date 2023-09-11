use clap::Parser;
use serde::Deserialize;

#[derive(clap::ValueEnum, Clone, Debug, Hash, Deserialize, Eq, PartialEq)]
pub enum Environment {
    PROD,
    STAGE,
    SANDBOX,
    DEV,
}

#[derive(Parser, Debug, Clone)]
#[clap(author = "Max Koval", version)]
///Helper package in case you need to read data from file and push it into some API
pub struct CliParams {
    #[clap(value_enum)]
    pub environment: Environment,
    ///Path to the csv file with the data to read from
    pub path: std::path::PathBuf,
    #[clap(default_value = ",")]
    ///CSV file delimiter
    pub delimiter: String,
    #[clap(default_value_t = 300)]
    pub limit: u16,
}