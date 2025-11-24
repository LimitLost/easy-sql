use crate::macros_components::keyword::DoubleArrow;

use super::{column::Column, next_clause::next_clause_token};

use super::keyword;
use ::{
    proc_macro2::{self, TokenStream},
    syn::{
        self,
        parse::{Lookahead1, Parse},
        spanned::Spanned,
    },
};
use easy_macros::always_context;
use easy_macros::readable_token_stream;
use quote::{ToTokens, quote, quote_spanned};

#[derive(Debug, Clone)]
pub enum Operator {
    ///AND Keyword
    And,
    ///OR Keyword
    Or,
    ///+
    Add,
    ///-
    Sub,
    ///*
    Mul,
    /// /
    Div,
    ///%
    Mod,
    /// ||
    Concat,
    ///-> or ->>
    JsonExtract,
    /// &
    BitAnd,
    /// |
    BitOr,
    /// <<
    BitShiftLeft,
    /// >>
    BitShiftRight,
}

#[always_context]
impl Parse for Operator {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::and) {
            input.parse::<keyword::and>()?;
            Ok(Operator::And)
        } else if lookahead.peek(keyword::or) {
            input.parse::<keyword::or>()?;
            Ok(Operator::Or)
        } else if lookahead.peek(syn::Token![+]) {
            input.parse::<syn::Token![+]>()?;
            Ok(Operator::Add)
        } else if lookahead.peek(syn::Token![-]) {
            input.parse::<syn::Token![-]>()?;
            Ok(Operator::Sub)
        } else if lookahead.peek(syn::Token![*]) {
            input.parse::<syn::Token![*]>()?;
            Ok(Operator::Mul)
        } else if lookahead.peek(syn::Token![/]) {
            input.parse::<syn::Token![/]>()?;
            Ok(Operator::Div)
        } else if lookahead.peek(syn::Token![%]) {
            input.parse::<syn::Token![%]>()?;
            Ok(Operator::Mod)
        } else if lookahead.peek(syn::Token![||]) {
            input.parse::<syn::Token![||]>()?;
            Ok(Operator::Concat)
        } else if lookahead.peek(DoubleArrow) {
            input.parse::<DoubleArrow>()?;
            Ok(Operator::JsonExtract)
        } else if lookahead.peek(syn::Token![->]) {
            input.parse::<syn::Token![->]>()?;
            Ok(Operator::JsonExtract)
        } else if lookahead.peek(syn::Token![&]) {
            input.parse::<syn::Token![&]>()?;
            Ok(Operator::BitAnd)
        } else if lookahead.peek(syn::Token![|]) {
            input.parse::<syn::Token![|]>()?;
            Ok(Operator::BitOr)
        } else if lookahead.peek(syn::Token![<<]) {
            input.parse::<syn::Token![<<]>()?;
            Ok(Operator::BitShiftLeft)
        } else if lookahead.peek(syn::Token![>>]) {
            input.parse::<syn::Token![>>]>()?;
            Ok(Operator::BitShiftRight)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value),
    Parenthesized(Box<Expr>),
    OperatorChain(Box<Expr>, Vec<(Operator, Expr)>),
    Not(Box<Expr>),
    IsNull(Value),
    IsNotNull(Value),
    Equal(Value, Value),
    NotEqual(Value, Value),
    GreaterThan(Value, Value),
    GreaterThanOrEqual(Value, Value),
    LessThan(Value, Value),
    LessThanOrEqual(Value, Value),
    Like(Value, Value),
    In(Value, ValueIn),
    Between(Value, Value, Value),
}

