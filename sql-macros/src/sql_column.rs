use easy_macros::syn::{self, parse::Parse};

pub enum SqlColumn {
    SpecificTableColumn(syn::Path, syn::Ident),
    Column(syn::Ident),
}

impl Parse for SqlColumn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path_or_ident: syn::Path = input.parse()?;
        let lookahead2 = input.lookahead1();
        if lookahead2.peek(syn::Token![.]) {
            input.parse::<syn::Token![.]>()?;
            let ident: syn::Ident = input.parse()?;
            return Ok(SqlColumn::SpecificTableColumn(path_or_ident, ident));
        } else {
            if let Some(ident) = path_or_ident.get_ident() {
                return Ok(SqlColumn::Column(ident.clone()));
            } else {
                return Err(input.error("Expected identifier instead of path (or dot after path)"));
            }
        }
    }
}
