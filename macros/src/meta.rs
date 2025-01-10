use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Result, Token,
};

pub struct WupsMeta {
    pub name: Ident,
    pub value: LitStr,
}

impl WupsMeta {
    pub fn prefixed(&self) -> Ident {
        Ident::new(&format!("wups_meta_{}", self.name), self.name.span())
    }
}

impl Parse for WupsMeta {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        _ = input.parse::<Token![,]>()?;
        let value = input.parse()?;

        Ok(Self { name, value })
    }
}
