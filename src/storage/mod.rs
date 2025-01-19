/// Storage
///
/// Persistent key-value-like datastore.
use core::ffi;

use crate::bindings as c_wups;
use alloc::{
    ffi::CString,
    string::{String, ToString},
    vec::Vec,
};
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

const STORAGE_MAX_LENGTH: usize = 1024;

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

// region: Impls

impl WupsStorage for i32 {
    type T = i32;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type =
        c_wups::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_S32;
}

impl WupsStorage for i64 {
    type T = i64;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type =
        c_wups::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_S64;
}

impl WupsStorage for u32 {
    type T = u32;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type =
        c_wups::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_U32;
}

impl WupsStorage for u64 {
    type T = u64;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type =
        c_wups::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_U64;
}

impl WupsStorage for bool {
    type T = bool;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type =
        c_wups::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_BOOL;
}

impl WupsStorage for f32 {
    type T = f32;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type =
        c_wups::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_FLOAT;
}

impl WupsStorage for f64 {
    type T = f64;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type =
        c_wups::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_DOUBLE;
}

// endregion

impl WupsStorage for String {
    type T = String;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type =
        c_wups::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_STRING;

    fn load(name: &str) -> Result<Self::T, StorageError> {
        let name = CString::new(name)?;
        let mut value = [0u8; STORAGE_MAX_LENGTH];
        let mut out = 0;

        let status = unsafe {
            c_wups::WUPSStorageAPI_GetItem(
                core::ptr::null_mut(),
                name.as_ptr(),
                Self::ITEM_TYPE,
                &mut value as *mut _ as *mut ffi::c_void,
                value.len() as u32,
                &mut out,
            )
        };
        debug_assert!(out < value.len() as u32);
        StorageError::try_from(status)?;

        let s = String::from_utf8_lossy(&value[..(out as usize)]);
        let s = s.strip_suffix('\0').unwrap_or(&s).to_string();
        Ok(s)
    }

    fn store(name: &str, value: Self::T) -> Result<(), StorageError> {
        let name = CString::new(name)?;
        if value.len() >= STORAGE_MAX_LENGTH {
            return Err(StorageError::BUFFER_TOO_SMALL);
        }
        let mut value = value;

        let status = unsafe {
            c_wups::WUPSStorageAPI_StoreItem(
                core::ptr::null_mut(),
                name.as_ptr() as *const _,
                Self::ITEM_TYPE,
                value.as_mut_ptr() as *mut _,
                value.len() as u32,
            )
        };
        StorageError::try_from(status)?;

        Ok(())
    }
}

impl WupsStorage for Vec<u8> {
    type T = Vec<u8>;
    const ITEM_TYPE: c_wups::WUPSStorageItemTypes::Type =
        c_wups::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_BINARY;

    fn load(name: &str) -> Result<Self::T, StorageError> {
        let name = CString::new(name)?;
        let mut value = [0u8; STORAGE_MAX_LENGTH];
        let mut out = 0;

        let status = unsafe {
            c_wups::WUPSStorageAPI_GetItem(
                core::ptr::null_mut(),
                name.as_ptr(),
                Self::ITEM_TYPE,
                &mut value as *mut _ as *mut ffi::c_void,
                value.len() as u32,
                &mut out,
            )
        };
        debug_assert!(out < value.len() as u32);
        StorageError::try_from(status)?;

        Ok(value[..(out as usize)].to_vec())
    }

    fn store(name: &str, value: Self::T) -> Result<(), StorageError> {
        let name = CString::new(name)?;
        if value.len() >= STORAGE_MAX_LENGTH {
            return Err(StorageError::BUFFER_TOO_SMALL);
        }
        let mut value = value;

        let status = unsafe {
            c_wups::WUPSStorageAPI_StoreItem(
                core::ptr::null_mut(),
                name.as_ptr() as *const _,
                Self::ITEM_TYPE,
                value.as_mut_ptr() as *mut _,
                value.len() as u32,
            )
        };
        StorageError::try_from(status)?;

        Ok(())
    }
}

/// Loads previously saved data from storage.
///
/// # Examples
///
/// ```no_run
/// use wups::storage::{store, load};
///
/// store::<i32>("exists", 42);
/// assert_eq!(load::<i32>("exists"), 42);
/// assert_eq!(load::<i32>("doesnt exist"), Err(StorageError::NOT_FOUND));
/// ```
pub fn load<T: WupsStorage>(name: &str) -> Result<T::T, StorageError> {
    T::load(name)
}

/// Loads previously saved data from storage or returns default value for given type.
///
/// # Examples
///
/// ```no_run
/// use wups::storage::{store, load_or_default};
///
/// store::<i32>("exists", 42);
/// assert_eq!(load::<i32>("exists"), 42);
/// assert_eq!(load_or_default::<i32>("doesnt exist"), 0);
/// ```
pub fn load_or_default<T: WupsStorage>(name: &str) -> T::T {
    match T::load(name) {
        Ok(v) => v,
        Err(_) => Default::default(),
    }
}

/// Save data into storage.
///
/// # Examples
///
/// ```no_run
/// use wups::storage::store;
///
/// store::<i32>("integer", 42);
/// store::<u64>("big integer", 420);
/// store::<f32>("float", 3.14);
/// store::<String>("string", "Hello there!".to_string());
/// ```
pub fn store<T: WupsStorage>(name: &str, value: T::T) -> Result<(), StorageError> {
    T::store(name, value)
}

/// Deletes previously saved data from storage.
pub fn delete(name: &str) -> Result<(), StorageError> {
    let name = CString::new(name)?;
    let status = unsafe { c_wups::WUPSStorageAPI_DeleteItem(core::ptr::null_mut(), name.as_ptr()) };
    StorageError::try_from(status)?;
    Ok(())
}

/// Wipe the entire storage. **ALL DATA WILL BE LOST**.
pub fn reset() -> Result<(), StorageError> {
    let status = unsafe { c_wups::WUPSStorageAPI_WipeStorage() };
    StorageError::try_from(status)?;
    Ok(())
}

/// Force a reload of the storage.
pub fn reload() -> Result<(), StorageError> {
    let status = unsafe { c_wups::WUPSStorageAPI_ForceReloadStorage() };
    StorageError::try_from(status)?;
    Ok(())
}
