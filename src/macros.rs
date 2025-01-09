#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Macros

// Section attribute macro
#[macro_export]
macro_rules! wups_section {
    ($x:expr) => {
        #[link_section = concat!(".wups.", $x)]
    };
}

// Meta attribute macro
#[macro_export]
macro_rules! wups_meta {
    ($id:ident, $value:expr) => {
        $crate::wups_section!("meta");
        #[used] // Ensures the static is kept even if unused
        static $id: &[u8] = concat!(stringify!($id), "=", $value).as_bytes();
    };
}
