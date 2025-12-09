use ::{
    proc_macro2::TokenStream,
    quote::{quote, quote_spanned},
    syn::{self, Token, parse::Parse, punctuated::Punctuated, spanned::Spanned},
};
use easy_macros::always_context;
#[derive(Debug, Clone)]
pub enum Column {
    SpecificTableColumn(Punctuated<syn::Ident, Token![::]>, syn::Ident),
    Column(syn::Ident),
}

#[always_context]
impl Column {
    pub fn into_tokens_with_checks(
        self,
        checks: &mut Vec<TokenStream>,
        sql_crate: &TokenStream,
    ) -> TokenStream {
        match self {
            Column::SpecificTableColumn(path, ident) => {
                checks.push(quote_spanned! {path.span()=>
                    fn has_table<T:#sql_crate::HasTable<#path>>(test:&T){}
                    has_table(&___t___);
                    //TODO "RealColumns" trait with type leading to the struct with actual database columns
                    let mut table_instance = #sql_crate::never::never_any::<#path>();
                    let _ = table_instance.#ident;
                });

                let ident_str = ident.to_string();

                quote_spanned! {path.span()=>
                    format!("{}.{}",<#path as #sql_crate::Table>::table_name(), #ident_str)
                }
            }
            Column::Column(ident) => {
                checks.push(quote! {
                        let _ = ___t___.#ident;
                });

                let ident_str = ident.to_string();
                quote! {#ident_str.to_string()}
            }
        }
    }

    pub fn into_query_string(
        self,
        checks: &mut Vec<TokenStream>,
        sql_crate: &TokenStream,
        format_args: &mut Vec<TokenStream>,
    ) -> String {
        match self {
            Column::SpecificTableColumn(path, ident) => {
                checks.push(quote_spanned! {path.span()=>
                    fn has_table<T:#sql_crate::HasTable<#path>>(test:&T){}
                    has_table(&___t___);
                    //TODO "RealColumns" trait with type leading to the struct with actual database columns
                    let mut table_instance = #sql_crate::never::never_any::<#path>();
                    let _ = table_instance.#ident;
                });

                let ident_str = ident.to_string();

                format_args.push(quote! {<#path as #sql_crate::Table>::table_name()});

                format!(
                    "{{_easy_sql_d}}{{}}{{_easy_sql_d}}.{{_easy_sql_d}}{}{{_easy_sql_d}}",
                    ident_str
                )
                .to_string()
            }
            Column::Column(ident) => {
                checks.push(quote! {
                        let _ = ___t___.#ident;
                });

                format!("{{_easy_sql_d}}{}{{_easy_sql_d}}", ident.to_string())
            }
        }
    }
}

#[always_context]
impl Parse for Column {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path_or_ident: Punctuated<syn::Ident, Token![::]> =
            Punctuated::parse_separated_nonempty(input)?;
        let lookahead2 = input.lookahead1();
        if lookahead2.peek(syn::Token![.]) {
            input.parse::<syn::Token![.]>()?;
            let ident: syn::Ident = input.parse()?;
            return Ok(Column::SpecificTableColumn(path_or_ident, ident));
        } else if let Some(ident) = path_or_ident.first()
            && path_or_ident.len() == 1
        {
            return Ok(Column::Column(ident.clone()));
        } else {
            return Err(input.error("Expected identifier instead of path (or dot after path)"));
        }
    }
}
