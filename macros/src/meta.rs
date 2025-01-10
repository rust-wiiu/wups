use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Result, Token,
};

pub struct WupsMeta {
    pub name: Ident,
    pub value: LitStr,
    pub prefixed: Ident,
}

impl Parse for WupsMeta {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        _ = input.parse::<Token![,]>()?;
        let value = input.parse()?;

        let prefixed = Ident::new(&format!("wups_meta_{}", &name), name.span());

        Ok(Self {
            name,
            value,
            prefixed,
        })
    }
}
