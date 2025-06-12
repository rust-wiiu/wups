//! Storage Module
//!
//! This module provides a persistent key-value-like datastore for various data types. It allows
//! storing, loading, and deleting data with a simple API. The storage supports basic types such as
//! integers, floats, booleans, strings, and binary data.
//!
//! # Examples
//!
//! ```no_run
//! use wups::storage::{store, load, load_or_default, delete, reset, reload};
//!
//! // Store data
//! store::<i32>("integer", 42).unwrap();
//! store::<u64>("big_integer", 420).unwrap();
//! store::<f32>("float", 3.14).unwrap();
//! store::<String>("string", "Hello there!".to_string()).unwrap();
//!
//! // Load data
//! assert_eq!(load::<i32>("integer").unwrap(), 42);
//! assert_eq!(load::<u64>("big_integer").unwrap(), 420);
//! assert_eq!(load::<f32>("float").unwrap(), 3.14);
//! assert_eq!(load::<String>("string").unwrap(), "Hello there!".to_string());
//!
//! // Load data or return default value
//! assert_eq!(load_or_default::<i32>("nonexistent"), 0);
//!
//! // Delete data
//! delete("integer").unwrap();
//!
//! // Reset storage (wipe all data)
//! reset().unwrap();
//!
//! // Reload storage
//! reload().unwrap();
//! ```
//!
//! # Errors
//!
//! The module defines a `StorageError` enum to represent various errors that can occur during
//! storage operations. These include invalid arguments, memory allocation failures, unexpected
//! data types, buffer size issues, I/O errors, and more.
//!
//! # Traits
//!
//! The `WupsStorage` trait defines the interface for loading and storing data. It is implemented
//! for various data types, including integers, floats, booleans, strings, and binary data.
//!
//! # Constants
//!
//! - `STORAGE_MAX_LENGTH`: The maximum length for storage items, set to 1024 bytes.
//!
//! # Functions
//!
//! - [load][crate::storage::load]: Loads previously saved data
//!   from storage.
//! - [load_or_default][crate::storage::load_or_default]: Loads previously saved data from
//!   storage or returns the default value for the given type.
//! - [store][crate::storage::store]: Saves data into storage.
//! - [delete][crate::storage::delete]: Deletes previously saved data from storage.
//! - [reset][crate::storage::reset]: Wipes the entire storage, deleting all data.
//! - [reload][crate::storage::reload]: Forces a reload of the storage.

use core::ffi;

use alloc::{
    ffi::CString,
    string::{String, ToString},
    vec::Vec,
};
use thiserror::Error;
use wups_sys as sys;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("")]
    InvalidArgs,
    #[error("")]
    MallocFailed,
    #[error("")]
    UnexpectedDataType,
    #[error("")]
    BufferTooSmall,
    #[error("")]
    AlreadyExists,
    #[error("")]
    IoError,
    #[error("")]
    NotFound,
    #[error("")]
    InternalNotInitialized,
    #[error("")]
    InternalInvalidVersion,
    #[error("")]
    UnknownError(i32),
    #[error("CString cannot contain internal 0-bytes.")]
    ContainsNullBytes(#[from] alloc::ffi::NulError),
}

impl TryFrom<i32> for StorageError {
    type Error = Self;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use sys::WUPSStorageError as E;
        if value >= 0 {
            Ok(Self::UnknownError(value))
        } else {
            match value {
                E::WUPS_STORAGE_ERROR_INVALID_ARGS => Err(Self::InvalidArgs),
                E::WUPS_STORAGE_ERROR_MALLOC_FAILED => Err(Self::MallocFailed),
                E::WUPS_STORAGE_ERROR_UNEXPECTED_DATA_TYPE => Err(Self::UnexpectedDataType),
                E::WUPS_STORAGE_ERROR_BUFFER_TOO_SMALL => Err(Self::BufferTooSmall),
                E::WUPS_STORAGE_ERROR_ALREADY_EXISTS => Err(Self::AlreadyExists),
                E::WUPS_STORAGE_ERROR_IO_ERROR => Err(Self::IoError),
                E::WUPS_STORAGE_ERROR_NOT_FOUND => Err(Self::NotFound),
                E::WUPS_STORAGE_ERROR_INTERNAL_NOT_INITIALIZED => Err(Self::InternalNotInitialized),
                E::WUPS_STORAGE_ERROR_INTERNAL_INVALID_VERSION => Err(Self::InternalInvalidVersion),
                v => Err(Self::UnknownError(v)),
            }
        }
    }
}

