use easy_macros::{
    anyhow::{self, Context},
    helpers::{context, parse_macro_input},
    macros::get_attributes,
    quote::{ToTokens, quote},
    syn,
};

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

    let field_names = fields.iter().map(|field| field.ident.as_ref().unwrap());
    let field_names2 = field_names.clone();
    let field_names_str = field_names.clone().map(|field| field.to_string());
    let field_names_str2 = field_names_str.clone();

    let context_strs = fields.iter().map(|field| {
        format!(
            "Getting field `{}` with type {} for struct `{}`",
            field.ident.as_ref().unwrap(),
            field.ty.to_token_stream(),
            item_name
        )
    });

    let mut table = None;

    for attr in get_attributes!(item, #[sql(table = __unknown__)]) {
        if table.is_some() {
            anyhow::bail!("Only one table attribute is allowed");
        }
        table = Some(attr);
    }

    let table = table.with_context(context!("Table attribute is required"))?;

    Ok(quote! {
        impl easy_lib::sql::SqlOutput<#table, easy_lib::sql::Row> for #item_name {
            fn sql_to_query<'a>(sql: &'a easy_lib::sql::Sql<'a>) -> easy_lib::anyhow::Result<easy_lib::sql::QueryData<'a>> {
                easy_lib::sql::never::never_fn(|| {
                    //Check for validity
                    let table_instance = easy_lib::sql::never::never_any::<#table>();

                    Self {
                        #(#field_names: table_instance.#field_names),*
                    }
                });

                let requested_columns = vec![
                    #(
                        easy_lib::sql::RequestedColumn {
                            name: #field_names_str.to_owned(),
                            alias: None,
                        }
                    ),*
                ];

                sql.query_output(requested_columns)
            }
            fn convert<'r>(data: easy_lib::sql::Row) -> easy_lib::anyhow::Result<Self> {
                use easy_lib::anyhow::Context;
                use easy_macros::helpers::context;

                Ok(Self {
                    #(
                        #field_names2: <easy_lib::sql::Row as easy_lib::sql::SqlxRow>::try_get(&data, #field_names_str2).with_context(
                            context!(#context_strs),
                        )?
                    ),*
                })
            }
        }

    }.into())
}
