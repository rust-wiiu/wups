#![allow(non_snake_case)]

mod hooks;
mod meta;

use hooks::WupsHooks;
use meta::WupsMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro]
pub fn wups_meta(input: TokenStream) -> TokenStream {
    let WupsMeta {
        name,
        value,
        prefixed,
    } = parse_macro_input!(input as WupsMeta);

    TokenStream::from(quote! {
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
    })
}

#[proc_macro]
pub fn wups_hooks(input: TokenStream) -> TokenStream {
    let m = parse_macro_input!(input as WupsHooks);

    let mut result = TokenStream::new();

    result.extend(m.init.to_tokens());
    result.extend(m.fini.to_tokens());

    result
}

#[proc_macro]
pub fn wups_socket_hooks(_input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
        extern "C" {
            // #[linkage = "extern_weak"]
            fn __init_wut_socket();
            // #[linkage = "extern_weak"]
            fn __fini_wut_socket();
        }
        #[no_mangle]
        pub unsafe extern "C" fn on_init_wut_sockets() {
            if __init_wut_socket as *const () != ::core::ptr::null() {
                __init_wut_socket();
            }
        }
        #[no_mangle]
        pub unsafe extern "C" fn on_fini_wut_sockets() {
            if __fini_wut_socket as *const () != ::core::ptr::null() {
                __fini_wut_socket();
            }
        }

        #[used]
        #[no_mangle]
        #[link_section = ".wups.hooks"]
        #[allow(non_upper_case_globals)]
        static wups_hooks_on_init_wut_sockets: WupsLoaderHook = WupsLoaderHook {
            hook_type: wups_loader_hook_type_t::WUPS_LOADER_HOOK_INIT_WUT_SOCKETS,
            target: on_init_wut_sockets as *const ()
        };

        #[used]
        #[no_mangle]
        #[link_section = ".wups.hooks"]
        #[allow(non_upper_case_globals)]
        static wups_hooks_on_fini_wut_sockets: WupsLoaderHook = WupsLoaderHook {
            hook_type: wups_loader_hook_type_t::WUPS_LOADER_HOOK_FINI_WUT_SOCKETS,
            target: on_fini_wut_sockets as *const ()
        };
    })
}

#[proc_macro]
pub fn wups_init_hooks(_input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
        extern "C" {
            fn __init();
        }
        #[no_mangle]
        pub unsafe extern "C" fn __init_wrapper() {
            if (wups::bindings::wut_get_thread_specific(0x13371337) != 0x42424242) {
                wups::bindings::OSFatal(wups::bindings::wups_meta_info_linking_order.as_ptr());
            }
            __init();
        }

        #[used]
        #[no_mangle]
        #[link_section = ".wups.hooks"]
        #[allow(non_upper_case_globals)]
        static wups_hooks___init_wrapper: WupsLoaderHook = WupsLoaderHook {
            hook_type: wups_loader_hook_type_t::WUPS_LOADER_HOOK_INIT_WRAPPER,
            target: __init_wrapper as *const ()
        };
    })
}

#[proc_macro]
pub fn wups_fini_hooks(_input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
        extern "C" {
            fn __fini();
        }
        #[no_mangle]
        pub unsafe extern "C" fn __fini_wrapper() {
            __fini();
        }

        #[used]
        #[no_mangle]
        #[link_section = ".wups.hooks"]
        #[allow(non_upper_case_globals)]
        static wups_hooks___fini_wrapper: WupsLoaderHook = WupsLoaderHook {
            hook_type: wups_loader_hook_type_t::WUPS_LOADER_HOOK_FINI_WRAPPER,
            target: __fini_wrapper as *const ()
        };
    })
}

#[proc_macro]
pub fn wups_init_config_functions(_input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
        use wups::bindings::WUPSConfigAPIStatus;
        extern "C" {
            fn WUPSConfigAPI_InitLibrary_Internal(args: wups::bindings::wups_loader_init_config_args_t) -> WUPSConfigAPIStatus::Type;
        }
        #[no_mangle]
        pub unsafe extern "C" fn wups_init_config_functions(args: wups::bindings::wups_loader_init_config_args_t) {
            WUPSConfigAPI_InitLibrary_Internal(args);
        }

        #[used]
        #[no_mangle]
        #[link_section = ".wups.hooks"]
        #[allow(non_upper_case_globals)]
        static wups_hooks_wups_init_config_functions: WupsLoaderHook = WupsLoaderHook {
            hook_type: wups_loader_hook_type_t::WUPS_LOADER_HOOK_INIT_CONFIG,
            target: wups_init_config_functions as *const ()
        };
    })
}

#[proc_macro]
pub fn WUPS_PLUGIN_AUTHOR(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let meta = quote! {
        author, #input_str
    };
    wups_meta(meta.into())
}

#[proc_macro]
pub fn WUPS_PLUGIN_VERSION(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let meta = quote! {
        version, #input_str
    };
    wups_meta(meta.into())
}

