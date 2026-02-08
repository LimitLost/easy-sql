use super::CollectedData;
use ::{
    quote::ToTokens,
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
    pub fn into_query_string(self, data: &mut CollectedData, for_custom_select: bool) -> String {
        let sql_crate = data.sql_crate;
        match self {
            Column::SpecificTableColumn(table_type, col_name) => {
                // When output_ty matches table_type,
                // validate against Output type fields instead of Table

                fn output_matches_type(
                    output_ty: &proc_macro2::TokenStream,
                    table_type: &proc_macro2::TokenStream,
                ) -> bool {
                    use syn::Type;

                    // Parse both types
                    let output_type: Type = match syn::parse2(output_ty.clone()) {
                        Ok(ty) => ty,
                        Err(_) => return false,
                    };
                    let table_type: Type = match syn::parse2(table_type.clone()) {
                        Ok(ty) => ty,
                        Err(_) => return false,
                    };

                    fn types_equal(a: &Type, b: &Type) -> bool {
                        // Direct comparison of token streams
                        a.to_token_stream().to_string() == b.to_token_stream().to_string()
                    }

                    fn contains_type_recursively(haystack: &Type, needle: &Type) -> bool {
                        // Direct match
                        if types_equal(haystack, needle) {
                            return true;
                        }

                        // Recursively check generic arguments
                        match haystack {
                            Type::Path(type_path) => {
                                for segment in &type_path.path.segments {
                                    if let syn::PathArguments::AngleBracketed(args) =
                                        &segment.arguments
                                    {
                                        for arg in &args.args {
                                            if let syn::GenericArgument::Type(inner_ty) = arg
                                                && contains_type_recursively(inner_ty, needle)
                                            {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                            Type::Reference(type_ref) => {
                                return contains_type_recursively(&type_ref.elem, needle);
                            }
                            Type::Paren(type_paren) => {
                                return contains_type_recursively(&type_paren.elem, needle);
                            }
                            Type::Group(type_group) => {
                                return contains_type_recursively(&type_group.elem, needle);
                            }
                            Type::Tuple(type_tuple) => {
                                for elem in &type_tuple.elems {
                                    if contains_type_recursively(elem, needle) {
                                        return true;
                                    }
                                }
                            }
                            Type::Array(type_array) => {
                                return contains_type_recursively(&type_array.elem, needle);
                            }
                            Type::Slice(type_slice) => {
                                return contains_type_recursively(&type_slice.elem, needle);
                            }
                            _ => {}
                        }

                        false
                    }

                    contains_type_recursively(&output_type, &table_type)
                }

                if let Some(output_type) = data.output_ty
                    && output_matches_type(output_type, &table_type.to_token_stream())
                {
                    // User specified OutputType.column - validate against Output type fields (custom select can't reference other columns from select statement)

                    if for_custom_select {
                        // In custom select mode, referencing select columns (created in OutputType) is unsupported
                        data.checks.push(quote::quote_spanned! {table_type.span()=>
                                {
                                    compile_error!("Referencing select columns in custom select statements is not supported. Please use table column references instead.");
                                }
                            });
                        return format!("{{delimeter}}{}{{delimeter}}", col_name);
                    }

                    let output_ty = &data.output_ty;
                    let main_table_type = &data.main_table_type;

                    // and generate unqualified column reference (just the column name)
                    let drivers_iter = data.driver.iter_for_checks();
                    data.checks.push(quote::quote_spanned! {col_name.span()=>
                                #({
                                    let output_instance : <#output_ty as #sql_crate::Output<#main_table_type, #drivers_iter>>::UsedForChecks = #sql_crate::macro_support::never_any::<#table_type>();
                                    let _ = output_instance.#col_name;
                                })*
                            });

                    // Generate unqualified column name (Output fields map to table columns)
                    return format!("{{_easy_sql_d}}{}{{_easy_sql_d}}", col_name);
                }

                // Standard behavior: validate against Table type
                // User specified a different table - validate normally
                data.checks.push(quote::quote_spanned! {col_name.span()=>
                    {
                        fn has_table<T:#sql_crate::markers::HasTable<#table_type>>(_test:&T){}
                        has_table(&___t___);
                        let table_instance = #sql_crate::macro_support::never_any::<#table_type>();
                        let _ = table_instance.#col_name;
                    }
                });

                let delimeter = if for_custom_select {
                    "delimeter"
                } else {
                    "_easy_sql_d"
                };

                data.format_params
                    .push(data.driver.table_name(sql_crate, &table_type));

                format!(
                    "{{{delimeter}}}{{}}{{{delimeter}}}.{{{delimeter}}}{}{{{delimeter}}}",
                    col_name
                )
            }
            Column::Column(ident) => {
                let main_table_type = if let Some(mt) = data.main_table_type {
                    mt
                } else {
                    // Inside table join - no main table type available
                    data.checks.push(quote::quote_spanned! {ident.span()=>
                            {
                                compile_error!("Column references without a table prefix are not allowed inside of JOIN clauses. Please specify the table name explicitly, e.g., TableName.column_name");
                            }
                        });
                    return if for_custom_select {
                        format!("{{delimeter}}{ident}{{delimeter}}")
                    } else {
                        format!("{{_easy_sql_d}}{}{{_easy_sql_d}}", ident)
                    };
                };

                #[cfg(feature = "use_output_columns")]
                if !for_custom_select {
                    // Feature enabled: validate against Output type if provided, custom select can't reference other columns from select statement
                    if let Some(output_type) = data.output_ty {
                        let drivers_iter = data.driver.iter_for_checks();
                        data.checks.push(quote::quote_spanned! {ident.span()=>
                                #({
                                    let output_instance = #sql_crate::macro_support::never_any::<<#output_type as #sql_crate::Output<#main_table_type, #drivers_iter>>::UsedForChecks>();
                                    let _ = output_instance.#ident;
                                })*
                            });

                        return format!("{{_easy_sql_d}}{}{{_easy_sql_d}}", ident.to_string());
                    }
                }

                // Standard behavior: validate against Table type
                // This runs when:
                // - Feature is disabled (always validates against Table)
                // - Feature is enabled but no output_ty provided (fallback to Table validation)
                // - Custom select mode (can't reference other columns from select statement)
                data.checks.push(quote::quote_spanned! {ident.span()=>
                        {
                            let table_instance = #sql_crate::macro_support::never_any::<#main_table_type>();
                            let _ = table_instance.#ident;
                        }
                    });

                if for_custom_select {
                    format!("{{delimeter}}{ident}{{delimeter}}")
                } else {
                    format!("{{_easy_sql_d}}{}{{_easy_sql_d}}", ident)
                }
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
            Ok(Column::SpecificTableColumn(path_or_ident, ident))
        } else if let Some(ident) = path_or_ident.first()
            && path_or_ident.len() == 1
        {
            Ok(Column::Column(ident.clone()))
        } else {
            Err(input.error("Expected identifier instead of path (or dot after path)"))
        }
    }
}
