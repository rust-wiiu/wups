use crate::bindings as c_wups;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("")]
    INVALID_ARGS,
    #[error("")]
    MALLOC_FAILED,
    #[error("")]
    UNEXPECTED_DATA_TYPE,
    #[error("")]
    BUFFER_TOO_SMALL,
    #[error("")]
    ALREADY_EXISTS,
    #[error("")]
    IO_ERROR,
    #[error("")]
    NOT_FOUND,
    #[error("")]
    INTERNAL_NOT_INITIALIZED,
    #[error("")]
    INTERNAL_INVALID_VERSION,
    #[error("")]
    UNKNOWN_ERROR(i32),
    #[error("An error occurred while parsing the UI tree.")]
    MENU_UI_ERROR(#[from] crate::ui::MenuError),
}

impl TryFrom<i32> for ConfigError {
    type Error = ConfigError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use c_wups::WUPSStorageError::*;
        if value >= 0 {
            Ok(Self::UNKNOWN_ERROR(value))
        } else {
            match value {
                WUPS_STORAGE_ERROR_INVALID_ARGS => Err(Self::INVALID_ARGS),
                WUPS_STORAGE_ERROR_MALLOC_FAILED => Err(Self::MALLOC_FAILED),
                WUPS_STORAGE_ERROR_UNEXPECTED_DATA_TYPE => Err(Self::UNEXPECTED_DATA_TYPE),
                WUPS_STORAGE_ERROR_BUFFER_TOO_SMALL => Err(Self::BUFFER_TOO_SMALL),
                WUPS_STORAGE_ERROR_ALREADY_EXISTS => Err(Self::ALREADY_EXISTS),
                WUPS_STORAGE_ERROR_IO_ERROR => Err(Self::IO_ERROR),
                WUPS_STORAGE_ERROR_NOT_FOUND => Err(Self::NOT_FOUND),
                WUPS_STORAGE_ERROR_INTERNAL_NOT_INITIALIZED => Err(Self::INTERNAL_NOT_INITIALIZED),
                WUPS_STORAGE_ERROR_INTERNAL_INVALID_VERSION => Err(Self::INTERNAL_INVALID_VERSION),
                v => Err(Self::UNKNOWN_ERROR(v)),
            }
        }
    }
}

pub trait WupsConfig {
    type Item;
    fn load() -> Result<Self::Item, ConfigError>;
    fn save(&self) -> Result<(), ConfigError>;
}
