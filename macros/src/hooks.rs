use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_str, Block, ExprPath, Ident, ItemFn, Result, Token,
};

pub struct WupsHooks {
    pub init: WupsHookInner,
    pub fini: WupsHookInner,
}

impl Parse for WupsHooks {
    fn parse(input: ParseStream) -> Result<Self> {
        let hook = input.parse::<Ident>()?;

        let init = WupsHookInner::new(hook.clone(), "init")?;
        let fini = WupsHookInner::new(hook, "fini")?;

        Ok(Self { init, fini })
    }
}

pub struct WupsHookInner {
    pub extern_: Ident,
    pub wrap: Ident,
    pub hook: ExprPath,
    pub static_: Ident,
}

impl WupsHookInner {
    pub fn to_tokens(&self) -> proc_macro::TokenStream {
        let WupsHookInner {
            extern_,
            wrap,
            hook,
            static_,
        } = &self;
        TokenStream::from(quote! {
            extern "C" {
                fn #extern_();
            }
            #[no_mangle]
            pub unsafe extern "C" fn #wrap() {
                #extern_();
            }

            #[used]
            #[no_mangle]
            #[link_section = ".wups.hooks"]
            #[allow(non_upper_case_globals)]
            static #static_: WupsLoaderHook = WupsLoaderHook {
                hook_type: #hook,
                target: #wrap as *const ()
            };
        })
    }
}

impl WupsHookInner {
    fn new(hook: Ident, prefix: &str) -> Result<Self> {
        let extern_ = Ident::new(&format!("__{}_wut_{}", prefix, &hook), hook.span());

        let wrap = Ident::new(&format!("on_{}_wut_{}", prefix, &hook), hook.span());

        let hook: ExprPath = parse_str(&format!(
            "wups_loader_hook_type_t::WUPS_LOADER_HOOK_{}_WUT_{}",
            prefix.to_uppercase(),
            &hook.to_string().to_uppercase()
        ))?;

        let static_ = Ident::new(&format!("wups_hooks_{}", &wrap), wrap.span());

        Ok(Self {
            extern_,
            wrap,
            hook,
            static_,
        })
    }
}
