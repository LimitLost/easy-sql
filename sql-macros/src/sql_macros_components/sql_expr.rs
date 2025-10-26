use crate::sql_macros_components::sql_keyword::DoubleArrow;

use super::{sql_column::SqlColumn, sql_next_clause::next_clause_token};

use super::sql_keyword;
use ::{
    proc_macro2::{self, TokenStream},
    syn::{
        self,
        parse::{Lookahead1, Parse},
        spanned::Spanned,
    },
};
use easy_macros::helpers::readable_token_stream;
use easy_macros::macros::always_context;
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
        if lookahead.peek(sql_keyword::and) {
            input.parse::<sql_keyword::and>()?;
            Ok(Operator::And)
        } else if lookahead.peek(sql_keyword::or) {
            input.parse::<sql_keyword::or>()?;
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
pub enum SqlExpr {
    Value(SqlValue),
    Parenthesized(Box<SqlExpr>),
    OperatorChain(Box<SqlExpr>, Vec<(Operator, SqlExpr)>),
    Not(Box<SqlExpr>),
    IsNull(SqlValue),
    IsNotNull(SqlValue),
    Equal(SqlValue, SqlValue),
    NotEqual(SqlValue, SqlValue),
    GreaterThan(SqlValue, SqlValue),
    GreaterThanOrEqual(SqlValue, SqlValue),
    LessThan(SqlValue, SqlValue),
    LessThanOrEqual(SqlValue, SqlValue),
    Like(SqlValue, SqlValue),
    In(SqlValue, SqlValueIn),
    Between(SqlValue, SqlValue, SqlValue),
}

#[always_context]
impl SqlExpr {
    pub fn into_tokens_with_checks(
        self,
        checks: &mut Vec<TokenStream>,
        binds: &mut Vec<TokenStream>,
        sql_crate: &TokenStream,
        perform_bool_check: bool,
        driver: &TokenStream,
    ) -> proc_macro2::TokenStream {
        match self {
            SqlExpr::Value(sql_value) => match sql_value {
                SqlValue::Column(sql_column) => match sql_column {
                    SqlColumn::SpecificTableColumn(path, ident) => {
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
                            #sql_crate::SqlExpr::ColumnFromTable{
                                table: <#path as #sql_crate::SqlTable<#driver>>::table_name().to_owned(),
                                column: #ident_str.to_string(),
                            }
                        }
                    }
                    SqlColumn::Column(ident) => {
                        if perform_bool_check {
                            checks.push(quote_spanned! {ident.span()=>
                                {
                                    let _= bool::from(___t___.#ident);
                                }
                            });
                        }
                        let ident_str = ident.to_string();

                        quote_spanned! {ident.span()=>
                            #sql_crate::SqlExpr::Column(#ident_str.to_string())
                        }
                    }
                },
                SqlValue::Lit(lit) => {
                    if perform_bool_check {
                        match lit {
                            syn::Lit::Bool(lit_bool) => {
                                let debug_str =
                                    format!("Failed to bind `{}` to query", lit_bool.value());

                                binds.push(quote_spanned! {lit_bool.span()=>
                                    __easy_sql_builder.bind(#lit_bool).context(#debug_str)?;
                                });

                                quote_spanned! {lit_bool.span()=>
                                    #sql_crate::SqlExpr::Value
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
                                    #sql_crate::SqlExpr::Error
                                }
                            }
                        }
                    } else {
                        binds.push(quote_spanned! {lit.span()=>
                            __easy_sql_builder.bind(#lit)?;
                        });

                        quote_spanned! {lit.span()=>
                            #sql_crate::SqlExpr::Value
                        }
                    }
                }
                SqlValue::OutsideVariable(expr) => {
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
                        #sql_crate::SqlExpr::Value
                    }
                }
            },
            SqlExpr::Parenthesized(where_expr) => {
                let inside_parsed = where_expr.into_tokens_with_checks(
                    checks,
                    binds,
                    sql_crate,
                    perform_bool_check,
                    driver,
                );
                quote! {
                    #sql_crate::SqlExpr::Parenthesized(Box::new(#inside_parsed))
                }
            }
            SqlExpr::OperatorChain(where_expr, items) => {
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
                    let current_expected_bool =
                        matches!(and_or, Operator::And | Operator::Or) || next_item_bool;

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
                    #sql_crate::SqlExpr::OperatorChain(Box::new(#first_parsed), vec![#(#items_parsed),*])
                }
            }
            SqlExpr::Not(where_expr) => {
                let parsed =
                    where_expr.into_tokens_with_checks(checks, binds, sql_crate, true, driver);
                quote! {
                    #sql_crate::SqlExpr::Not(Box::new(#parsed))
                }
            }
            SqlExpr::IsNull(sql_value) => {
                let parsed = sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);
                quote! {
                    #sql_crate::SqlExpr::IsNull(Box::new(#parsed))
                }
            }
            SqlExpr::IsNotNull(sql_value) => {
                let parsed = sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);
                quote! {
                    #sql_crate::SqlExpr::IsNotNull(Box::new(#parsed))
                }
            }
            SqlExpr::Equal(sql_value, sql_value1) => {
                let parsed = sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);
                let parsed1 = sql_value1.into_tokens_with_checks(checks, binds, sql_crate, driver);
                quote! {
                    #sql_crate::SqlExpr::Eq(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            SqlExpr::NotEqual(sql_value, sql_value1) => {
                let parsed = sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);
                let parsed1 = sql_value1.into_tokens_with_checks(checks, binds, sql_crate, driver);
                quote! {
                    #sql_crate::SqlExpr::NotEq(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            SqlExpr::GreaterThan(sql_value, sql_value1) => {
                let parsed = sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);
                let parsed1 = sql_value1.into_tokens_with_checks(checks, binds, sql_crate, driver);
                quote! {
                    #sql_crate::SqlExpr::Gt(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            SqlExpr::GreaterThanOrEqual(sql_value, sql_value1) => {
                let parsed = sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);
                let parsed1 = sql_value1.into_tokens_with_checks(checks, binds, sql_crate, driver);
                quote! {
                    #sql_crate::SqlExpr::GtEq(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            SqlExpr::LessThan(sql_value, sql_value1) => {
                let parsed = sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);
                let parsed1 = sql_value1.into_tokens_with_checks(checks, binds, sql_crate, driver);
                quote! {
                    #sql_crate::SqlExpr::Lt(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            SqlExpr::LessThanOrEqual(sql_value, sql_value1) => {
                let parsed = sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);
                let parsed1 = sql_value1.into_tokens_with_checks(checks, binds, sql_crate, driver);
                quote! {
                    #sql_crate::SqlExpr::LtEq(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            SqlExpr::Like(sql_value, sql_value1) => {
                let parsed = sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);
                let parsed1 = sql_value1.into_tokens_with_checks(checks, binds, sql_crate, driver);
                quote! {
                    #sql_crate::SqlExpr::Like(Box::new(#parsed), Box::new(#parsed1))
                }
            }
            SqlExpr::In(sql_value, sql_value_in) => {
                let parsed = sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);

                match sql_value_in {
                    SqlValueIn::Single(sql_value) => {
                        //Iterator expected
                        match sql_value {
                            SqlValue::OutsideVariable(expr) => {
                                quote_spanned! {expr.span()=>
                                    {
                                        fn ___collect_iterator<'a,D:#sql_crate::Driver,Y:Into<D::Value<'a>>,T:Iterator<Item=Y>>(i:T)->Vec<D::Value<'a>>{
                                            let collected=Vec::new();
                                            for item in i{
                                                collected.push(item.into());
                                            }
                                            collected
                                        }

                                        #sql_crate::SqlExpr::In(Box::new(#parsed),___collect_iterator({#expr}))
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
                                    #sql_crate::SqlExpr::Error
                                }
                            }
                        }
                    }
                    SqlValueIn::Multiple(sql_values) => {
                        let mut parsed_values = Vec::new();
                        for sql_value in sql_values {
                            let parsed_value =
                                sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);
                            parsed_values.push(parsed_value);
                        }
                        quote! {
                            #sql_crate::SqlExpr::In(Box::new(#parsed), vec![#(#parsed_values),*])
                        }
                    }
                }
            }
            SqlExpr::Between(sql_value, sql_value1, sql_value2) => {
                let parsed = sql_value.into_tokens_with_checks(checks, binds, sql_crate, driver);
                let parsed1 = sql_value1.into_tokens_with_checks(checks, binds, sql_crate, driver);
                let parsed2 = sql_value2.into_tokens_with_checks(checks, binds, sql_crate, driver);
                quote! {
                    #sql_crate::SqlExpr::Between(Box::new(#parsed), Box::new(#parsed1), Box::new(#parsed2))
                }
            }
        }
    }
}
#[derive(Debug, Clone)]
pub enum SqlValue {
    Column(SqlColumn),
    Lit(syn::Lit),
    OutsideVariable(syn::Expr),
}

