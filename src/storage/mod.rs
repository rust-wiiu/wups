use core::ffi;

use crate::bindings as c_wups;
use alloc::ffi::CString;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
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
    #[error("CString cannot contain internal 0-bytes.")]
    CONTAINS_NULL_BYTES(#[from] alloc::ffi::NulError),
}

impl TryFrom<i32> for StorageError {
    type Error = StorageError;
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

pub trait WupsStorage {
    type T: Default;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type;

    fn load(name: &str) -> Result<Self::T, StorageError> {
        let name = CString::new(name)?;
        let mut value: Self::T = Default::default();
        let mut out = 0;

        let status = unsafe {
            c_wups::WUPSStorageAPI_GetItem(
                core::ptr::null_mut(),
                name.as_ptr(),
                Self::ITEM_TYPE,
                &mut value as *mut _ as *mut ffi::c_void,
                core::mem::size_of::<Self::T>() as u32,
                &mut out,
            )
        };
        debug_assert_eq!(out, core::mem::size_of::<Self::T>() as u32);
        StorageError::try_from(status)?;

        Ok(value)
    }

    fn store(name: &str, value: Self::T) -> Result<(), StorageError> {
        let name = CString::new(name)?;
        let mut value = value;
        let status = unsafe {
            c_wups::WUPSStorageAPI_StoreItem(
                core::ptr::null_mut(),
                name.as_ptr() as *const _,
                Self::ITEM_TYPE,
                &mut value as *mut _ as *mut ffi::c_void,
                core::mem::size_of::<Self::T>() as u32,
            )
        };
        StorageError::try_from(status)?;

        Ok(())
    }
}

impl WupsStorage for i32 {
    type T = i32;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type =
        c_wups::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_S32;
}

impl WupsStorage for f32 {
    type T = f32;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type =
        c_wups::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_FLOAT;
}

pub fn load<T: WupsStorage>(name: &str) -> Result<T::T, StorageError> {
    T::load(name)
}

pub fn store<T: WupsStorage>(name: &str, value: T::T) -> Result<(), StorageError> {
    T::store(name, value)
}

pub fn load_or_default<T: WupsStorage>(name: &str) -> T::T {
    match T::load(name) {
        Ok(v) => v,
        Err(_) => Default::default(),
    }
}
