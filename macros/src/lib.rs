#![allow(non_snake_case)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, spanned::Spanned};

// region: wups_meta

struct Meta {
    name: syn::Ident,
    value: syn::Expr,
}

impl syn::parse::Parse for Meta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        _ = input.parse::<syn::Token![,]>()?;
        let value = input.parse()?;
        Ok(Self { name, value })
    }
}

#[proc_macro]
pub fn wups_meta(input: TokenStream) -> TokenStream {
    let Meta { name, value } = parse_macro_input!(input as Meta);

    let value_str = match &value {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => lit_str.value(),
        syn::Expr::Macro(syn::ExprMacro { mac, .. }) if mac.path.is_ident("env") => {
            let tokens = mac.tokens.to_string();
            let env_var = tokens.trim_matches(|c| c == '(' || c == ')' || c == '"' || c == '\'');
            std::env::var(env_var).unwrap_or_default()
        }
        _ => {
            let evaluated_value: syn::Lit = syn::parse_quote!(#value);
            match evaluated_value {
                syn::Lit::Str(lit_str) => lit_str.value(),
                syn::Lit::Int(lit_int) => lit_int.base10_digits().to_string(),
                syn::Lit::Float(lit_float) => lit_float.base10_digits().to_string(),
                syn::Lit::Bool(lit_bool) => lit_bool.value.to_string(),
                _ => quote!(#value).to_string(),
            }
        }
    };

    let value = syn::LitByteStr::new(
        format!("{}={}\0", name.to_string(), value_str).as_bytes(),
        value.span(),
    );
    let len = value.value().len();
    let name = syn::Ident::new(&format!("wups_meta_{}", name.to_string()), name.span());

    TokenStream::from(quote! {
        #[used]
        #[no_mangle]
        #[link_section = ".wups.meta"]
        #[allow(non_upper_case_globals)]
        static #name: [u8; #len] = *#value;
    })
}

// endregion

// region: wups_hook_ex

struct Hook {
    hook_type: syn::Ident,
    hook_target: syn::Ident,
}

impl syn::parse::Parse for Hook {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let hook_type = input.parse()?;
        _ = input.parse::<syn::Token![,]>()?;
        let hook_target = input.parse()?;
        Ok(Self {
            hook_type,
            hook_target,
        })
    }
}

#[proc_macro]
pub fn wups_hook_ex(input: TokenStream) -> TokenStream {
    let Hook {
        hook_type,
        hook_target,
    } = parse_macro_input!(input as Hook);

    let hook_type: syn::ExprPath = syn::parse_str(&format!(
        "wups::bindings::wups_loader_hook_type_t::WUPS_LOADER_HOOK_{}",
        hook_type.to_string()
    ))
    .unwrap();

    let name = syn::Ident::new(&format!("wups_hooks_{}", hook_target), hook_target.span());

    TokenStream::from(quote! {
        #[used]
        #[no_mangle]
        #[link_section = ".wups.hooks"]
        #[allow(non_upper_case_globals)]
        static #name: wups::bindings::wups_loader_hook_t = wups::bindings::wups_loader_hook_t {
            type_: #hook_type,
            target: #hook_target as *const ::core::ffi::c_void
        };
    })
}

// endregion

