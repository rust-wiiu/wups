#![no_std]

pub use wups_core::*;
pub use wups_macros as macros;
pub use wups_sys as sys;

pub mod prelude {
    pub use wups_core::config::{Attachable, ConfigMenu};
    pub use wups_macros::WUPS_PLUGIN_NAME;
}
