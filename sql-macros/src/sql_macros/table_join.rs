use easy_macros::macros::always_context;
use syn::{self, parse::Parse};

use crate::sql_crate;
use crate::sql_macros_components::sql_expr::SqlExpr;

use crate::sql_macros_components::sql_keyword;

struct Input {
    struct_name: syn::Ident,
    main_table: syn::Path,
    joins: Vec<Join>,
}
enum Join {
    Inner { table: syn::Path, on: SqlExpr },
    Left { table: syn::Path, on: SqlExpr },
    Right { table: syn::Path, on: SqlExpr },
    Cross { table: syn::Path },
}

#[always_context]
impl Parse for Join {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(sql_keyword::inner) {
            input.parse::<sql_keyword::inner>()?;
            input.parse::<sql_keyword::join>()?;

            let table = input.parse::<syn::Path>()?;

            input.parse::<sql_keyword::on>()?;

            let on = input.parse::<SqlExpr>()?;
            Ok(Join::Inner { table, on })
        } else if lookahead.peek(sql_keyword::left) {
            input.parse::<sql_keyword::left>()?;
            input.parse::<sql_keyword::join>()?;

            let table = input.parse::<syn::Path>()?;

            input.parse::<sql_keyword::on>()?;

            let on = input.parse::<SqlExpr>()?;
            Ok(Join::Left { table, on })
        } else if lookahead.peek(sql_keyword::right) {
            input.parse::<sql_keyword::right>()?;
            input.parse::<sql_keyword::join>()?;

            let table = input.parse::<syn::Path>()?;

            input.parse::<sql_keyword::on>()?;

            let on = input.parse::<SqlExpr>()?;
            Ok(Join::Right { table, on })
        } else if lookahead.peek(sql_keyword::cross) {
            input.parse::<sql_keyword::cross>()?;
            input.parse::<sql_keyword::join>()?;

            let table = input.parse::<syn::Path>()?;

            input.parse::<sql_keyword::on>()?;
            Ok(Join::Cross { table })
        } else {
            Err(input.error("Expected join type"))
        }
    }
}

#[always_context]
impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let struct_name = input.parse::<syn::Ident>()?;
        // Separator
        input.parse::<syn::Token![|]>()?;
        let main_table = input.parse::<syn::Path>()?;
        let mut joins = Vec::new();

        while !input.is_empty() {
            let join = input.parse::<Join>()?;
            joins.push(join);
        }

        Ok(Input {
            struct_name,
            main_table,
            joins,
        })
    }
}

pub fn table_join(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as Input);

    let sql_crate = sql_crate();

    let item_name = input.struct_name;
    let main_table_struct = input.main_table;
    //input.joins

    let has_table_impls = input
        .joins
        .iter()
        .map(|join| {
            let table = match join {
                Join::Inner { table, .. } => table,
                Join::Left { table, .. } => table,
                Join::Right { table, .. } => table,
                Join::Cross { table } => table,
            };
            quote! {
                #table
            }
        })
        .collect::<Vec<_>>();

    let mut checks = Vec::new();

    let mut not_optional_joined_tables = vec![&main_table_struct];
    let mut optional_joined_tables = Vec::new();

    for join in input.joins.iter() {
        match join {
            Join::Inner { table, on: _ } => {
                not_optional_joined_tables.push(table);
            }
            Join::Left { table, on: _ } => {
                optional_joined_tables.push(table);
            }
            Join::Right { table, on: _ } => {
                //The right joined table takes priority
                optional_joined_tables.append(&mut not_optional_joined_tables);
                not_optional_joined_tables.push(table);
            }
            Join::Cross { table } => {
                not_optional_joined_tables.push(table);
            }
        }
    }

    let has_table_joined_impls = not_optional_joined_tables
        .iter()
        .map(|table| {
            quote! {
                impl #sql_crate::HasTableJoined<#table> for #item_name{
                    type MaybeOption<Y> = Y;

                    fn into_maybe_option<Y>(t: Y) -> Self::MaybeOption<Y>{
                        t
                    }
                }
            }
        })
        .chain(optional_joined_tables.iter().map(|table| {
            quote! {
                impl #sql_crate::HasTableJoined<#table> for #item_name{
                    type MaybeOption<Y> = Option<Y>;

                    fn into_maybe_option<Y>(t: Y) -> Self::MaybeOption<Y>{
                        Some(t)
                    }
                }
            }
        }))
        .collect::<Vec<_>>();

    let table_joins = input
        .joins
        .into_iter()
        .map(|join| {
            let (table, join_type, on) = match join {
                Join::Inner { table, on } => {
                    let on = on.into_tokens_with_checks(&mut checks, &sql_crate, true);

                    (
                        table,
                        quote! {#sql_crate::JoinType::Inner},
                        quote! {Some(#on)},
                    )
                }
                Join::Left { table, on } => {
                    let on = on.into_tokens_with_checks(&mut checks, &sql_crate, true);

                    (
                        table,
                        quote! {#sql_crate::JoinType::Left},
                        quote! {Some(#on)},
                    )
                }
                Join::Right { table, on } => {
                    let on = on.into_tokens_with_checks(&mut checks, &sql_crate, true);

                    (
                        table,
                        quote! {#sql_crate::JoinType::Right},
                        quote! {Some(#on)},
                    )
                }
                Join::Cross { table } => {
                    (table, quote! {#sql_crate::JoinType::Cross}, quote! {None})
                }
            };

            let alias = quote! {None};

            quote! {
                #sql_crate::TableJoin{
                    table: <#table as #sql_crate::SqlTable>::table_name(),
                    join_type: #join_type,
                    alias: #alias,
                    on: #on,
                }
            }
        })
        .collect::<Vec<_>>();

    let result = quote! {
        struct #item_name;

        impl #sql_crate::SqlTable for #item_name {
            fn table_name() -> &'static str {
                <#main_table_struct as #sql_crate::SqlTable>::table_name()
            }

            fn primary_keys() -> Vec<&'static str>{
                vec![]
            }

            fn table_joins() -> Vec<#sql_crate::TableJoin<'static>> {
                let _ = |___t___:#item_name|{
                    #(#checks)*
                };

                vec![
                    #(#table_joins),*
                ]
            }
        }

        impl #sql_crate::HasTable<#main_table_struct> for #item_name{}

        #(impl #sql_crate::HasTable<#has_table_impls> for #item_name{})*

        #(#has_table_joined_impls)*
    };

    // panic!("{}", result.to_string());

    result.into()
}
