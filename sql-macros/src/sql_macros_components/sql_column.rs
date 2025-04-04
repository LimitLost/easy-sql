use easy_macros::{
    proc_macro2::TokenStream,
    quote::{quote, quote_spanned},
    syn::{self, parse::Parse, spanned::Spanned},
};
#[derive(Debug)]
pub enum SqlColumn {
    SpecificTableColumn(syn::Path, syn::Ident),
    Column(syn::Ident),
}

impl SqlColumn {
    pub fn into_tokens_with_checks(self, checks: &mut Vec<TokenStream>) -> TokenStream {
        match self {
            SqlColumn::SpecificTableColumn(path, ident) => {
                checks.push(quote_spanned! {path.span()=>
                    fn has_table<T:easy_lib::easy_sql::HasTable<#path>>(test:&T){}
                    has_table(&___t___);
                    //TODO "RealColumns" trait with type leading to the struct with actual database columns
                    let mut table_instance = easy_lib::never::never_any::<#path>();
                    let _ = table_instance.#ident;
                });

                let ident_str = ident.to_string();

                quote_spanned! {path.span()=>
                    format!("{}.{}",<#path as SqlTable>::table_name(), #ident_str)
                }
            }
            SqlColumn::Column(ident) => {
                checks.push(quote! {
                        let _ = ___t___.#ident;
                });

                let ident_str = ident.to_string();
                quote! {#ident_str}
            }
        }
    }
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
