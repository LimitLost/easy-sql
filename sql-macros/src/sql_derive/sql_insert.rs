use easy_macros::{
    anyhow::{self, Context},
    helpers::{context, parse_macro_input},
    macros::{always_context, get_attributes},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::{self, parse::Parse, punctuated::Punctuated},
};

use super::ty_to_variant;

struct DefaultAttr {
    fields: Punctuated<syn::Ident, syn::Token![,]>,
}

#[always_context]
impl Parse for DefaultAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fields = Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated(&input)?;
        Ok(DefaultAttr { fields })
    }
}

#[always_context]
pub fn sql_insert_base(
    item_name: &syn::Ident,
    fields: &Punctuated<syn::Field, syn::Token![,]>,
    table: &TokenStream,
    defaults: Vec<syn::Ident>,
) -> anyhow::Result<TokenStream> {
    let field_names = fields.iter().map(|field| field.ident.as_ref().unwrap());
    let field_names_str = field_names.clone().map(|field| field.to_string());

    let mut insert_values = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        insert_values.push(ty_to_variant(
            field_name.to_token_stream(),
            &field.ty,
            &quote! {easy_lib::sql},
            true,
        )?);
    }

    Ok(quote! {
        impl easy_lib::sql::SqlInsert<#table> for #item_name {
            fn insert_columns() -> Vec<String> {
                easy_lib::sql::never::never_fn(|| {
                    //Check for validity
                    let this_instance = easy_lib::sql::never::never_any::<Self>();

                    #table {
                        #(
                            #defaults: Default::default(),
                        )*
                        #(
                        #field_names: this_instance.#field_names,
                        )*
                    }
                });
                vec![
                    #(
                        #field_names_str.to_string(),
                    )*
                ]
            }

            fn insert_values(&self) -> anyhow::Result<Vec<Vec<easy_lib::sql::SqlValueMaybeRef<'_>>>> {
                vec![vec![
                    #(#insert_values)*
                ]]
            }
        }

    })
}

#[always_context]
pub fn sql_insert(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let item = parse_macro_input!(item as syn::ItemStruct);
    let item_name = &item.ident;

    let fields = match item.fields {
        syn::Fields::Named(fields_named) => fields_named.named,
        syn::Fields::Unnamed(_) => {
            anyhow::bail!("Unnamed struct fields is not supported")
        }
        syn::Fields::Unit => anyhow::bail!("Unit struct is not supported"),
    };

    let mut table = None;

    for attr in get_attributes!(item, #[sql(table = __unknown__)]) {
        if table.is_some() {
            anyhow::bail!("Only one table attribute is allowed");
        }
        table = Some(attr);
    }
    panic!("File: {} Line: {}", file!(), line!());

    #[no_context]
    let table = table.with_context(context!("Table attribute is required"))?;

    let mut defaults = Vec::new();

    for attr in get_attributes!(item, #[sql(default = __unknown__)]) {
        let parsed: DefaultAttr = syn::parse2(
            #[context(tokens)]
            attr.clone(),
        )?;
        defaults.extend(parsed.fields.into_iter());
    }

    sql_insert_base(&item_name, &fields, &table, defaults).map(|e| e.into())
}