#[always_context]
impl SqlValue {
    fn span(&self) -> proc_macro2::Span {
        match self {
            SqlValue::Column(sql_column) => match sql_column {
                SqlColumn::SpecificTableColumn(path, _) => path.span(),
                SqlColumn::Column(ident) => ident.span(),
            },
            SqlValue::Lit(lit) => lit.span(),
            SqlValue::OutsideVariable(expr) => expr.span(),
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
            SqlValue::Column(sql_column) => {
                match sql_column {
                    SqlColumn::SpecificTableColumn(path, ident) => {
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
                            #sql_crate::SqlExpr::ColumnFromTable{
                                table: <#path as #sql_crate::SqlTable<#driver>>::table_name().to_owned(),
                                column: #ident_str.to_string(),
                            }
                        }
                    }
                    SqlColumn::Column(ident) => {
                        checks.push(quote_spanned! {ident.span()=>
                            {
                                let _= ___t___.#ident;
                            }
                        });
                        let ident_str = ident.to_string();

                        quote_spanned! {ident.span()=>
                            #sql_crate::SqlExpr::Column(#ident_str.to_string())
                        }
                    }
                }
            }
            SqlValue::Lit(lit) => {
                let debug_str = format!(
                    "Failed to bind `{}` to query",
                    readable_token_stream(&lit.to_token_stream().to_string())
                );
                binds.push(quote_spanned! {lit.span()=>
                    __easy_sql_builder.bind(#lit).context(#debug_str)?;
                });

                quote_spanned! {lit.span()=>
                    #sql_crate::SqlExpr::Value
                }
            }
            SqlValue::OutsideVariable(expr) => {
                let debug_str = format!(
                    "Failed to bind `{}` to query",
                    readable_token_stream(&expr.to_token_stream().to_string())
                );
                binds.push(quote_spanned! {expr.span()=>
                    __easy_sql_builder.bind({#expr}).context(#debug_str)?;
                });

                quote_spanned! {expr.span()=>
                    #sql_crate::SqlExpr::Value
                }
            }
        }
    }
}
#[derive(Debug, Clone)]
pub enum SqlValueIn {
    Single(SqlValue),
    Multiple(Vec<SqlValue>),
}

#[always_context]
impl SqlValue {
    fn lookahead(l: &Lookahead1<'_>) -> bool {
        l.peek(syn::Ident) || l.peek(syn::Lit) || l.peek(syn::token::Brace)
    }
}

#[always_context]
impl Parse for SqlValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
            Ok(SqlValue::Column(input.parse()?))
        } else if lookahead.peek(syn::Lit) {
            let lit: syn::Lit = input.parse()?;
            Ok(SqlValue::Lit(lit))
        } else if lookahead.peek(syn::token::Brace) {
            let inside_braces;
            syn::braced!(inside_braces in input);
            let expr: syn::Expr = inside_braces.parse()?;
            Ok(SqlValue::OutsideVariable(expr))
        } else {
            Err(lookahead.error())
        }
    }
}

