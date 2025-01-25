#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate alloc;
extern crate flagset;
extern crate thiserror;

extern crate wups_macros;
pub use wups_macros::*;

pub mod bindings;
pub mod config;
pub mod storage;

pub mod prelude {
    pub use crate::config::{Attachable, ConfigMenu};
}

#[cfg(feature = "panic_handler")]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    use alloc::format;
    let msg = if let Some(location) = info.location() {
        format!(
            "Panic!\n\n{}\n\n[{} : Ln {}, Col {}]\0",
            info.message(),
            location
                .file()
                .strip_prefix("/home/gerald/Projects/Rust/rust-wiiu/")
                .unwrap_or(location.file()),
            location.line(),
            location.column()
        )
    } else {
        format!("Panic!\n\n{}\0", info.message())
    };

    unsafe {
        crate::bindings::OSFatal(msg.as_ptr() as *const _);
    }
    loop {}
}
