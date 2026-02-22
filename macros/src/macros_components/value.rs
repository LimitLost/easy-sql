use super::{
    CollectedData, builtin_functions,
    column::Column,
    expr::Expr,
};
use ::{
    proc_macro2::{self},
    syn::{self, spanned::Spanned},
};
use convert_case::{Case, Casing};
use easy_macros::always_context;
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::ext::IdentExt;

#[derive(Debug, Clone)]
pub enum Value {
    Column(Column),
    Lit(syn::Lit),
    OutsideVariable(syn::Expr),
    Cast {
        expr: Box<Expr>,
        ty: syn::Type,
    },
    FunctionCall { name: syn::Ident, args: Option<Vec<Expr>> },
    Star(syn::Token![*]), // Special case for COUNT(*) and similar
}

#[derive(Debug, Clone)]
pub enum ValueIn {
    SingleVar(syn::Expr),
    SingleColumn(Column),
    Multiple(Vec<Expr>),
}

#[always_context]
impl Value {
    pub(super) fn lookahead(input: &syn::parse::ParseStream) -> bool {
        let lookahead = input.lookahead1();
        lookahead.peek(syn::Lit)
            || lookahead.peek(syn::token::Brace)
            || lookahead.peek(syn::Ident::peek_any)
    }

    pub(super) fn function_call_start(
        input: &syn::parse::ParseStream,
    ) -> syn::Result<Option<proc_macro2::Ident>> {
        if input.peek(syn::Ident::peek_any) && input.peek2(syn::token::Paren) {
            let ident = input.call(syn::Ident::parse_any)?;
            Ok(Some(ident))
        } else {
            Ok(None)
        }
    }

