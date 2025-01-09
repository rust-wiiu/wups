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
pub mod macros;

pub mod prelude {
    pub use crate::{wups_meta, wups_section};
}

pub fn wups() {}