#[always_context]
impl Expr {
    pub fn into_query_string(
        &self,
        binds: &mut Vec<proc_macro2::TokenStream>,
        checks: &mut Vec<proc_macro2::TokenStream>,
        sql_crate: &proc_macro2::TokenStream,
        driver: &proc_macro2::TokenStream,
        current_param_n: &mut usize,
        current_format_params: &mut Vec<proc_macro2::TokenStream>,
        before_param_n: &proc_macro2::TokenStream,
    ) -> String {
        match self {
            Expr::Value(val) => val.into_query_string(
                binds,
                checks,
                sql_crate,
                driver,
                current_param_n,
                current_format_params,
                before_param_n,
            ),
            Expr::Equal(left, right) => {
                let left_sql = left.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                let right_sql = right.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("({} = {})", left_sql, right_sql)
            }
            Expr::NotEqual(left, right) => {
                let left_sql = left.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                let right_sql = right.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("({} != {})", left_sql, right_sql)
            }
            Expr::GreaterThan(left, right) => {
                let left_sql = left.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                let right_sql = right.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("({} > {})", left_sql, right_sql)
            }
            Expr::GreaterThanOrEqual(left, right) => {
                let left_sql = left.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                let right_sql = right.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("({} >= {})", left_sql, right_sql)
            }
            Expr::LessThan(left, right) => {
                let left_sql = left.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                let right_sql = right.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("({} < {})", left_sql, right_sql)
            }
            Expr::LessThanOrEqual(left, right) => {
                let left_sql = left.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                let right_sql = right.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("({} <= {})", left_sql, right_sql)
            }
            Expr::Like(left, right) => {
                let left_sql = left.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                let right_sql = right.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("({} LIKE {})", left_sql, right_sql)
            }
            Expr::IsNull(val) => {
                let val_sql = val.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("({} IS NULL)", val_sql)
            }
            Expr::IsNotNull(val) => {
                let val_sql = val.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("({} IS NOT NULL)", val_sql)
            }
            Expr::Not(inner) => {
                let inner_sql = inner.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("(NOT {})", inner_sql)
            }
            Expr::Parenthesized(inner) => {
                let inner_sql = inner.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("({})", inner_sql)
            }
            Expr::OperatorChain(first, rest) => {
                let mut result = format!(
                    "({}",
                    first.into_query_string(
                        binds,
                        checks,
                        sql_crate,
                        driver,
                        current_param_n,
                        current_format_params,
                        before_param_n,
                    )
                );
                for (op, expr) in rest {
                    let op_str = match op {
                        Operator::And => " AND ",
                        Operator::Or => " OR ",
                        Operator::Add => " + ",
                        Operator::Sub => " - ",
                        Operator::Mul => " * ",
                        Operator::Div => " / ",
                        Operator::Mod => " % ",
                        Operator::Concat => " || ",
                        Operator::JsonExtract => " -> ",
                        Operator::BitAnd => " & ",
                        Operator::BitOr => " | ",
                        Operator::BitShiftLeft => " << ",
                        Operator::BitShiftRight => " >> ",
                    };
                    result.push_str(op_str);
                    result.push_str(&expr.into_query_string(
                        binds,
                        checks,
                        sql_crate,
                        driver,
                        current_param_n,
                        current_format_params,
                        before_param_n,
                    ));
                }
                result.push(')');
                result
            }
            Expr::In(val, values) => {
                let val_sql = val.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                match values {
                    ValueIn::Multiple(vals) => {
                        let mut in_items = Vec::new();
                        for v in vals {
                            in_items.push(v.into_query_string(
                                binds,
                                checks,
                                sql_crate,
                                driver,
                                current_param_n,
                                current_format_params,
                                before_param_n,
                            ));
                        }
                        format!("({} IN ({}))", val_sql, in_items.join(", "))
                    }
                    ValueIn::Single(_) => {
                        // Iterator case - not supported in compile-time generation yet
                        format!("({} IN ({{ITERATOR}}))", val_sql)
                    }
                }
            }
            Expr::Between(val, min, max) => {
                let val_sql = val.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                let min_sql = min.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                let max_sql = max.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    current_param_n,
                    current_format_params,
                    before_param_n,
                );
                format!("({} BETWEEN {} AND {})", val_sql, min_sql, max_sql)
            }
        }
    }

    pub fn into_tokens_with_checks(
        self,
        checks: &mut Vec<TokenStream>,
        binds: &mut Vec<TokenStream>,
        sql_crate: &TokenStream,
        perform_bool_check: bool,
        driver: &TokenStream,
    ) -> proc_macro2::TokenStream {
        // Helper to convert Value using the appropriate method based on use_safe_bindings
        let convert_value =
            |val: Value, checks: &mut Vec<TokenStream>, binds: &mut Vec<TokenStream>| {
                val.into_tokens_with_checks(checks, binds, sql_crate, driver)
            };

        match self {
            Expr::Value(sql_value) => match sql_value {
                Value::Column(sql_column) => match sql_column {
                    Column::SpecificTableColumn(path, ident) => {
                        let bool_check = if perform_bool_check {
                            quote! {
                                let _ = bool::from(table_instance.#ident);
                            }
                        } else {
                            quote! {}
                        };
                        checks.push(quote_spanned! {ident.span()=>
                            {
                                fn has_table<T:#sql_crate::HasTable<#path>>(_test:&T){}
                                has_table(&___t___);
                                //TODO "RealColumns" trait with type leading to the struct with actual database columns
                                let table_instance = #sql_crate::never::never_any::<#path>();
                                #bool_check
                            }
                        });

                        let ident_str = ident.to_string();
                        quote_spanned! {ident.span()=>
                            #sql_crate::Expr::ColumnFromTable{
                                table: <#path as #sql_crate::Table<#driver>>::table_name().to_owned(),
                                column: #ident_str.to_string(),
                            }
                        }
                    }
                    Column::Column(ident) => {
                        if perform_bool_check {
                            checks.push(quote_spanned! {ident.span()=>
                                {
                                    let _= bool::from(___t___.#ident);
                                }
                            });
                        }
                        let ident_str = ident.to_string();

                        quote_spanned! {ident.span()=>
                            #sql_crate::Expr::Column(#ident_str.to_string())
                        }
                    }
                },
                Value::Lit(lit) => {
                    if perform_bool_check {
                        match lit {
                            syn::Lit::Bool(lit_bool) => {
                                let debug_str =
                                    format!("Failed to bind `{}` to query", lit_bool.value());

                                binds.push(quote_spanned! {lit_bool.span()=>
                                    __easy_sql_builder.bind(#lit_bool).context(#debug_str)?;
                                });

                                quote_spanned! {lit_bool.span()=>
                                    #sql_crate::Expr::Value
                                }
                            }
                            l => {
                                let error_str = format!(
                                    "Expected a boolean literal, got {}",
                                    l.to_token_stream()
                                );
                                checks.push(quote_spanned! {l.span()=>
                                    {
                                        compile_error!(#error_str);
                                    }
                                });
                                quote! {
                                    #sql_crate::Expr::Error
                                }
                            }
                        }
                    } else {
                        binds.push(quote_spanned! {lit.span()=>
                            __easy_sql_builder.bind(#lit)?;
                        });

                        quote_spanned! {lit.span()=>
                            #sql_crate::Expr::Value
                        }
                    }
                }
                Value::OutsideVariable(expr) => {
                    if perform_bool_check {
                        checks.push(quote_spanned! {expr.span()=>
                            {
                                let _ =bool::from({#expr});
                            }
                        });
                    }
                    let debug_str = format!(
                        "Failed to bind `{}` to query",
                        readable_token_stream(&expr.to_token_stream().to_string())
                    );
                    binds.push(quote_spanned! {expr.span()=>
                        __easy_sql_builder.bind({#expr}).context(#debug_str)?;

                    });

                    quote_spanned! {expr.span()=>
                        #sql_crate::Expr::Value
                    }
                }
            },
            Expr::Parenthesized(where_expr) => {
                let inside_parsed = where_expr.into_tokens_with_checks(
                    checks,
                    binds,
                    sql_crate,
                    perform_bool_check,
                    driver,
                );
                quote! {
                    #sql_crate::Expr::Parenthesized(Box::new(#inside_parsed))
                }
            }
            Expr::OperatorChain(where_expr, items) => {
                let first_bool_check = items
                    .iter()
                    .any(|(op, _)| matches!(op, Operator::And | Operator::Or));

                let first_parsed = where_expr.into_tokens_with_checks(
                    checks,
                    binds,
                    sql_crate,
                    first_bool_check,
                    driver,
                );

                let next_item_bool = items
                    .iter()
                    .skip(1)
                    .map(|(op, _)| matches!(op, Operator::And | Operator::Or))
                    .chain(std::iter::once(false))
                    .collect::<Vec<_>>();

                let mut items_parsed = Vec::new();
                for ((and_or, where_expr), next_item_bool) in items.into_iter().zip(next_item_bool)
                {
                    let current_expected_bool = matches!(and_or, Operator::And | Operator::Or);

                    let inside_parsed = where_expr.into_tokens_with_checks(
                        checks,
                        binds,
                        sql_crate,
                        current_expected_bool,
                        driver,
                    );
                    let and_or_parsed = match and_or {
                        Operator::And => quote! {(#sql_crate::Operator::And, #inside_parsed)},
                        Operator::Or => quote! {(#sql_crate::Operator::Or, #inside_parsed)},
                        Operator::Add => quote! {(#sql_crate::Operator::Add, #inside_parsed)},
                        Operator::Sub => quote! {(#sql_crate::Operator::Sub, #inside_parsed)},
                        Operator::Mul => quote! {(#sql_crate::Operator::Mul, #inside_parsed)},
                        Operator::Div => quote! {(#sql_crate::Operator::Div, #inside_parsed)},
                        Operator::Mod => quote! {(#sql_crate::Operator::Mod, #inside_parsed)},
                        Operator::Concat => quote! {(#sql_crate::Operator::Concat, #inside_parsed)},
                        Operator::JsonExtract => {
                            quote! {(#sql_crate::Operator::JsonExtract, #inside_parsed)}
                        }
                        Operator::BitAnd => quote! {(#sql_crate::Operator::BitAnd, #inside_parsed)},
                        Operator::BitOr => quote! {(#sql_crate::Operator::BitOr, #inside_parsed)},
                        Operator::BitShiftLeft => {
                            quote! {(#sql_crate::Operator::BitShiftLeft, #inside_parsed)}
                        }
                        Operator::BitShiftRight => {
                            quote! {(#sql_crate::Operator::BitShiftRight, #inside_parsed)}
                        }
                    };
                    items_parsed.push(and_or_parsed);
                }

                quote! {
                    #sql_crate::Expr::OperatorChain(Box::new(#first_parsed), vec![#(#items_parsed),*])
                }
            }
            Expr::Not(where_expr) => {
                let parsed =
                    where_expr.into_tokens_with_checks(checks, binds, sql_crate, true, driver);
                quote! {
                    #sql_crate::Expr::Not(Box::new(#parsed))
                }
            }
            Expr::IsNull(sql_value) => {
                let parsed = convert_value(sql_value, checks, binds);
                quote! {
                    #sql_crate::Expr::IsNull(Box::new(#parsed))
                }
            }
            Expr::IsNotNull(sql_value) => {
                let parsed = convert_value(sql_value, checks, binds);
                quote! {
                    #sql_crate::Expr::IsNotNull(Box::new(#parsed))
                }
            }
            Expr::Equal(sql_value, sql_value1) => {
                let parsed = convert_value(sql_value, checks, binds);
                let parsed1 = convert_value(sql_value1, checks, binds);
                quote! {
                    #sql_crate::Expr::Eq(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            Expr::NotEqual(sql_value, sql_value1) => {
                let parsed = convert_value(sql_value, checks, binds);
                let parsed1 = convert_value(sql_value1, checks, binds);
                quote! {
                    #sql_crate::Expr::NotEq(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            Expr::GreaterThan(sql_value, sql_value1) => {
                let parsed = convert_value(sql_value, checks, binds);
                let parsed1 = convert_value(sql_value1, checks, binds);
                quote! {
                    #sql_crate::Expr::Gt(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            Expr::GreaterThanOrEqual(sql_value, sql_value1) => {
                let parsed = convert_value(sql_value, checks, binds);
                let parsed1 = convert_value(sql_value1, checks, binds);
                quote! {
                    #sql_crate::Expr::GtEq(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            Expr::LessThan(sql_value, sql_value1) => {
                let parsed = convert_value(sql_value, checks, binds);
                let parsed1 = convert_value(sql_value1, checks, binds);
                quote! {
                    #sql_crate::Expr::Lt(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            Expr::LessThanOrEqual(sql_value, sql_value1) => {
                let parsed = convert_value(sql_value, checks, binds);
                let parsed1 = convert_value(sql_value1, checks, binds);
                quote! {
                    #sql_crate::Expr::LtEq(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            Expr::Like(sql_value, sql_value1) => {
                let parsed = convert_value(sql_value, checks, binds);
                let parsed1 = convert_value(sql_value1, checks, binds);
                quote! {
                    #sql_crate::Expr::Like(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            Expr::In(sql_value, sql_value_in) => {
                let parsed = convert_value(sql_value, checks, binds);

                match sql_value_in {
                    ValueIn::Single(sql_value) => {
                        //Iterator expected
                        match sql_value {
                            Value::OutsideVariable(expr) => {
                                quote_spanned! {expr.span()=>
                                    {
                                        fn ___collect_iterator<'a,D:#sql_crate::Driver,Y:Into<D::Value<'a>>,T:Iterator<Item=Y>>(i:T)->Vec<D::Value<'a>>{
                                            let collected=Vec::new();
                                            for item in i{
                                                collected.push(item.into());
                                            }
                                            collected
                                        }

                                        #sql_crate::Expr::In(Box::new(#parsed),___collect_iterator({#expr}))
                                    }
                                }
                            }
                            v => {
                                let err_message = format!("Expected a list of values, got {:?}", v);

                                checks.push(quote_spanned! {v.span()=>

                                    {
                                        compile_error!(#err_message)
                                    }
                                });

                                quote! {
                                    #sql_crate::Expr::Error
                                }
                            }
                        }
                    }
                    ValueIn::Multiple(sql_values) => {
                        let mut parsed_values = Vec::new();
                        for sql_value in sql_values {
                            let parsed_value = convert_value(sql_value, checks, binds);
                            parsed_values.push(parsed_value);
                        }
                        quote! {
                            #sql_crate::Expr::In(Box::new(#parsed), vec![#(#parsed_values),*])
                        }
                    }
                }
            }
            Expr::Between(sql_value, sql_value1, sql_value2) => {
                let parsed = convert_value(sql_value, checks, binds);
                let parsed1 = convert_value(sql_value1, checks, binds);
                let parsed2 = convert_value(sql_value2, checks, binds);
                quote! {
                    #sql_crate::Expr::Between(Box::new(#parsed), Box::new(#parsed1), Box::new(#parsed2))
                }
            }
        }
    }
}
#[derive(Debug, Clone)]
pub enum Value {
    Column(Column),
    Lit(syn::Lit),
    OutsideVariable(syn::Expr),
}

#[always_context]
impl Value {
    fn span(&self) -> proc_macro2::Span {
        match self {
            Value::Column(sql_column) => match sql_column {
                Column::SpecificTableColumn(path, _) => path.span(),
                Column::Column(ident) => ident.span(),
            },
            Value::Lit(lit) => lit.span(),
            Value::OutsideVariable(expr) => expr.span(),
        }
    }

    fn into_query_string(
        &self,
        binds: &mut Vec<proc_macro2::TokenStream>,
        checks: &mut Vec<proc_macro2::TokenStream>,
        sql_crate: &proc_macro2::TokenStream,
        driver: &proc_macro2::TokenStream,
        current_param_n: &mut usize,
        current_format_params: &mut Vec<proc_macro2::TokenStream>,
        before_param_n: &proc_macro2::TokenStream,
    ) -> String {
        match self {
            Value::Column(col) => match col {
                Column::SpecificTableColumn(table_type, col_name) => {
                    checks.push(quote::quote_spanned! {col_name.span()=>
                        {
                            fn has_table<T:#sql_crate::HasTable<#table_type>>(_test:&T){}
                            has_table(&___t___);
                            let table_instance = #sql_crate::never::never_any::<#table_type>();
                            let _ = table_instance.#col_name;
                        }
                    });
                    current_format_params.push(quote! {
                        <#table_type as #sql_crate::Table<#driver>>::table_name()
                    });
                    format!(
                        "{{_easy_sql_d}}{{}}{{_easy_sql_d}}.{{_easy_sql_d}}{}{{_easy_sql_d}}",
                        col_name
                    )
                }
                Column::Column(ident) => {
                    checks.push(quote::quote_spanned! {ident.span()=>
                        {
                            let _= ___t___.#ident;
                        }
                    });
                    format!("{{_easy_sql_d}}{}{{_easy_sql_d}}", ident.to_string())
                }
            },
            Value::Lit(lit) => {
                let debug_str = format!(
                    "Failed to bind `{}` to query parameter",
                    lit.to_token_stream().to_string()
                );
                binds.push(quote::quote_spanned! {lit.span()=>
                        _easy_sql_args.add(&#lit).map_err(anyhow::Error::from_boxed).context(#debug_str)?;
                    });
                current_format_params.push(quote! {
                    <#driver as #sql_crate::Driver>::parameter_placeholder(#before_param_n #current_param_n)
                });
                *current_param_n += 1;
                "{}".to_string()
            }
            Value::OutsideVariable(expr_val) => {
                let debug_str = format!(
                    "Failed to bind `{}` to query parameter",
                    expr_val.to_token_stream().to_string()
                );
                binds.push(quote::quote_spanned! {expr_val.span()=>
                        _easy_sql_args.add(&#expr_val).map_err(anyhow::Error::from_boxed).context(#debug_str)?;
                    });
                current_format_params.push(quote! {
                    <#driver as #sql_crate::Driver>::parameter_placeholder(#before_param_n #current_param_n)
                });
                *current_param_n += 1;
                "{}".to_string()
            }
        }
    }

    fn into_tokens_with_checks(
        self,
        checks: &mut Vec<TokenStream>,
        binds: &mut Vec<TokenStream>,
        sql_crate: &TokenStream,
        driver: &TokenStream,
    ) -> proc_macro2::TokenStream {
        match self {
            Value::Column(sql_column) => {
                match sql_column {
                    Column::SpecificTableColumn(path, ident) => {
                        checks.push(quote_spanned! {ident.span()=>
                            {
                                fn has_table<T:#sql_crate::HasTable<#path>>(_test:&T){}
                                has_table(&___t___);
                                //TODO "RealColumns" trait with type leading to the struct with actual database columns
                                let table_instance = #sql_crate::never::never_any::<#path>();
                                let _ = table_instance.#ident;
                            }
                        });

                        let ident_str = ident.to_string();
                        quote_spanned! {ident.span()=>
                            #sql_crate::Expr::ColumnFromTable{
                                table: <#path as #sql_crate::Table<#driver>>::table_name().to_owned(),
                                column: #ident_str.to_string(),
                            }
                        }
                    }
                    Column::Column(ident) => {
                        checks.push(quote_spanned! {ident.span()=>
                            {
                                let _= ___t___.#ident;
                            }
                        });
                        let ident_str = ident.to_string();

                        quote_spanned! {ident.span()=>
                            #sql_crate::Expr::Column(#ident_str.to_string())
                        }
                    }
                }
            }
            Value::Lit(lit) => {
                let debug_str = format!(
                    "Failed to bind `{}` to query",
                    readable_token_stream(&lit.to_token_stream().to_string())
                );
                binds.push(quote_spanned! {lit.span()=>
                    __easy_sql_builder.bind(#lit).context(#debug_str)?;
                });

                quote_spanned! {lit.span()=>
                    #sql_crate::Expr::Value
                }
            }
            Value::OutsideVariable(expr) => {
                let debug_str = format!(
                    "Failed to bind `{}` to query",
                    readable_token_stream(&expr.to_token_stream().to_string())
                );
                binds.push(quote_spanned! {expr.span()=>
                    __easy_sql_builder.bind({#expr}).context(#debug_str)?;
                });

                quote_spanned! {expr.span()=>
                    #sql_crate::Expr::Value
                }
            }
        }
    }
}
#[derive(Debug, Clone)]
pub enum ValueIn {
    Single(Value),
    Multiple(Vec<Value>),
}

#[always_context]
impl Value {
    fn lookahead(l: &Lookahead1<'_>) -> bool {
        l.peek(syn::Ident) || l.peek(syn::Lit) || l.peek(syn::token::Brace)
    }
}

#[always_context]
impl Parse for Value {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
            Ok(Value::Column(input.parse()?))
        } else if lookahead.peek(syn::Lit) {
            let lit: syn::Lit = input.parse()?;
            Ok(Value::Lit(lit))
        } else if lookahead.peek(syn::token::Brace) {
            let inside_braces;
            syn::braced!(inside_braces in input);
            let expr: syn::Expr = inside_braces.parse()?;
            Ok(Value::OutsideVariable(expr))
        } else {
            Err(lookahead.error())
        }
    }
}

#[always_context]
impl Parse for ValueIn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Paren) {
            let inside_paren;
            syn::parenthesized!(inside_paren in input);
            let mut values = Vec::new();
            while !inside_paren.is_empty() {
                let value = inside_paren.parse::<Value>()?;
                values.push(value);
                if inside_paren.is_empty() {
                    break;
                }
                let lookahead2 = inside_paren.lookahead1();
                if lookahead2.peek(syn::Token![,]) {
                    inside_paren.parse::<syn::Token![,]>()?;
                } else {
                    break;
                }
            }
            Ok(ValueIn::Multiple(values))
        } else if Value::lookahead(&lookahead) {
            let value = input.parse::<Value>()?;
            Ok(ValueIn::Single(value))
        } else {
            Err(lookahead.error())
        }
    }
}

fn continue_parse_value_no_expr(
    input: syn::parse::ParseStream,
    current_value: Value,
    lookahead: syn::parse::Lookahead1<'_>,
) -> syn::Result<Expr> {
    if input.is_empty() || next_clause_token(&lookahead) {
        return Ok(Expr::Value(current_value));
    }

    if lookahead.peek(keyword::is) {
        input.parse::<keyword::is>()?;
        let lookahead2 = input.lookahead1();
        if lookahead2.peek(keyword::not) {
            input.parse::<keyword::not>()?;
            let lookahead3 = input.lookahead1();
            if lookahead3.peek(keyword::null) {
                input.parse::<keyword::null>()?;
                Ok(Expr::IsNotNull(current_value))
            } else {
                Err(lookahead3.error())
            }
        } else if lookahead2.peek(keyword::null) {
            input.parse::<keyword::null>()?;
            Ok(Expr::IsNull(current_value))
        } else {
            Err(lookahead2.error())
        }
    } else if lookahead.peek(syn::Token![=]) {
        input.parse::<syn::Token![=]>()?;
        let right_value = input.parse::<Value>()?;
        Ok(Expr::Equal(current_value, right_value))
    } else if lookahead.peek(syn::Token![!=]) {
        input.parse::<syn::Token![!=]>()?;
        let right_value = input.parse::<Value>()?;
        Ok(Expr::NotEqual(current_value, right_value))
    } else if lookahead.peek(syn::Token![>=]) {
        input.parse::<syn::Token![>=]>()?;
        let right_value = input.parse::<Value>()?;
        Ok(Expr::GreaterThanOrEqual(current_value, right_value))
    } else if lookahead.peek(syn::Token![>]) {
        input.parse::<syn::Token![>]>()?;
        let right_value = input.parse::<Value>()?;
        Ok(Expr::GreaterThan(current_value, right_value))
    } else if lookahead.peek(syn::Token![<=]) {
        input.parse::<syn::Token![<=]>()?;
        let right_value = input.parse::<Value>()?;
        Ok(Expr::LessThanOrEqual(current_value, right_value))
    } else if lookahead.peek(syn::Token![<]) {
        input.parse::<syn::Token![<]>()?;
        let right_value = input.parse::<Value>()?;
        Ok(Expr::LessThan(current_value, right_value))
    } else if lookahead.peek(keyword::like) {
        input.parse::<keyword::like>()?;
        let right_value = input.parse::<Value>()?;
        Ok(Expr::Like(current_value, right_value))
    } else if lookahead.peek(keyword::in_) {
        input.parse::<keyword::in_>()?;
        let right_value = input.parse::<ValueIn>()?;
        Ok(Expr::In(current_value, right_value))
    } else if lookahead.peek(keyword::between) {
        input.parse::<keyword::between>()?;
        let middle_value = input.parse::<Value>()?;
        let lookahead2 = input.lookahead1();
        if lookahead2.peek(keyword::and) {
            input.parse::<keyword::and>()?;
            let right_value = input.parse::<Value>()?;
            Ok(Expr::Between(current_value, middle_value, right_value))
        } else {
            Err(lookahead2.error())
        }
    } else {
        Err(lookahead.error())
    }
}

fn continue_parse_value_maybe_expr(
    input: syn::parse::ParseStream,
    current_value: Value,
) -> syn::Result<Expr> {
    if input.is_empty() {
        return Ok(Expr::Value(current_value));
    }

    let lookahead = input.lookahead1();

    if lookahead.peek(keyword::and)
        || lookahead.peek(keyword::or)
        || lookahead.peek(syn::Token![+])
        || lookahead.peek(syn::Token![-])
        || lookahead.peek(syn::Token![*])
        || lookahead.peek(syn::Token![/])
        || lookahead.peek(syn::Token![%])
        || lookahead.peek(syn::Token![||])
        || lookahead.peek(DoubleArrow)
        || lookahead.peek(syn::Token![->])
        || lookahead.peek(syn::Token![&])
        || lookahead.peek(syn::Token![|])
        || lookahead.peek(syn::Token![<<])
        || lookahead.peek(syn::Token![>>])
    {
        // We handle operators in the Expr::parse method
        Ok(Expr::Value(current_value))
    } else {
        continue_parse_value_no_expr(input, current_value, lookahead)
    }
}

fn sub_where_expr(input: syn::parse::ParseStream) -> syn::Result<Expr> {
    let lookahead = input.lookahead1();

    if lookahead.peek(keyword::not) {
        input.parse::<keyword::not>()?;

        let expr = sub_where_expr(input)?;
        Ok(Expr::Not(Box::new(expr)))
    } else if lookahead.peek(syn::token::Paren) {
        let inside_paren;
        syn::parenthesized!(inside_paren in input);
        let expr = inside_paren.parse::<Expr>()?;
        Ok(Expr::Parenthesized(Box::new(expr)))
    } else if Value::lookahead(&lookahead) {
        let parsed = input.parse::<Value>()?;

        Ok(continue_parse_value_maybe_expr(input, parsed)?)
    } else {
        Err(lookahead.error())
    }
}

#[always_context]
impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut first_expr = None;
        let mut next_exprs = vec![];

        while !input.is_empty() {
            let and_or = if first_expr.is_some() {
                let lookahead = input.lookahead1();

                if next_clause_token(&lookahead) {
                    break;
                }

                Some(input.parse::<Operator>()?)
            } else {
                None
            };

            let current_expr = sub_where_expr(&input)?;

            if let Some(and_or) = and_or {
                next_exprs.push((and_or, current_expr));
            } else {
                first_expr = Some(current_expr);
            }
        }

        let first_expr = if let Some(first_expr) = first_expr {
            first_expr
        } else {
            return Err(input.error("Expected a valid where expression, if you don't want to use any conditions, use `true`"));
        };

        if next_exprs.is_empty() {
            Ok(first_expr)
        } else {
            Ok(Expr::OperatorChain(Box::new(first_expr), next_exprs))
        }
    }
}