    pub(super) fn into_query_string(
        self,
        data: &mut CollectedData,
        inside_count: bool,
        for_custom_select: bool,
    ) -> String {
        let sql_crate = data.sql_crate;
        match self {
            Value::Column(col) => col.into_query_string(data, for_custom_select),
            Value::Lit(lit) => {
                if for_custom_select {
                    match lit {
                        syn::Lit::Str(lit_str) => {
                            let value = lit_str.value();
                            let escaped = value.replace("'", "''");
                            return format!("'{}'", escaped);
                        }
                        syn::Lit::Int(lit_int) => {
                            return lit_int.to_string();
                        }
                        syn::Lit::Float(lit_float) => {
                            return lit_float.to_string();
                        }
                        syn::Lit::Bool(lit_bool) => {
                            return if lit_bool.value { "TRUE" } else { "FALSE" }.to_string();
                        }
                        _ => {
                            return lit.to_token_stream().to_string();
                        }
                    }
                }

                let debug_str = format!(
                    "Failed to bind `{}` to query parameter",
                    lit.to_token_stream()
                );
                data.binds.push(quote::quote_spanned! {lit.span()=>
                    _easy_sql_args.add(&#lit).map_err(anyhow::Error::from_boxed).context(#debug_str)?;
                });
                data.format_params.push(data.driver.parameter_placeholder(
                    sql_crate,
                    lit.span(),
                    &data.before_param_n,
                    *data.current_param_n,
                ));
                *data.current_param_n += 1;
                "{}".to_string()
            }
            Value::OutsideVariable(expr_val) => {
                if for_custom_select {
                    let expr_val_span = expr_val.span();
                    if let syn::Expr::Path(expr_path) = expr_val
                        && expr_path.path.segments.len() == 1
                    {
                        let ident = &expr_path.path.segments[0].ident;
                        let ident_str = ident.to_string();
                        if let Some(stripped) = ident_str.strip_prefix("arg")
                            && let Ok(_idx) = stripped.parse::<usize>()
                        {
                            data.format_params.push(quote::quote! {
                                #ident
                            });
                            return "{}".to_string();
                        }
                    }
                    data.checks.push(quote::quote_spanned! {expr_val_span=>
                        {
                            compile_error!("Outside variables in custom select must be in the form {argN}, where N is the argument index. Then enter them in the query! like this: query!(SELECT CurrentType(input0, input1, ...) FROM ...)");
                        }
                    });
                    return "{}".to_string();
                }

                if data.main_table_type.is_none() {
                    data.checks.push(quote::quote_spanned! {expr_val.span()=>
                        {
                            compile_error!("Outside variables are not allowed inside of JOIN clauses. (yet)");
                        }
                    });
                    return "{}".to_string();
                }

                let debug_str = format!(
                    "Failed to bind `{}` to query parameter",
                    expr_val.to_token_stream()
                );
                data.binds.push(quote::quote_spanned! {expr_val.span()=>
                    _easy_sql_args.add(&#expr_val).map_err(anyhow::Error::from_boxed).context(#debug_str)?;
                });
                data.format_params.push(data.driver.parameter_placeholder(
                    sql_crate,
                    expr_val.span(),
                    &data.before_param_n,
                    *data.current_param_n,
                ));

                *data.current_param_n += 1;
                "{}".to_string()
            }
            Value::Cast { expr, ty } => {
                for driver_ty in data.driver.iter_for_checks() {
                    data.checks.push(quote_spanned! {ty.span()=>
                        {
                            fn __easy_sql_assert_supports_fn<T: #sql_crate::markers::functions::SupportsCast<1>>() {}
                            __easy_sql_assert_supports_fn::<#driver_ty>();
                        }
                    });
                }

                let arg_sql = expr.into_query_string(data, inside_count, for_custom_select);
                let type_info = data.driver.type_info(sql_crate, &ty);
                data.types_driver_support_needed.push(ty.to_token_stream());
                data.format_params.push(quote_spanned! {ty.span()=>
                    #type_info
                });
                format!("CAST({} AS {{}})", arg_sql)
            }
            Value::FunctionCall { name, args } => {
                let func_name_str = name.to_string();
                let builtin_fn_data = builtin_functions::get_builtin_fn(&func_name_str);
                let arg_count = args.as_ref().map(|args| args.len() as isize).unwrap_or(-1);
                let (func_name, is_count) = if builtin_fn_data.is_some() {
                    let func_name = func_name_str.to_uppercase();
                    let trait_name = format!("Supports{}", func_name.to_case(Case::Pascal));
                    let trait_ident = format_ident!("{}", trait_name);
                    let trait_path = quote! {#sql_crate::markers::functions::#trait_ident};

                    for driver_ty in data.driver.iter_for_checks() {
                        data.checks.push(quote_spanned! {name.span()=>
                            {
                                fn __easy_sql_assert_supports_fn<T: #trait_path<#arg_count>>() {}
                                __easy_sql_assert_supports_fn::<#driver_ty>();
                            }
                        });
                    }

                    (func_name.clone(), func_name == "COUNT")
                } else {
                    let macro_name = func_name_str.to_lowercase();
                    let macro_ident = proc_macro2::Ident::new_raw(&macro_name, name.span());
                    let dummy_args = (0..arg_count)
                        .map(|_| quote_spanned! {name.span()=> ()})
                        .collect::<Vec<_>>();
                    let macro_call = quote_spanned! {name.span()=>
                        #macro_ident!(#(#dummy_args),*)
                    };
                    data.format_params.push(macro_call);
                    ("{}".to_string(), false)
                };

                let mut arg_strings = Vec::new();

                if let Some(args) = args {
                    for arg in args {
                        let arg_sql = arg.into_query_string(data, is_count, for_custom_select);
                        arg_strings.push(arg_sql);
                    }
                    format!("{}({})", func_name, arg_strings.join(", "))
                } else {
                    func_name
                }
            }
            Value::Star(s) => {
                if !inside_count {
                    data.checks.push(quote_spanned! {s.span()=>
                        {
                            compile_error!("Star (*) is only valid inside function calls like COUNT(*).");
                        }
                    });
                }
                "*".to_string()
            }
        }
    }

    pub(super) fn collect_indices_impl(&self, indices: &mut std::collections::BTreeSet<usize>) {
        match self {
            Value::OutsideVariable(expr) => {
                if let syn::Expr::Path(expr_path) = expr
                    && expr_path.path.segments.len() == 1
                {
                    let ident_str = expr_path.path.segments[0].ident.to_string();
                    if let Some(stripped) = ident_str.strip_prefix("arg")
                        && let Ok(idx) = stripped.parse::<usize>()
                    {
                        indices.insert(idx);
                    }
                }
            }
            Value::FunctionCall {
                args: Some(args), ..
            } => {
                for e in args {
                    e.collect_indices_impl(indices);
                }
            }
            Value::Cast { expr, .. } => {
                expr.collect_indices_impl(indices);
            }
            _ => {}
        }
    }
}