const STORAGE_MAX_LENGTH: usize = 1024;

pub trait StorageCompatible {
    type T: Default;
    const ITEM_TYPE: sys::WUPSStorageItemTypes::Type;

    fn load(name: &str) -> Result<Self::T, StorageError> {
        let name = CString::new(name)?;
        let mut value: Self::T = Default::default();
        let mut out = 0;

        let status = unsafe {
            sys::WUPSStorageAPI_GetItem(
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
            sys::WUPSStorageAPI_StoreItem(
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

impl StorageCompatible for i32 {
    type T = Self;
    const ITEM_TYPE: sys::WUPSStorageItemTypes::Type =
        sys::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_S32;
}

impl StorageCompatible for i64 {
    type T = Self;
    const ITEM_TYPE: sys::WUPSStorageItemTypes::Type =
        sys::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_S64;
}

impl StorageCompatible for u32 {
    type T = Self;
    const ITEM_TYPE: sys::WUPSStorageItemTypes::Type =
        sys::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_U32;
}

impl StorageCompatible for u64 {
    type T = Self;
    const ITEM_TYPE: sys::WUPSStorageItemTypes::Type =
        sys::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_U64;
}

impl StorageCompatible for bool {
    type T = Self;
    const ITEM_TYPE: sys::WUPSStorageItemTypes::Type =
        sys::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_BOOL;
}

impl StorageCompatible for f32 {
    type T = Self;
    const ITEM_TYPE: sys::WUPSStorageItemTypes::Type =
        sys::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_FLOAT;
}

impl StorageCompatible for f64 {
    type T = Self;
    const ITEM_TYPE: sys::WUPSStorageItemTypes::Type =
        sys::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_DOUBLE;
}

// endregion

impl StorageCompatible for String {
    type T = Self;
    const ITEM_TYPE: sys::WUPSStorageItemTypes::Type =
        sys::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_STRING;

    fn load(name: &str) -> Result<Self::T, StorageError> {
        let name = CString::new(name)?;
        let mut value = [0u8; STORAGE_MAX_LENGTH];
        let mut out = 0;

        let status = unsafe {
            sys::WUPSStorageAPI_GetItem(
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
            return Err(StorageError::BufferTooSmall);
        }
        let mut value = value;

        let status = unsafe {
            sys::WUPSStorageAPI_StoreItem(
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

impl StorageCompatible for Vec<u8> {
    type T = Self;
    const ITEM_TYPE: sys::WUPSStorageItemTypes::Type =
        sys::WUPSStorageItemTypes::WUPS_STORAGE_ITEM_BINARY;

    fn load(name: &str) -> Result<Self::T, StorageError> {
        let name = CString::new(name)?;
        let mut value = [0u8; STORAGE_MAX_LENGTH];
        let mut out = 0;

        let status = unsafe {
            sys::WUPSStorageAPI_GetItem(
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
            return Err(StorageError::BufferTooSmall);
        }
        let mut value = value;

        let status = unsafe {
            sys::WUPSStorageAPI_StoreItem(
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
#[inline]
pub fn load<T: StorageCompatible>(name: &str) -> Result<T::T, StorageError> {
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
#[inline]
pub fn load_or_default<T: StorageCompatible>(name: &str) -> T::T {
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
#[inline]
pub fn store<T: StorageCompatible>(name: &str, value: T::T) -> Result<(), StorageError> {
    T::store(name, value)
}

/// Deletes previously saved data from storage.
#[inline]
pub fn delete(name: &str) -> Result<(), StorageError> {
    let name = CString::new(name)?;
    let status = unsafe { sys::WUPSStorageAPI_DeleteItem(core::ptr::null_mut(), name.as_ptr()) };
    StorageError::try_from(status)?;
    Ok(())
}

/// Wipe the entire storage. **ALL DATA WILL BE LOST**.
#[inline]
pub fn reset() -> Result<(), StorageError> {
    let status = unsafe { sys::WUPSStorageAPI_WipeStorage() };
    StorageError::try_from(status)?;
    Ok(())
}

/// Force a reload of the storage.
#[inline]
pub fn reload() -> Result<(), StorageError> {
    let status = unsafe { sys::WUPSStorageAPI_ForceReloadStorage() };
    StorageError::try_from(status)?;
    Ok(())
}

/// Save the storage to disk
#[inline]
pub fn save(force: bool) -> Result<(), StorageError> {
    let status = unsafe { sys::WUPSStorageAPI_SaveStorage(force) };
    StorageError::try_from(status)?;
    Ok(())
}
