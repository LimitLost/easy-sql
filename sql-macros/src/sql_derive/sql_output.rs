use easy_macros::{
    anyhow::{self, Context},
    helpers::{context, parse_macro_input},
    macros::{always_context, get_attributes},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::{self, punctuated::Punctuated},
};

use crate::{easy_lib_crate, easy_macros_helpers_crate, sql_crate};

pub fn sql_output_base(
    item_name: &syn::Ident,
    fields: &Punctuated<syn::Field, syn::Token![,]>,
    table: &TokenStream,
) -> TokenStream {
    let field_names = fields.iter().map(|field| field.ident.as_ref().unwrap());
    let field_names2 = field_names.clone();
    let field_names_str = field_names.clone().map(|field| field.to_string());
    let field_names_str2 = field_names_str.clone();

    let sql_crate = sql_crate();
    let easy_lib_crate = easy_lib_crate();
    let easy_macros_helpers_crate = easy_macros_helpers_crate();

    let context_strs = fields.iter().map(|field| {
        format!(
            "Getting field `{}` with type {} for struct `{}`",
            field.ident.as_ref().unwrap(),
            field.ty.to_token_stream(),
            item_name
        )
    });

    quote! {
        impl #sql_crate::SqlOutput<#table, #sql_crate::Row> for #item_name {
            fn sql_to_query<'a>(sql: &'a #sql_crate::Sql<'a>) -> #easy_lib_crate::anyhow::Result<#sql_crate::QueryData<'a>> {
                #sql_crate::never::never_fn(|| {
                    //Check for validity
                    let table_instance = #sql_crate::never::never_any::<#table>();

                    Self {
                        #(#field_names: table_instance.#field_names),*
                    }
                });

                let requested_columns = vec![
                    #(
                        #sql_crate::RequestedColumn {
                            name: #field_names_str.to_owned(),
                            alias: None,
                        }
                    ),*
                ];

                sql.query_output(requested_columns)
            }
            fn convert<'r>(data: #sql_crate::Row) -> #easy_lib_crate::anyhow::Result<Self> {
                use #easy_lib_crate::anyhow::Context;
                use #easy_macros_helpers_crate::context;

                Ok(Self {
                    #(
                        #field_names2: <#sql_crate::Row as #sql_crate::SqlxRow>::try_get(&data, #field_names_str2).with_context(
                            context!(#context_strs),
                        )?
                    ),*
                })
            }
        }

    }
}

#[always_context]
pub fn sql_output(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
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

    #[no_context]
    let table = table.with_context(context!("Table attribute is required"))?;

    Ok(sql_output_base(&item_name, &fields, &table).into())
}
