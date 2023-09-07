use std::fs::File;
use std::path::PathBuf;

use csv::{Reader, ReaderBuilder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CsvCell {
    #[serde(skip_serializing)]
    pub mid: String,
    #[serde(rename = "id_item")]
    pub id: String,
    pub market: String,
    pub status: i8,
    pub status_reason: Option<String>,
}

pub fn create_reader(filepath: PathBuf, delimiter: &u8) -> Reader<File> {
    return ReaderBuilder::new()
        .delimiter(*delimiter)
        .flexible(true)
        .from_path(filepath)
        .expect("Error building csv reader");
}