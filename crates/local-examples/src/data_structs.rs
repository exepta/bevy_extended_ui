use bevy_extended_ui_macros::html_shared;
use serde::Serialize;

/// Represents a data container with versioning, binary data, and usage tracking.
///
/// # Fields
/// - `version` (String): A string representing the version of the data pack.
/// - `data` (Vec<u8>): A vector of bytes that stores the binary content of the data pack.
/// - `used` (bool): A boolean flag indicating whether the data pack has been used or not.
#[html_shared]
#[derive(Debug, Serialize)]
pub struct DataPack {
    pub version: String,
    pub data: Vec<u8>,
    pub used: bool,
}

impl Default for DataPack {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            data: vec![0, 2, 1, 4, 18, 22, 29],
            used: false,
        }
    }
}

impl DataPack {
    #[allow(dead_code)]
    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

#[html_shared]
#[derive(Debug, Default, Serialize)]
#[allow(dead_code)]
pub enum DataState {
    Active,
    #[default]
    Inactive,
}
