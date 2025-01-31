// ui

// this is quite cool but overkill for now: https://github.com/dkosmari/libwupsxx

pub mod glyphs;

use core::ffi::CStr;

use crate::{bindings as c_wups, storage};
use alloc::{
    ffi::{CString, NulError},
    string::{String, ToString},
    vec::Vec,
};
use thiserror::Error;

// region: MenuError

#[derive(Debug, Error)]
pub enum MenuError {
    #[error("Unknown error")]
    UNKNOWN(c_wups::WUPSConfigAPIStatus::Type),
    #[error("The base of the UI must be a root node.")]
    ALREADY_INITIALIZED,
    #[error("")]
    INVALID_ARGUMENT,
    #[error("")]
    OUT_OF_MEMORY,
    #[error("")]
    NOT_FOUND,
    #[error("")]
    INVALID_PLUGIN_IDENTIFIER,
    #[error("")]
    MISSING_CALLBACK,
    #[error("")]
    MODULE_NOT_FOUND,
    #[error("")]
    MODULE_MISSING_EXPORT,
    #[error("")]
    UNSUPPORTED_VERSION,
    #[error("")]
    UNSUPPORTED_COMMAND,
    #[error("")]
    LIB_UNINITIALIZED,
    #[error("Storage API was not initialized")]
    STORAGE(#[from] storage::StorageError),
    #[error("Internal 0-bytes")]
    INTERNAL_NULL_BYTE(#[from] NulError),
}

impl TryFrom<c_wups::WUPSConfigAPICallbackStatus::Type> for MenuError {
    type Error = Self;
    fn try_from(value: c_wups::WUPSConfigAPICallbackStatus::Type) -> Result<Self, Self::Error> {
        use c_wups::WUPSConfigAPIStatus as E;

        match value {
            E::WUPSCONFIG_API_RESULT_SUCCESS => Ok(Self::UNKNOWN(E::WUPSCONFIG_API_RESULT_SUCCESS)),

            E::WUPSCONFIG_API_RESULT_INVALID_ARGUMENT => Err(Self::INVALID_ARGUMENT),

            E::WUPSCONFIG_API_RESULT_OUT_OF_MEMORY => Err(Self::OUT_OF_MEMORY),

            E::WUPSCONFIG_API_RESULT_NOT_FOUND => Err(Self::NOT_FOUND),

            E::WUPSCONFIG_API_RESULT_INVALID_PLUGIN_IDENTIFIER => {
                Err(Self::INVALID_PLUGIN_IDENTIFIER)
            }

            E::WUPSCONFIG_API_RESULT_MISSING_CALLBACK => Err(Self::MISSING_CALLBACK),

            E::WUPSCONFIG_API_RESULT_MODULE_NOT_FOUND => Err(Self::MODULE_NOT_FOUND),

            E::WUPSCONFIG_API_RESULT_MODULE_MISSING_EXPORT => Err(Self::MODULE_MISSING_EXPORT),

            E::WUPSCONFIG_API_RESULT_UNSUPPORTED_VERSION => Err(Self::UNSUPPORTED_VERSION),

            E::WUPSCONFIG_API_RESULT_UNSUPPORTED_COMMAND => Err(Self::UNSUPPORTED_COMMAND),

            E::WUPSCONFIG_API_RESULT_LIB_UNINITIALIZED => Err(Self::LIB_UNINITIALIZED),

            v => Err(Self::UNKNOWN(v)),
        }
    }
}

// endregion

/// Used for creating **stateless** config menu
///
/// Open the menu by pressing "↓ + L + Minus" on the gamepad
pub trait ConfigMenu {
    /// Initialize the config menu
    ///
    /// Should be called inside the [on_initialize][crate::on_initialize] function.
    ///
    /// **Should not be overwritten unless special control is required.**
    fn init(name: &str) -> Result<(), MenuError> {
        let name = CString::new(name).unwrap();
        let opt = c_wups::WUPSConfigAPIOptionsV1 {
            name: name.as_ptr(),
        };

        let status = unsafe {
            c_wups::WUPSConfigAPI_Init(opt, Some(Self::_open_callback), Some(Self::_close_callback))
        };
        MenuError::try_from(status)?;

        Ok(())
    }

    /// C callback function for config menu
    ///
    /// **Should not be overwritten unless special control is required.**
    extern "C" fn _open_callback(
        root: c_wups::WUPSConfigCategoryHandle,
    ) -> c_wups::WUPSConfigAPICallbackStatus::Type {
        use c_wups::WUPSConfigAPICallbackStatus as S;
        match Self::open(MenuRoot::from(root)) {
            Ok(_) => S::WUPSCONFIG_API_CALLBACK_RESULT_SUCCESS,
            Err(_) => S::WUPSCONFIG_API_CALLBACK_RESULT_ERROR,
        }
    }

    /// C callback function for config menu
    ///
    /// **Should not be overwritten unless special control is required.**
    extern "C" fn _close_callback() {}

    fn open(root: MenuRoot) -> Result<(), MenuError>;

    fn close() -> Result<(), MenuError> {
        Ok(())
    }
}

pub trait MenuItem {
    fn attach(self, handle: c_wups::WUPSConfigCategoryHandle) -> Result<(), MenuError>;
}

pub trait Attachable {
    fn add(&self, item: impl MenuItem) -> Result<(), MenuError>;
}

// region: MenuRoot

pub struct MenuRoot(c_wups::WUPSConfigCategoryHandle);

impl From<c_wups::WUPSConfigCategoryHandle> for MenuRoot {
    fn from(value: c_wups::WUPSConfigCategoryHandle) -> Self {
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

pub struct Menu {
    text: String,
    handle: c_wups::WUPSConfigCategoryHandle,
}

impl Menu {
    pub fn new(text: &str) -> Result<Self, MenuError> {
        let mut handle = c_wups::WUPSConfigCategoryHandle::default();
        let c_text = CString::new(text).unwrap();

        let opt = c_wups::WUPSConfigAPICreateCategoryOptions {
            version: c_wups::WUPS_API_CATEGORY_OPTION_VERSION_V1,
            data: c_wups::WUPSConfigAPICreateCategoryOptions__bindgen_ty_1 {
                v1: c_wups::WUPSConfigAPICreateCategoryOptionsV1 {
                    name: c_text.as_ptr(),
                },
            },
        };

        let status = unsafe { c_wups::WUPSConfigAPI_Category_CreateEx(opt, &mut handle) };
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
    fn attach(self, handle: c_wups::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
        let status = unsafe { c_wups::WUPSConfigAPI_Category_AddCategory(handle, self.handle) };
        MenuError::try_from(status)?;
        Ok(())
    }
}

// endregion

// region: Label

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
    fn attach(self, handle: c_wups::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
        let text = CString::new(self.text.as_str()).unwrap();

        let status = unsafe { c_wups::WUPSConfigItemStub_AddToCategory(handle, text.as_ptr()) };
        MenuError::try_from(status)?;
        Ok(())
    }
}

// endregion

// region: Toggle

pub struct Toggle {
    text: String,
    id: String,
    default: bool,
    trueValue: String,
    falseValue: String,
}

impl Toggle {
    pub fn new(text: &str, id: &str, default: bool, trueValue: &str, falseValue: &str) -> Self {
        Self {
            text: text.to_string(),
            id: id.to_string(),
            default,
            trueValue: trueValue.to_string(),
            falseValue: falseValue.to_string(),
        }
    }
}

impl MenuItem for Toggle {
    fn attach(self, handle: c_wups::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
        let text = CString::new(self.text.as_str()).unwrap();
        let id = CString::new(self.id.as_str()).unwrap();
        let trueValue = CString::new(self.trueValue.as_str()).unwrap();
        let falseValue = CString::new(self.falseValue.as_str()).unwrap();

        let current = match storage::load::<bool>(&self.id) {
            Ok(v) => v,
            Err(storage::StorageError::NOT_FOUND) => {
                storage::store::<bool>(&self.id, self.default)?;
                self.default
            }
            Err(e) => return Err(MenuError::STORAGE(e)),
        };

        let status = unsafe {
            c_wups::WUPSConfigItemBoolean_AddToCategoryEx(
                handle,
                id.as_ptr(),
                text.as_ptr(),
                self.default,
                current,
                Some(_callback_toggle_changed),
                trueValue.as_ptr(),
                falseValue.as_ptr(),
            )
        };
        MenuError::try_from(status)?;

        Ok(())
    }
}

extern "C" fn _callback_toggle_changed(item: *mut c_wups::ConfigItemBoolean, value: bool) {
    let _ = storage::store::<bool>(
        &unsafe { CStr::from_ptr((*item).identifier) }.to_string_lossy(),
        value,
    );
}

// endregion

// region: Range

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
    fn attach(self, handle: c_wups::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
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
            Err(storage::StorageError::NOT_FOUND) => {
                storage::store::<i32>(&self.id, self.default)?;
                self.default
            }
            Err(e) => return Err(MenuError::STORAGE(e)),
        };

        let status = unsafe {
            c_wups::WUPSConfigItemIntegerRange_AddToCategory(
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

extern "C" fn _callback_range_changed(item: *mut c_wups::ConfigItemIntegerRange, value: i32) {
    let _ = storage::store::<i32>(
        &unsafe { CStr::from_ptr((*item).identifier) }.to_string_lossy(),
        value,
    );
}

// this is overkill but should outline on how to extend API in future
/*
pub trait RangeCompatible {
    type T: storage::StorageCompatible<T: From<i32> + Into<i32>>;
    extern "C" fn callback(item: *mut c_wups::ConfigItemIntegerRange, value: i32) {
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
    fn attach(&self, handle: c_wups::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
        todo!()
    }
}
    */

// endregion

// region: Select

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
    fn attach(self, handle: c_wups::WUPSConfigCategoryHandle) -> Result<(), MenuError> {
        let text = CString::new(self.text.as_str()).unwrap();
        let id = CString::new(self.id.as_str()).unwrap();

        let strings: Result<Vec<CString>, NulError> =
            self.options.into_iter().map(|s| CString::new(s)).collect();
        let strings = strings?;

        let mut options: Vec<_> = strings
            .iter()
            .enumerate()
            .map(|(i, s)| c_wups::ConfigItemMultipleValuesPair {
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
            Err(storage::StorageError::NOT_FOUND) => {
                storage::store::<u32>(&self.id, self.default)?;
                self.default
            }
            Err(e) => return Err(MenuError::STORAGE(e)),
        };

        let status = unsafe {
            c_wups::WUPSConfigItemMultipleValues_AddToCategory(
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

extern "C" fn _callback_select_changed(item: *mut c_wups::ConfigItemMultipleValues, index: u32) {
    let _ = storage::store::<u32>(
        &unsafe { CStr::from_ptr((*item).identifier) }.to_string_lossy(),
        index,
    );
}

// endregion

/*
use crate::{bindings as c_wups, storage::StorageError};
use alloc::{ffi::CString, string::String, vec::Vec};
use core::ffi::CStr;

use wut::sync::OnceLock;

static MENU_UI: OnceLock<MenuItem> = OnceLock::new();

#[derive(Debug, Clone)]
pub enum MenuItem {
    Root {
        name: String,
        items: Vec<MenuItem>,
    },
    Label {
        text: String,
    },
    Toggle {
        text: String,
        value: bool,
        trueValue: String,
        falseValue: String,
        changed: (),
    },
    Range {
        text: String,
        value: i32,
        min: i32,
        max: i32,
        changed: (),
    },
    Select {
        text: String,
        index: i32,
        options: Vec<&'static CStr>,
        changed: (),
    },
    // Category,
}

*/

/*

pub struct MenuUI;

impl MenuUI {
    pub fn new(ui: MenuItem) -> Result<(), StorageError> {
        if MENU_UI.get().is_some() {
            return Err(StorageError::MENU_UI_ERROR(MenuError::ALREADY_INITIALIZED));
        }

        if let MenuItem::Root { ref name, .. } = ui {
            let name = CString::new(name.clone()).unwrap();
            let opt = c_wups::WUPSConfigAPIOptionsV1 {
                name: name.as_ptr(),
            };
            let status =
                unsafe { c_wups::WUPSConfigAPI_Init(opt, Some(menu_open), Some(menu_close)) };
            MenuError::try_from(status)?;

            let _ = MENU_UI.set(ui);
            Ok(())
        } else {
            Err(StorageError::MENU_UI_ERROR(MenuError::MUST_CONTAIN_ROOT))
        }
    }
}

unsafe extern "C" fn menu_open(
    root: c_wups::WUPSConfigCategoryHandle,
) -> c_wups::WUPSConfigAPICallbackStatus::Type {
    use c_wups::WUPSConfigAPICallbackStatus::{
        WUPSCONFIG_API_CALLBACK_RESULT_ERROR as ERROR,
        WUPSCONFIG_API_CALLBACK_RESULT_SUCCESS as SUCCESS,
    };
    use c_wups::WUPSConfigAPIStatus as Status;

    wut::bindings::WHBLogUdpInit();

    let ui = if let Some(ui) = MENU_UI.get() {
        ui
    } else {
        return SUCCESS;
    };

    let mut status = Status::Type::default();
    if let MenuItem::Root { items, .. } = ui {
        for item in items {
            match item {
                MenuItem::Label { text } => {
                    let text = CString::new(text.as_str()).unwrap();

                    status = c_wups::WUPSConfigItemStub_AddToCategory(root, text.as_ptr());
                }
                MenuItem::Toggle {
                    text,
                    value,
                    trueValue,
                    falseValue,
                    changed,
                } => {
                    let text = CString::new(text.as_str()).unwrap();
                    let trueValue = CString::new(trueValue.as_str()).unwrap();
                    let falseValue = CString::new(falseValue.as_str()).unwrap();

                    status = c_wups::WUPSConfigItemBoolean_AddToCategoryEx(
                        root,
                        c"toggle".as_ptr(),
                        text.as_ptr(),
                        Default::default(),
                        *value,
                        Some(callback_boolean),
                        trueValue.as_ptr(),
                        falseValue.as_ptr(),
                    );
                }
                MenuItem::Range {
                    text,
                    value,
                    min,
                    max,
                    changed,
                } => {
                    let text = CString::new(text.as_str()).unwrap();

                    status = c_wups::WUPSConfigItemIntegerRange_AddToCategory(
                        root,
                        c"range".as_ptr(),
                        text.as_ptr(),
                        Default::default(),
                        *value,
                        *min,
                        *max,
                        None,
                    );
                }
                MenuItem::Select {
                    text,
                    index,
                    options,
                    changed,
                } => {
                    let text = CString::new(text.as_str()).unwrap();
                    let mut values = options
                        .iter()
                        .enumerate()
                        .map(|(i, s)| c_wups::ConfigItemMultipleValuesPair {
                            value: i as u32,
                            valueName: s.as_ptr(),
                        })
                        .collect::<Vec<_>>();

                    status = c_wups::WUPSConfigItemMultipleValues_AddToCategory(
                        root,
                        c"select".as_ptr(),
                        text.as_ptr(),
                        Default::default(),
                        *index,
                        values.as_mut_ptr(),
                        values.len() as i32,
                        None,
                    );
                }
                MenuItem::Root { .. } => return ERROR,
                _ => return ERROR,
            }

            if status != Status::WUPSCONFIG_API_RESULT_SUCCESS {
                break;
            }
        }

        wut::
        bindings::WHBLogUdpDeinit();

        SUCCESS
    } else {
        ERROR
    }
}

unsafe extern "C" fn menu_close() {}

unsafe extern "C" fn callback_boolean(config_item: *mut c_wups::ConfigItemBoolean, value: bool) {
    wut::bindings::WHBLogUdpInit();

    wut::println!("{:?}, {:?}", *config_item, value);

    wut::bindings::WHBLogUdpDeinit();
}
*/
