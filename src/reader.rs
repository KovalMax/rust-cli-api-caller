use std::fs::File;
use std::path::PathBuf;

use csv::{Reader, ReaderBuilder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CsvRow {
    #[serde(skip_serializing)]
    pub mid: String,
    #[serde(rename = "id_item", skip_serializing)]
    pub id: String,
    #[serde(skip_serializing)]
    pub market: String,
    pub status: i8,
    #[serde(rename(serialize = "statusReason"))]
    pub status_reason: Option<String>,
}

pub fn create_reader(filepath: PathBuf, delimiter: &u8) -> Reader<File> {
    ReaderBuilder::new()
        .delimiter(*delimiter)
        .flexible(true)
        .from_path(filepath)
        .expect("Error building csv reader")
}
