use bevy_extended_ui::BeuStore;
use bevy_extended_ui_macros::html_shared;
use serde::Serialize;

/// Represents a data container with versioning, binary data, and usage tracking.
///
/// # Fields
/// - `version` (String): A string representing the version of the data pack.
/// - `data` (Vec<u8>): A vector of bytes that stores the binary content of the data pack.
/// - `used` (bool): A boolean flag indicating whether the data pack has been used or not.
#[html_shared]
#[derive(BeuStore, PartialEq, Clone, Debug, Serialize)]
pub struct DataPack {
    pub version: String,
    pub data: Vec<u8>,
    pub used: bool,
    pub state: DataState,
}

impl Default for DataPack {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            data: vec![0, 2, 1, 4, 18, 22, 29],
            used: false,
            state: DataState::Inactive,
        }
    }
}

impl DataPack {
    #[allow(dead_code)]
    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    #[allow(dead_code)]
    pub fn set_used(&mut self) {
        self.used = true;
    }
}

/// Represents the state of data, typically to indicate whether it is active or inactive.
///
/// # Variants
/// - `Active`: Represents an active state for the data.
/// - `Inactive`: The default variant, represents an inactive or idle state for the data.
///
/// # Attributes
/// - `#[html_shared]`: Used to mark this enum as shared for HTML-related purposes.
/// - `#[derive(Debug, Default, Serialize)]`: Automatically derives the `Debug`, `Default`, and `Serialize` traits for the `DataState` enum.
/// - `#[allow(dead_code)]`: Prevents warnings for unused code related to this enum.
#[html_shared]
#[derive(BeuStore, PartialEq, Clone, Debug, Default, Serialize)]
#[allow(dead_code)]
pub enum DataState {
    Active,
    /// The default state.
    #[default]
    Inactive,
}