#[proc_macro]
pub fn WUPS_PLUGIN_NAME(input: TokenStream) -> TokenStream {
    let mut stream = TokenStream::new();

    // region: WUPS_META name, description, version, license, buildtimestamp

    let name = parse_macro_input!(input as syn::LitStr);
    let buildtimestamp = chrono::Utc::now().format("%b %d %Y %H:%M:%S").to_string(); // format as: "Feb 12 1996 23:59:01"

    stream.extend(TokenStream::from(quote! {
        wups_meta!(name, #name);
        wups_meta!(description, env!("CARGO_PKG_DESCRIPTION"));
        wups_meta!(version, env!("CARGO_PKG_VERSION"));
        wups_meta!(author, env!("CARGO_PKG_AUTHORS"));
        wups_meta!(license, env!("CARGO_PKG_LICENSE"));
        wups_meta!(buildtimestamp, #buildtimestamp);
    }));

    // endregion

    // region: WUPS_META(wups, WUPS_VERSION_STR)

    stream.extend(TokenStream::from(quote! {
        #[used]
        #[no_mangle]
        #[link_section = ".wups.meta"]
        #[allow(non_upper_case_globals)]
        static wups_meta_wups: [u8; wups::bindings::WUPS_VERSION_STR.to_bytes_with_nul().len()+5] = {
            let bytes = wups::bindings::WUPS_VERSION_STR.to_bytes_with_nul();
            const N: usize = wups::bindings::WUPS_VERSION_STR.to_bytes_with_nul().len() + 5;
            let mut array = [0u8; N];
            array[0] = b'w';
            array[1] = b'u';
            array[2] = b'p';
            array[3] = b's';
            array[4] = b'=';
            let mut i = 5;
            while i < N {
                array[i] = bytes[i-5];
                i += 1;
            }
            array
        };
    }));

    // endregion

    // region: WUPS_USE_WUT_MALLOC

    stream.extend(TokenStream::from(quote! {
        extern "C" {
            fn __init_wut_malloc();
            fn __fini_wut_malloc();
        }
        #[no_mangle]
        unsafe extern "C" fn on_init_wut_malloc() {
            __init_wut_malloc();
        }
        #[no_mangle]
        unsafe extern "C" fn on_fini_wut_malloc() {
            __fini_wut_malloc();
        }

        wups_hook_ex!(INIT_WUT_MALLOC, on_init_wut_malloc);
        wups_hook_ex!(FINI_WUT_MALLOC, on_fini_wut_malloc);
    }));

    // endregion

    // region: WUPS_USE_WUT_SOCKETS

    stream.extend(TokenStream::from(quote! {
        extern "C" {
            // #[linkage="weak"]
            fn __init_wut_socket();
            // #[linkage="weak"]
            fn __fini_wut_socket();
        }
        #[no_mangle]
        unsafe extern "C" fn on_init_wut_sockets() {
            if __init_wut_socket as *const () != ::core::ptr::null() {
                __init_wut_socket();
            }
        }
        #[no_mangle]
        unsafe extern "C" fn on_fini_wut_sockets() {
            if __fini_wut_socket as *const () != ::core::ptr::null() {
                __fini_wut_socket();
            }
        }

        wups_hook_ex!(INIT_WUT_SOCKETS, on_init_wut_sockets);
        wups_hook_ex!(FINI_WUT_SOCKETS, on_fini_wut_sockets);
    }));

    // endregion

    // region: WUPS_USE_WUT_NEWLIB

    stream.extend(TokenStream::from(quote! {
        extern "C" {
            fn __init_wut_newlib();
            fn __fini_wut_newlib();
        }
        #[no_mangle]
        unsafe extern "C" fn on_init_wut_newlib() {
            __init_wut_newlib();
        }
        #[no_mangle]
        unsafe extern "C" fn on_fini_wut_newlib() {
            __fini_wut_newlib();
        }

        wups_hook_ex!(INIT_WUT_NEWLIB, on_init_wut_newlib);
        wups_hook_ex!(FINI_WUT_NEWLIB, on_fini_wut_newlib);
    }));

    // endregion

    // region: WUPS_USE_WUT_STDCPP

    stream.extend(TokenStream::from(quote! {
        extern "C" {
            fn __init_wut_stdcpp();
            fn __fini_wut_stdcpp();
        }
        #[no_mangle]
        unsafe extern "C" fn on_init_wut_stdcpp() {
            __init_wut_stdcpp();
        }
        #[no_mangle]
        unsafe extern "C" fn on_fini_wut_stdcpp() {
            __fini_wut_stdcpp();
        }

        wups_hook_ex!(INIT_WUT_STDCPP, on_init_wut_stdcpp);
        wups_hook_ex!(FINI_WUT_STDCPP, on_fini_wut_stdcpp);
    }));
    // endregion

    // region: WUPS_USE_WUT_DEVOPTAB

    stream.extend(TokenStream::from(quote! {
        extern "C" {
            fn __init_wut_devoptab();
            fn __fini_wut_devoptab();
        }
        #[no_mangle]
        unsafe extern "C" fn on_init_wut_devoptab() {
            __init_wut_stdcpp();
        }
        #[no_mangle]
        unsafe extern "C" fn on_fini_wut_devoptab() {
            __fini_wut_stdcpp();
        }

        wups_hook_ex!(INIT_WUT_DEVOPTAB, on_init_wut_devoptab);
        wups_hook_ex!(FINI_WUT_DEVOPTAB, on_fini_wut_devoptab);
    }));

    // endregion

    // region: WUPS___INIT_WRAPPER & WUPS___FINI_WRAPPER

    stream.extend(TokenStream::from(quote! {
        extern "C" {
            fn __init();
            fn __fini();
        }
        #[no_mangle]
        unsafe extern "C" fn __init_wrapper() {
            if wups::bindings::wut_get_thread_specific(0x13371337) != 0x42424242 {
                wups::bindings::OSFatal(wups_meta_info_linking_order.as_ptr() as *const _);
            }
            __init();
        }
        #[no_mangle]
        unsafe extern "C" fn __fini_wrapper() {
            __fini();
        }
    }));

    stream.extend(wups_hook_ex(
        quote! {
            INIT_WRAPPER, __init_wrapper
        }
        .into(),
    ));

    stream.extend(wups_hook_ex(
        quote! {
            FINI_WRAPPER, __fini_wrapper
        }
        .into(),
    ));

    // endregion

    // region: WUPS_INIT_CONFIG_FUNCTIONS

    stream.extend(TokenStream::from(quote! {
        extern "C" {
            fn WUPSConfigAPI_InitLibrary_Internal(
                args: wups::bindings::wups_loader_init_config_args_t,
            ) -> wups::bindings::WUPSConfigAPIStatus::Type;
        }

        #[no_mangle]
        unsafe extern "C" fn wups_init_config_functions(args: wups::bindings::wups_loader_init_config_args_t) {
            WUPSConfigAPI_InitLibrary_Internal(args);
        }

        wups_hook_ex!(INIT_CONFIG, wups_init_config_functions);

    }));

    // endregion

    // region: WUPS_USE_STORAGE

    stream.extend(wups_meta(
        quote! {
            storage_id, #name
        }
        .into(),
    ));

    stream.extend(TokenStream::from(quote! {
        unsafe extern "C" fn init_storage(args: wups::bindings::wups_loader_init_storage_args_t_) {
            let s = wups::bindings::WUPSStorageAPI_InitInternal(args);
            if s != wups::bindings::WUPSStorageError::WUPS_STORAGE_ERROR_SUCCESS {
                panic!("Storage initialization failed: {:?}\n{:?}", wups::storage::StorageError::try_from(s), args.version as i32);
            }
        }

        wups_hook_ex!(INIT_STORAGE, init_storage);

    }));

    // endregion

    // region: wups_meta_plugin_name

    let plugin_name = syn::LitByteStr::new(format!("{}\0", name.value()).as_bytes(), name.span());
    let len = plugin_name.value().len();

    stream.extend(TokenStream::from(quote! {
        #[used]
        #[no_mangle]
        #[link_section = ".wups.meta"]
        #[allow(non_upper_case_globals)]
        pub static wups_meta_plugin_name: [u8; #len] = *#plugin_name;
    }));

    let plugin_name = syn::LitStr::new(name.value().as_str(), name.span());
    stream.extend(TokenStream::from(quote! {
        pub static PLUGIN_NAME: &str = #plugin_name;
    }));

    // endregion

    // region: wups_meta_info_dump

    let info_dump = syn::LitStr::new(
        &format!(
            "(plugin: {}; wups: {}; buildtime: {})",
            name.value(),
            "0.8.1", // wups::bindings::WUPS_VERSION_STR (idk how to extract here)
            buildtimestamp
        ),
        name.span(),
    );

    stream.extend(wups_meta(
        quote! {
            info_dump, #info_dump
        }
        .into(),
    ));

    // endregion

    // region: wups_meta_info_linking_order

    let linking_order = syn::LitStr::new(
        &format!(
            "Loading \"{}\" failed.\nFunction \"wut_get_thread_specific\" returned unexpected value.\nPlease check linking order (expected \"-lwups -lwut\")",
            name.value()
        ),
        name.span(),
    );

    stream.extend(wups_meta(
        quote! {
            info_linking_order, #linking_order
        }
        .into(),
    ));

    // endregion

    stream
}

/*
#[proc_macro]
pub fn WUPS_PLUGIN_DESCRIPTION(input: TokenStream) -> TokenStream {
    let value = parse_macro_input!(input as syn::LitStr);

    TokenStream::from(quote! {
        wups_meta!(
            description, #value
        );
    })
}

#[proc_macro]
pub fn WUPS_PLUGIN_VERSION(input: TokenStream) -> TokenStream {
    let value = parse_macro_input!(input as syn::LitStr);
    wups_meta(
        quote! {
            version, #value
        }
        .into(),
    )
}

#[proc_macro]
pub fn WUPS_PLUGIN_AUTHOR(input: TokenStream) -> TokenStream {
    let value = parse_macro_input!(input as syn::LitStr);
    wups_meta(
        quote! {
            author, #value
        }
        .into(),
    )
}

#[proc_macro]
pub fn WUPS_PLUGIN_LICENSE(input: TokenStream) -> TokenStream {
    let value = parse_macro_input!(input as syn::LitStr);
    wups_meta(
        quote! {
            license, #value
        }
        .into(),
    )
}
*/
fn generate_proc_macro_attribute(
    hook_type: &str,
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let args =
        parse_macro_input!(attr with syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
            .into_iter()
            .map(|arg| {
                let path = match arg {
                    syn::Meta::Path(path) => path,
                    _ => panic!("Expected: Cafe, Console, Module, Udp"),
                };
                let ident = path.get_ident().unwrap();
                quote! { wut::logger::Channel::#ident }
            })
            .collect::<Vec<_>>();

    let input = parse_macro_input!(item as syn::ItemFn);
    let func = &input.sig.ident;
    let block = &input.block;

    let logger_init = if !args.is_empty() {
        quote! {
            let _ = wut::logger::init(#(#args),*).expect("logger init failed");
        }
    } else {
        quote! {}
    };

    let logger_deinit = if !args.is_empty() {
        quote! {
            wut::logger::deinit();
        }
    } else {
        quote! {}
    };

    let hook_type = syn::Ident::new(hook_type, hook_type.span());

    TokenStream::from(quote! {
        #[no_mangle]
        extern "C" fn #func() {
            #logger_init
            #block
            #logger_deinit
        }

        wups_hook_ex!(#hook_type, #func);
    })
}

#[proc_macro_attribute]
pub fn on_initialize(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_proc_macro_attribute("INIT_PLUGIN", attr, item)
}

#[proc_macro_attribute]
pub fn on_deinitialize(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_proc_macro_attribute("DEINIT_PLUGIN", attr, item)
}

#[proc_macro_attribute]
pub fn on_application_start(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_proc_macro_attribute("APPLICATION_STARTS", attr, item)
}

#[proc_macro_attribute]
pub fn on_release_foreground(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_proc_macro_attribute("RELEASE_FOREGROUND", attr, item)
}

#[proc_macro_attribute]
pub fn on_acquired_foreground(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_proc_macro_attribute("ACQUIRED_FOREGROUND", attr, item)
}

#[proc_macro_attribute]
pub fn on_application_request_exit(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_proc_macro_attribute("APPLICATION_REQUESTS_EXIT", attr, item)
}

#[proc_macro_attribute]
pub fn on_application_exit(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_proc_macro_attribute("APPLICATION_ENDS", attr, item)
}

#[proc_macro_attribute]
pub fn function_hook(attr: TokenStream, item: TokenStream) -> TokenStream {
    // region: Attributes

    struct Attributes {
        library: syn::Path,
        function: syn::Ident,
    }

    impl syn::parse::Parse for Attributes {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            input.parse::<syn::Ident>()?; // Expect `library`
            input.parse::<syn::Token![=]>()?; // Expect `=`
            let library: syn::Ident = input.parse()?; // Expect library name

            input.parse::<syn::Token![,]>()?; // Expect `,`
            input.parse::<syn::Ident>()?; // Expect `function`
            input.parse::<syn::Token![=]>()?; // Expect `=`
            let function: syn::Ident = input.parse()?; // Expect function name

            let library = syn::Ident::new(
                &format!("WUPS_LOADER_LIBRARY_{}", library.to_string()),
                library.span(),
            );
            let library = parse_quote! {
                ::wups::bindings::wups_loader_library_type_t::#library
            };

            Ok(Self { library, function })
        }
    }

    // endregion

    let item = parse_macro_input!(item as syn::ItemFn);
    let attr = parse_macro_input!(attr as Attributes);

    let mut stream = TokenStream::new();

    let real_func = syn::Ident::new(
        &format!("real_{}", attr.function.to_string()),
        attr.function.span(),
    );
    let signature = &item.sig.inputs;
    let output = &item.sig.output;

    stream.extend(TokenStream::from(quote! {
        #[used]
        #[no_mangle]
        #[link_section = ".data"]
        #[allow(non_upper_case_globals)]
        static mut #real_func: Option<
            unsafe extern "C" fn(#signature) #output
        > = None;
    }));

    let func = &item.sig;
    let block = &item.block;

    let wrapped_func = &attr.function;
    let wrapped_func: syn::Path = parse_quote! {
        ::wut::bindings::#wrapped_func
    };

    stream.extend(TokenStream::from(quote! {
        #[no_mangle]
        extern "C" #func {
            use #real_func as hooked;

            #block
        }

        const _: () = {
            let _ = #wrapped_func as unsafe extern "C" fn(#signature) #output;
        };
    }));

    let library = attr.library;
    let target: &syn::Ident = &item.sig.ident;
    let hooked_func_name = syn::LitByteStr::new(
        format!("{}\0", attr.function.to_string()).as_bytes(),
        attr.function.span(),
    );
    let my_func_name =
        syn::LitByteStr::new(format!("{}\0", &item.sig.ident).as_bytes(), item.span());
    let loader_name = syn::Ident::new(
        &format!("wups_loader_{}", target.to_string()),
        target.span(),
    );

    stream.extend(TokenStream::from(quote! {
        #[used]
        #[no_mangle]
        #[link_section = ".wups.load"]
        #[allow(non_upper_case_globals)]
        static #loader_name: ::wups::bindings::wups_loader_entry_t =
            ::wups::bindings::wups_loader_entry_t {
                type_: ::wups::bindings::wups_loader_entry_type_t::WUPS_LOADER_ENTRY_FUNCTION_MANDATORY,
                _function: ::wups::bindings::wups_loader_entry_t__bindgen_ty_1 {
                    physical_address: ::core::ptr::null(),
                    virtual_address: ::core::ptr::null(),
                    name: #hooked_func_name.as_ptr() as *const _,
                    library: #library,
                    my_function_name: #my_func_name.as_ptr()  as *const _,
                    target: #target as *const ::core::ffi::c_void,
                    call_addr: ::core::ptr::addr_of!(#real_func) as *const _ as *const ::core::ffi::c_void,
                    targetProcess:
                        ::wups::bindings::WUPSFPTargetProcess::WUPS_FP_TARGET_PROCESS_GAME_AND_MENU,
                },
            };
    }));

    stream
}