#[always_context]
impl Parse for SqlValueIn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Paren) {
            let inside_paren;
            syn::parenthesized!(inside_paren in input);
            let mut values = Vec::new();
            while !inside_paren.is_empty() {
                let value = inside_paren.parse::<SqlValue>()?;
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
            Ok(SqlValueIn::Multiple(values))
        } else if SqlValue::lookahead(&lookahead) {
            let value = input.parse::<SqlValue>()?;
            Ok(SqlValueIn::Single(value))
        } else {
            Err(lookahead.error())
        }
    }
}

fn continue_parse_value_no_expr(
    input: syn::parse::ParseStream,
    current_value: SqlValue,
    lookahead: syn::parse::Lookahead1<'_>,
) -> syn::Result<SqlExpr> {
    if input.is_empty() || next_clause_token(&lookahead) {
        return Ok(SqlExpr::Value(current_value));
    }

    if lookahead.peek(sql_keyword::is) {
        input.parse::<sql_keyword::is>()?;
        let lookahead2 = input.lookahead1();
        if lookahead2.peek(sql_keyword::not) {
            input.parse::<sql_keyword::not>()?;
            let lookahead3 = input.lookahead1();
            if lookahead3.peek(sql_keyword::null) {
                input.parse::<sql_keyword::null>()?;
                Ok(SqlExpr::IsNotNull(current_value))
            } else {
                Err(lookahead3.error())
            }
        } else if lookahead2.peek(sql_keyword::null) {
            input.parse::<sql_keyword::null>()?;
            Ok(SqlExpr::IsNull(current_value))
        } else {
            Err(lookahead2.error())
        }
    } else if lookahead.peek(syn::Token![=]) {
        input.parse::<syn::Token![=]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(SqlExpr::Equal(current_value, right_value))
    } else if lookahead.peek(syn::Token![!=]) {
        input.parse::<syn::Token![!=]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(SqlExpr::NotEqual(current_value, right_value))
    } else if lookahead.peek(syn::Token![>=]) {
        input.parse::<syn::Token![>=]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(SqlExpr::GreaterThanOrEqual(current_value, right_value))
    } else if lookahead.peek(syn::Token![>]) {
        input.parse::<syn::Token![>]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(SqlExpr::GreaterThan(current_value, right_value))
    } else if lookahead.peek(syn::Token![<=]) {
        input.parse::<syn::Token![<=]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(SqlExpr::LessThanOrEqual(current_value, right_value))
    } else if lookahead.peek(syn::Token![<]) {
        input.parse::<syn::Token![<]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(SqlExpr::LessThan(current_value, right_value))
    } else if lookahead.peek(sql_keyword::like) {
        input.parse::<sql_keyword::like>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(SqlExpr::Like(current_value, right_value))
    } else if lookahead.peek(sql_keyword::in_) {
        input.parse::<sql_keyword::in_>()?;
        let right_value = input.parse::<SqlValueIn>()?;
        Ok(SqlExpr::In(current_value, right_value))
    } else if lookahead.peek(sql_keyword::between) {
        input.parse::<sql_keyword::between>()?;
        let middle_value = input.parse::<SqlValue>()?;
        let lookahead2 = input.lookahead1();
        if lookahead2.peek(sql_keyword::and) {
            input.parse::<sql_keyword::and>()?;
            let right_value = input.parse::<SqlValue>()?;
            Ok(SqlExpr::Between(current_value, middle_value, right_value))
        } else {
            Err(lookahead2.error())
        }
    } else {
        Err(lookahead.error())
    }
}

