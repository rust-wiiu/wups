//! Config Menu in the Aroma Plugin Menu

// this is quite kinda swag frfr ngl but overkill for now: https://github.com/dkosmari/libwupsxx

use crate::storage;
use thiserror::Error;
use wups_sys as sys;
use wut::{
    ffi::{CStr, CString, NulError},
    string::{String, ToString},
    vec::Vec,
};

// region: MenuError

#[derive(Debug, Error)]
pub enum MenuError {
    #[error("Unknown error")]
    Unknown(sys::WUPSConfigAPIStatus::Type),
    #[error("The base of the UI must be a root node.")]
    AlreadyInitialized,
    #[error("")]
    InvalidArgument,
    #[error("")]
    OutOfMemory,
    #[error("")]
    NotFound,
    #[error("")]
    InvalidPluginIdentifier,
    #[error("")]
    MissingCallback,
    #[error("")]
    ModuleNotFound,
    #[error("")]
    ModuleMissingExport,
    #[error("")]
    UnsupportedVersion,
    #[error("")]
    UnsupportedCommand,
    #[error("")]
    LibUninitialized,
    #[error("Storage API was not initialized")]
    STORAGE(#[from] storage::StorageError),
    #[error("Internal 0-bytes")]
    InternalNullByte(#[from] NulError),
}

impl TryFrom<sys::WUPSConfigAPICallbackStatus::Type> for MenuError {
    type Error = Self;
    fn try_from(value: sys::WUPSConfigAPICallbackStatus::Type) -> Result<Self, Self::Error> {
        use sys::WUPSConfigAPIStatus as E;

        match value {
            E::WUPSCONFIG_API_RESULT_SUCCESS => Ok(Self::Unknown(E::WUPSCONFIG_API_RESULT_SUCCESS)),

            E::WUPSCONFIG_API_RESULT_INVALID_ARGUMENT => Err(Self::InvalidArgument),

            E::WUPSCONFIG_API_RESULT_OUT_OF_MEMORY => Err(Self::OutOfMemory),

            E::WUPSCONFIG_API_RESULT_NOT_FOUND => Err(Self::NotFound),

            E::WUPSCONFIG_API_RESULT_INVALID_PLUGIN_IDENTIFIER => {
                Err(Self::InvalidPluginIdentifier)
            }

            E::WUPSCONFIG_API_RESULT_MISSING_CALLBACK => Err(Self::MissingCallback),

            E::WUPSCONFIG_API_RESULT_MODULE_NOT_FOUND => Err(Self::ModuleNotFound),

            E::WUPSCONFIG_API_RESULT_MODULE_MISSING_EXPORT => Err(Self::ModuleMissingExport),

            E::WUPSCONFIG_API_RESULT_UNSUPPORTED_VERSION => Err(Self::UnsupportedVersion),

            E::WUPSCONFIG_API_RESULT_UNSUPPORTED_COMMAND => Err(Self::UnsupportedCommand),

            E::WUPSCONFIG_API_RESULT_LIB_UNINITIALIZED => Err(Self::LibUninitialized),

            v => Err(Self::Unknown(v)),
        }
    }
}

// endregion

/// Used for creating **stateless** config menu. Information is stored via [storage][crate::storage].
///
/// Open the menu by pressing "↓ + L + Minus" on the gamepad.
///
/// # Example
/// ```
/// struct MyMenu;
/// impl ConfigMenu for MyMenu {
///     fn open(root: config::MenuRoot) -> Result<(), config::MenuError> {
///         root.add(config::Label::new("Label"))?;
///     }
/// }
/// ```
pub trait ConfigMenu {
    /// Initialize the config menu
    ///
    /// Should be called inside the [on_initialize][crate::on_initialize] function.
    ///
    /// **Should not be overwritten unless special control is required.**
    fn init(name: &str) -> Result<(), MenuError> {
        let name = CString::new(name).unwrap();
        let opt = sys::WUPSConfigAPIOptionsV1 {
            name: name.as_ptr(),
        };

        let status = unsafe {
            sys::WUPSConfigAPI_Init(opt, Some(Self::_open_callback), Some(Self::_close_callback))
        };
        MenuError::try_from(status)?;

        Ok(())
    }

    /// C callback function for config menu
    ///
    /// **Should not be overwritten unless special control is required.**
    extern "C" fn _open_callback(
        root: sys::WUPSConfigCategoryHandle,
    ) -> sys::WUPSConfigAPICallbackStatus::Type {
        use sys::WUPSConfigAPICallbackStatus as S;
        match Self::open(MenuRoot::from(root)) {
            Ok(_) => S::WUPSCONFIG_API_CALLBACK_RESULT_SUCCESS,
            Err(_) => S::WUPSCONFIG_API_CALLBACK_RESULT_ERROR,
        }
    }

    /// C callback function for config menu
    ///
    /// **Should not be overwritten unless special control is required.**
    extern "C" fn _close_callback() {}

    /// Open callback.
    ///
    /// Called when the plugin menu is opened.
    ///
    /// All items must be rooted in the `root` node to be added to the UI.
    fn open(root: MenuRoot) -> Result<(), MenuError>;

    /// Close callback.
    ///
    /// Called when the plugin menu is closed.
    fn close() -> Result<(), MenuError> {
        storage::save(false)?;
        Ok(())
    }
}

pub trait MenuItem {
    fn attach(self, handle: sys::WUPSConfigCategoryHandle) -> Result<(), MenuError>;
}

pub trait Attachable {
    fn add(&self, item: impl MenuItem) -> Result<(), MenuError>;
}

// region: MenuRoot

pub struct MenuRoot(sys::WUPSConfigCategoryHandle);

impl From<sys::WUPSConfigCategoryHandle> for MenuRoot {
    fn from(value: sys::WUPSConfigCategoryHandle) -> Self {
        Self(value)
    }
}

impl Attachable for MenuRoot {
    fn add(&self, item: impl MenuItem) -> Result<(), MenuError> {
        item.attach(self.0)
    }
}

// endregion

// region: Menu

/// Sub menu containing items.
///
/// # Example
///
/// ```
/// let sub = config::Menu::new("Menu 1")?;
/// sub.add(config::Label::new("Label 1"))?;
/// sub.add(config::Label::new("Label 2"))?;
/// root.add(sub)?;
/// ```
pub struct Menu {
    text: String,
    handle: sys::WUPSConfigCategoryHandle,
}

impl Menu {
    pub fn new(text: &str) -> Result<Self, MenuError> {
        let mut handle = sys::WUPSConfigCategoryHandle::default();
        let c_text = CString::new(text).unwrap();

        let opt = sys::WUPSConfigAPICreateCategoryOptions {
            version: sys::WUPS_API_CATEGORY_OPTION_VERSION_V1,
            data: sys::WUPSConfigAPICreateCategoryOptions__bindgen_ty_1 {
                v1: sys::WUPSConfigAPICreateCategoryOptionsV1 {
                    name: c_text.as_ptr(),
                },
            },
        };

        let status = unsafe { sys::WUPSConfigAPI_Category_CreateEx(opt, &mut handle) };
        MenuError::try_from(status)?;

        Ok(Self {
            text: text.to_string(),
            handle: handle,
        })
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }
}

impl Attachable for Menu {
    fn add(&self, item: impl MenuItem) -> Result<(), MenuError> {
        item.attach(self.handle)
    }
}

impl MenuItem for Menu {
    fn attach(self, handle: sys::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
        let status = unsafe { sys::WUPSConfigAPI_Category_AddCategory(handle, self.handle) };
        MenuError::try_from(status)?;
        Ok(())
    }
}

// endregion

// region: Label

/// Text display
///
/// # Example
///
/// ```
/// root.add(config::Label::new("Label"))?;
/// ```
pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl MenuItem for Label {
    fn attach(self, handle: sys::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
        let text = CString::new(self.text.as_str()).unwrap();

        let status = unsafe { sys::WUPSConfigItemStub_AddToCategory(handle, text.as_ptr()) };
        MenuError::try_from(status)?;
        Ok(())
    }
}

// endregion

// region: Toggle

/// Binary toggle
///
/// # Example
///
/// ```
/// root.add(config::Toggle::new(
///     "Toggle",
///     "my_toggle_id",
///     true,
///     "On",
///     "Off",
/// ))?;
///
/// assert_eq!(storage::load::<bool>("my_toggle_id").unwrap(), true);
/// // toggle is changed...
/// assert_eq!(storage::load::<bool>("my_toggle_id").unwrap(), false);
/// ```
pub struct Toggle {
    text: String,
    id: String,
    default: bool,
    true_value: String,
    false_value: String,
}

impl Toggle {
    /// # Arguments
    ///
    /// - `text`: Text to display on the toggle.
    /// - `id`: [storage][crate::storage] id to access value.
    /// - `default`: Default value if id doesn't exist in storage.
    /// - `trueValue`: Text to display when value is `true`.
    /// - `falseValue`: Text to display when value is `false`.
    pub fn new(text: &str, id: &str, default: bool, true_value: &str, false_value: &str) -> Self {
        Self {
            text: text.to_string(),
            id: id.to_string(),
            default,
            true_value: true_value.to_string(),
            false_value: false_value.to_string(),
        }
    }
}

impl MenuItem for Toggle {
    fn attach(self, handle: sys::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
        let text = CString::new(self.text.as_str()).unwrap();
        let id = CString::new(self.id.as_str()).unwrap();
        let true_value = CString::new(self.true_value.as_str()).unwrap();
        let false_value = CString::new(self.false_value.as_str()).unwrap();

        let current = match storage::load::<bool>(&self.id) {
            Ok(v) => v,
            Err(storage::StorageError::NotFound) => {
                storage::store::<bool>(&self.id, self.default)?;
                self.default
            }
            Err(e) => return Err(MenuError::STORAGE(e)),
        };

        let status = unsafe {
            sys::WUPSConfigItemBoolean_AddToCategoryEx(
                handle,
                id.as_ptr(),
                text.as_ptr(),
                self.default,
                current,
                Some(_callback_toggle_changed),
                true_value.as_ptr(),
                false_value.as_ptr(),
            )
        };
        MenuError::try_from(status)?;

        Ok(())
    }
}

extern "C" fn _callback_toggle_changed(item: *mut sys::ConfigItemBoolean, value: bool) {
    let _ = storage::store::<bool>(
        &unsafe { CStr::from_ptr((*item).identifier) }.to_string_lossy(),
        value,
    );
}

// endregion

// region: Range

/// Select a number from a range.
///
/// # Example
///
/// ```
/// root.add(config::Range::new("Range", "my_range_id", 0, -5, 5))?;
///
/// assert_eq!(storage::load::<i32>("my_range_id").unwrap(), 0);
/// // range is increased...
/// assert_eq!(storage::load::<i32>("my_range_id").unwrap(), 1);
/// ```
pub struct Range {
    text: String,
    id: String,
    default: i32,
    min: i32,
    max: i32,
}

impl Range {
    pub fn new(text: &str, id: &str, default: i32, min: i32, max: i32) -> Self {
        debug_assert!(min < max);
        debug_assert!(min < default);
        debug_assert!(default < max);

        Self {
            text: text.to_string(),
            id: id.to_string(),
            default,
            min,
            max,
        }
    }
}

impl MenuItem for Range {
    fn attach(self, handle: sys::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
        let text = CString::new(self.text.as_str()).unwrap();
        let id = CString::new(self.id.as_str()).unwrap();

        let current = match storage::load::<i32>(&self.id) {
            Ok(v) => {
                if v > self.min && v < self.max {
                    v
                } else {
                    self.default
                }
            }
            Err(storage::StorageError::NotFound) => {
                storage::store::<i32>(&self.id, self.default)?;
                self.default
            }
            Err(e) => return Err(MenuError::STORAGE(e)),
        };

        let status = unsafe {
            sys::WUPSConfigItemIntegerRange_AddToCategory(
                handle,
                id.as_ptr(),
                text.as_ptr(),
                self.default,
                current,
                self.min,
                self.max,
                Some(_callback_range_changed),
            )
        };
        MenuError::try_from(status)?;

        Ok(())
    }
}

extern "C" fn _callback_range_changed(item: *mut sys::ConfigItemIntegerRange, value: i32) {
    let _ = storage::store::<i32>(
        &unsafe { CStr::from_ptr((*item).identifier) }.to_string_lossy(),
        value,
    );
}

// this is overkill but should outline on how to extend API in future
/*
pub trait RangeCompatible {
    type T: storage::StorageCompatible<T: From<i32> + Into<i32>>;
    extern "C" fn callback(item: *mut sys::ConfigItemIntegerRange, value: i32) {
        let _ = storage::store::<Self::T>(
            &unsafe { CStr::from_ptr((*item).identifier) }.to_string_lossy(),
            From::from(value),
        );
    }
}

impl RangeCompatible for i32 {
    type T = i32;
}

pub struct Range<T: RangeCompatible> {
    text: String,
    id: String,
    default: T,
    min: T,
    max: T,
}

impl<T: RangeCompatible> Range<T> {
    pub fn new(text: &str, id: &str, default: T, min: T, max: T) -> Self {
        Self {
            text: text.to_string(),
            id: id.to_string(),
            default,
            min,
            max,
        }
    }
}

impl<T: RangeCompatible> MenuItem for Range<T> {
    fn attach(&self, handle: sys::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
        todo!()
    }
}
    */

// endregion

// region: Select

/// Select a value from a predefined list.
///
/// # Example
///
/// ```
/// root.add(config::Select::new(
///     "Select",
///     "my_select_id",
///     0,
///     vec!["A", "B", "C"],
/// ))?;
///
/// assert_eq!(storage::load::<u32>("my_select_id").unwrap(), 0);
/// // select is switched to "C"...
/// assert_eq!(storage::load::<u32>("my_select_id").unwrap(), 2);
/// ```
pub struct Select {
    text: String,
    id: String,
    default: u32,
    options: Vec<String>,
}

impl Select {
    pub fn new(text: &str, id: &str, default: u32, options: Vec<impl ToString>) -> Self {
        debug_assert!(default < options.len() as u32);
        Select {
            text: text.to_string(),
            id: id.to_string(),
            default,
            options: options.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl MenuItem for Select {
    fn attach(self, handle: sys::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
        let text = CString::new(self.text.as_str()).unwrap();
        let id = CString::new(self.id.as_str()).unwrap();

        let strings: Result<Vec<CString>, NulError> =
            self.options.into_iter().map(|s| CString::new(s)).collect();
        let strings = strings?;

        let mut options: Vec<_> = strings
            .iter()
            .enumerate()
            .map(|(i, s)| sys::ConfigItemMultipleValuesPair {
                value: i as u32,
                valueName: s.as_ptr(),
            })
            .collect();

        let current = match storage::load::<u32>(&self.id) {
            Ok(v) => {
                if v > 0 && v < options.len() as u32 {
                    v
                } else {
                    self.default
                }
            }
            Err(storage::StorageError::NotFound) => {
                storage::store::<u32>(&self.id, self.default)?;
                self.default
            }
            Err(e) => return Err(MenuError::STORAGE(e)),
        };

        let status = unsafe {
            sys::WUPSConfigItemMultipleValues_AddToCategory(
                handle,
                id.as_ptr(),
                text.as_ptr(),
                self.default as i32,
                current as i32,
                options.as_mut_ptr(),
                options.len() as i32,
                Some(_callback_select_changed),
            )
        };
        MenuError::try_from(status)?;

        Ok(())
    }
}

extern "C" fn _callback_select_changed(item: *mut sys::ConfigItemMultipleValues, index: u32) {
    let _ = storage::store::<u32>(
        &unsafe { CStr::from_ptr((*item).identifier) }.to_string_lossy(),
        index,
    );
}

// endregion
