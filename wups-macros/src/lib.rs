use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, LitStr, Token};

struct MacroInput {
    id: Ident,
    _comma: Token![,],
    value: LitStr,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            id: input.parse()?,
            _comma: input.parse()?,
            value: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn test(input: TokenStream) -> TokenStream {
    let MacroInput { id, _comma, value } = parse_macro_input!(input as MacroInput);

    // Create the prefixed identifier
    let prefixed_id = format!("wups_meta_{}", id);
    let prefixed_id = Ident::new(&prefixed_id, id.span());

    let expanded = quote! {
        static #prefixed_id: &[u8] = concat!(stringify!(#id), "=", #value).as_bytes();
    };

    TokenStream::from(expanded)
}