fn continue_parse_value_maybe_expr(
    input: syn::parse::ParseStream,
    current_value: SqlValue,
) -> syn::Result<SqlExpr> {
    if input.is_empty() {
        return Ok(SqlExpr::Value(current_value));
    }

    let lookahead = input.lookahead1();

    if lookahead.peek(sql_keyword::and)
        || lookahead.peek(sql_keyword::or)
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
        // We handle operators in the SqlExpr::parse method
        Ok(SqlExpr::Value(current_value))
    } else {
        continue_parse_value_no_expr(input, current_value, lookahead)
    }
}

fn sub_where_expr(input: syn::parse::ParseStream) -> syn::Result<SqlExpr> {
    let lookahead = input.lookahead1();

    if lookahead.peek(sql_keyword::not) {
        input.parse::<sql_keyword::not>()?;

        let expr = sub_where_expr(input)?;
        Ok(SqlExpr::Not(Box::new(expr)))
    } else if lookahead.peek(syn::token::Paren) {
        let inside_paren;
        syn::parenthesized!(inside_paren in input);
        let expr = inside_paren.parse::<SqlExpr>()?;
        Ok(SqlExpr::Parenthesized(Box::new(expr)))
    } else if SqlValue::lookahead(&lookahead) {
        let parsed = input.parse::<SqlValue>()?;

        Ok(continue_parse_value_maybe_expr(input, parsed)?)
    } else {
        Err(lookahead.error())
    }
}

#[always_context]
impl Parse for SqlExpr {
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
            Ok(SqlExpr::OperatorChain(Box::new(first_expr), next_exprs))
        }
    }
}
