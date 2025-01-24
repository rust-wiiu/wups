// ui

// this is quite cool but overkill for now: https://github.com/dkosmari/libwupsxx

pub mod glyphs;

use crate::{bindings as c_wups, storage::StorageError};
use alloc::{ffi::CString, string::String, vec::Vec};
use core::ffi::CStr;
use thiserror::Error;
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

// region: MenuError

#[derive(Debug, Error, Clone)]
pub enum MenuError {
    #[error("Unknown error")]
    UNKNOWN(c_wups::WUPSConfigAPIStatus::Type),
    #[error("The base of the UI must be a root node.")]
    MUST_CONTAIN_ROOT,
    #[error("The menu UI can only be initialized once.")]
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
}

impl TryFrom<c_wups::WUPSConfigAPICallbackStatus::Type> for MenuError {
    type Error = Self;
    fn try_from(value: c_wups::WUPSConfigAPICallbackStatus::Type) -> Result<Self, Self::Error> {
        use c_wups::WUPSConfigAPIStatus as E;

        match value {
            E::WUPSCONFIG_API_RESULT_SUCCESS => Ok(Self::UNKNOWN(E::WUPSCONFIG_API_RESULT_SUCCESS)),
            E::WUPSCONFIG_API_RESULT_INVALID_PLUGIN_IDENTIFIER => Err(Self::INVALID_ARGUMENT),
            E::WUPSCONFIG_API_RESULT_OUT_OF_MEMORY => Err(Self::OUT_OF_MEMORY),
            E::WUPSCONFIG_API_RESULT_NOT_FOUND => Err(Self::NOT_FOUND),
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

        wut::bindings::WHBLogUdpDeinit();

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