#[proc_macro]
pub fn WUPS_PLUGIN_LICENSE(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let meta = quote! {
        license, #input_str
    };
    wups_meta(meta.into())
}

#[proc_macro]
pub fn WUPS_PLUGIN_DESCRIPTION(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let meta = quote! {
        description, #input_str
    };
    wups_meta(meta.into())
}

#[proc_macro]
pub fn WUPS_PLUGIN_CONFIG_REVISION(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let meta = quote! {
        config_revision, #input_str
    };
    wups_meta(meta.into())
}

#[proc_macro]
pub fn WUPS_PLUGIN_NAME(input: TokenStream) -> TokenStream {
    let name = input.to_string().replace("\"", "");
    let wups_version = "0.8.1";
    TokenStream::from(quote! {
        wups_meta!(name, #name);
        wups_meta!(wups, #wups_version); // TODO: extract from wups::bindings::WUPS_VERSION_STR
        wups_meta!(buildtimestamp, "DATE x TIME");
        use wups::bindings::wups_loader_hook_type_t;
        #[repr(C)]
        pub struct WupsLoaderHook {
            hook_type: wups_loader_hook_type_t::Type,
            target: *const (),
        }
        unsafe impl Sync for WupsLoaderHook {}
        //
        wups_hooks!(malloc);
        wups_hooks!(newlib);
        wups_hooks!(stdcpp);
        //
        wups_socket_hooks!();
        //
        wups_init_hooks!();
        wups_fini_hooks!();
        wups_init_config_functions!();
        //
        #[used]
        #[no_mangle]
        #[link_section = ".wups.meta"]
        #[allow(non_upper_case_globals)]
        static wups_meta_plugin_name: &::core::ffi::CStr = unsafe { ::core::ffi::CStr::from_bytes_with_nul_unchecked(concat!(#name, "\0").as_bytes()) };
        #[used]
        #[no_mangle]
        #[link_section = ".wups.meta"]
        #[allow(non_upper_case_globals)]
        static wups_meta_info_dump: &::core::ffi::CStr = unsafe { ::core::ffi::CStr::from_bytes_with_nul_unchecked(concat!(
            "(plugin: ", #name, ";wups: ", #wups_version, "; buildtime: x)\0"
        ).as_bytes()) };
        #[used]
        #[no_mangle]
        #[link_section = ".wups.meta"]
        #[allow(non_upper_case_globals)]
        static wups_meta_info_linking_order: &::core::ffi::CStr = unsafe { ::core::ffi::CStr::from_bytes_with_nul_unchecked(concat!(
            "Loading ", #name, "failed.\nFunction\"wut_get_thread_specific\"returned unexpected value.\nPlease check linking order (expected \"-lwups -lwut\")\0"
        ).as_bytes()) };
    })
}

#[proc_macro]
pub fn WUPS_USE_STORAGE(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let meta = quote! {
        storage_id, #input_str
    };

    let mut tokens = TokenStream::new();

    tokens.extend(wups_meta(meta.into()));
    tokens.extend(TokenStream::from(quote! {
        #[no_mangle]
        pub unsafe extern "C" fn init_storage(args: wups::bindings::wups_loader_init_storage_args_t) {
            wups::bindings::WUPSStorageAPI_InitInternal(args);
        }

        #[used]
        #[no_mangle]
        #[link_section = ".wups.hooks"]
        #[allow(non_upper_case_globals)]
        static wups_hooks_init_storage: WupsLoaderHook = WupsLoaderHook {
            hook_type: wups_loader_hook_type_t::WUPS_LOADER_HOOK_INIT_STORAGE,
            target: init_storage as *const ()
        };
    }));

    tokens
}

#[proc_macro_attribute]
pub fn initialize(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let func = &input.sig.ident;
    let block = &input.block;

    TokenStream::from(quote! {
        #[no_mangle]
        pub extern "C" fn #func() {
            #block
        }

        #[used]
        #[no_mangle]
        #[link_section = ".wups.hooks"]
        #[allow(non_upper_case_globals)]
        static wups_hooks_init_plugin: WupsLoaderHook = WupsLoaderHook {
            hook_type: wups_loader_hook_type_t::WUPS_LOADER_HOOK_INIT_PLUGIN,
            target: #func as *const ()
        };
    })
}

#[proc_macro_attribute]
pub fn deinitialize(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let func = &input.sig.ident;
    let block = &input.block;

    TokenStream::from(quote! {
        #[no_mangle]
        pub extern "C" fn #func() {
            #block
        }

        #[used]
        #[no_mangle]
        #[link_section = ".wups.hooks"]
        #[allow(non_upper_case_globals)]
        static wups_hooks_deinit_plugin: WupsLoaderHook = WupsLoaderHook {
            hook_type: wups_loader_hook_type_t::WUPS_LOADER_HOOK_DEINIT_PLUGIN,
            target: #func as *const ()
        };
    })
}
