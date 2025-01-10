#![allow(non_snake_case)]

mod meta;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn wups_meta(input: TokenStream) -> TokenStream {
    let m = parse_macro_input!(input as meta::WupsMeta);
    let prefixed = m.prefixed();
    let meta::WupsMeta { name, value } = m;

    let expanded = quote! {
        #[used]
        #[no_mangle]
        #[link_section = ".wups.meta"]
        #[allow(non_upper_case_globals)]
        static #prefixed: &::core::ffi::CStr = unsafe {
            core::ffi::CStr::from_bytes_with_nul_unchecked(concat!(
                stringify!(#name),
                "=",
                #value,
                "\0"
            ).as_bytes())
        };
    };

    expanded.into()
}

#[proc_macro]
pub fn WUPS_PLUGIN_AUTHOR(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let author_meta = quote! {
        author, #input_str
    };
    wups_meta(author_meta.into())
}
