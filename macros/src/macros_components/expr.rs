use std::fmt::Display;

use super::{CollectedData, builtin_functions};
use crate::macros_components::keyword::{DoubleArrow};

use super::{column::Column, next_clause::next_clause_token};

use super::keyword;
use ::{
    proc_macro2::{self},
    syn::{self, parse::Parse, spanned::Spanned},
};
use easy_macros::always_context;
use convert_case::{Case, Casing};
use quote::{IdentFragment, ToTokens, format_ident, quote, quote_spanned};
use syn::ext::IdentExt;

syn::custom_punctuation!(NotEqualsMicrosoft,<>);

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
    ///->>
    JsonExtractText,
    /// &
    BitAnd,
    /// |
    BitOr,
    /// <<
    BitShiftLeft,
    /// >>
    BitShiftRight,
    /// = or ==
    Equal,
    /// != or <>
    NotEqual,
    /// >
    GreaterThan,
    /// >=
    GreaterThanOrEqual,
    /// <
    LessThan,
    /// <=
    LessThanOrEqual,
    /// LIKE
    Like,
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
        } else if input.peek(syn::Token![-]) && input.peek2(syn::Token![>>]) {
            input.parse::<syn::Token![-]>()?;
            input.parse::<syn::Token![>>]>()?;
            Ok(Operator::JsonExtractText)
        }else if lookahead.peek(syn::Token![->]) {
            input.parse::<syn::Token![->]>()?;
            Ok(Operator::JsonExtract)
        }  else if input.peek(syn::Token![-]) && input.peek2(syn::Token![>]) {
            input.parse::<syn::Token![-]>()?;
            input.parse::<syn::Token![>]>()?;
            Ok(Operator::JsonExtract)
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
        } else if lookahead.peek(syn::Token![=]) {
            input.parse::<syn::Token![=]>()?;
            Ok(Operator::Equal)
        } else if lookahead.peek(syn::Token![==]) {
            input.parse::<syn::Token![==]>()?;
            Ok(Operator::Equal)
        } else if lookahead.peek(syn::Token![!=]) {
            input.parse::<syn::Token![!=]>()?;
            Ok(Operator::NotEqual)
        } else if lookahead.peek(NotEqualsMicrosoft) {
            input.parse::<NotEqualsMicrosoft>()?;
            Ok(Operator::NotEqual)
        } else if lookahead.peek(syn::Token![>=]) {
            input.parse::<syn::Token![>=]>()?;
            Ok(Operator::GreaterThanOrEqual)
        } else if lookahead.peek(syn::Token![<=]) {
            input.parse::<syn::Token![<=]>()?;
            Ok(Operator::LessThanOrEqual)
        } else if lookahead.peek(syn::Token![<]) {
            input.parse::<syn::Token![<]>()?;
            Ok(Operator::LessThan)
        } else if lookahead.peek(syn::Token![>]) {
            input.parse::<syn::Token![>]>()?;
            Ok(Operator::GreaterThan)
        } else if lookahead.peek(keyword::like) {
            input.parse::<keyword::like>()?;
            Ok(Operator::Like)
        } else {
            Err(lookahead.error())
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct NotChain {
    pub not_count: usize,
}

impl syn::parse::Parse for NotChain {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut not_count = 0;
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(keyword::not) {
                input.parse::<keyword::not>()?;
                not_count += 1;
            } else {
                break;
            }
        }
        Ok(NotChain { not_count })
    }
}

impl NotChain {
    pub fn into_query_string(self) -> String {
        let mut current_query = String::new();
        for _ in 0..self.not_count {
            current_query.push_str("NOT ");
        }
        current_query
    }
}

fn add_operator_support_check<T>(
    data: &mut CollectedData,
    operator_in_trait_name: T,
) where T:IdentFragment,for<'a> &'a T: Display+IdentFragment{
    let sql_crate = data.sql_crate;
    let trait_ident= format_ident!("Supports{}", operator_in_trait_name);
    for driver_ty in data.driver.iter_for_checks() {
        data.checks.push(quote_spanned! {proc_macro2::Span::call_site()=>
            {
                fn __easy_sql_assert_supports_operator<T: #sql_crate::markers::operators::#trait_ident>() {}
                __easy_sql_assert_supports_operator::<#driver_ty>();
            }
        });
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Box<Value>),
    Parenthesized(Box<Expr>),
    OperatorChain(NotChain, Box<Expr>, Vec<(NotChain, Operator, Expr)>),
    IsNull(Box<Value>),
    IsNotNull(Box<Value>),
    In(Box<Value>, Box<ValueIn>),
    Between(Box<Value>, Box<Value>, Box<Value>),
}

#[always_context]
impl Expr {
    ///`main_table_type` - None means we're inside of table join
    pub fn into_query_string(
        self,
        data: &mut CollectedData,
        inside_count_fn: bool,
        for_custom_select: bool,
    ) -> String {
        match self {
            Expr::Value(val) => val.into_query_string(
                data,
                inside_count_fn,
                for_custom_select,
            ),
            Expr::IsNull(val) => {
                add_operator_support_check(
                    data,
                    "IsNull",
                );
                let val_sql = val.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                format!("{} IS NULL", val_sql)
            }
            Expr::IsNotNull(val) => {
                add_operator_support_check(
                    data,
                    "IsNotNull",
                );
                let val_sql = val.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                format!("{} IS NOT NULL", val_sql)
            }
            Expr::Parenthesized(inner) => {
                let inner_sql = inner.into_query_string(
                    data,
                    inside_count_fn,
                    for_custom_select,
                );
                format!("({})", inner_sql)
            }
            Expr::OperatorChain(not_chain, first, rest) => {
                let mut result = format!(
                    "{}{}",
                    not_chain.into_query_string(),
                    first.into_query_string(
                        data,
                        false,
                        for_custom_select,
                    )
                );
                for (not_chain, op, expr) in rest.into_iter() {
                    let op_trait_name = match &op {
                        Operator::Mod|Operator::Concat => format!("{:?}Operator",op),
                        op=>format!("{:?}",op)
                    };
                    add_operator_support_check(
                        data,
                        op_trait_name,
                    );
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
                        Operator::JsonExtractText => " ->> ",
                        Operator::BitAnd => " & ",
                        Operator::BitOr => " | ",
                        Operator::BitShiftLeft => " << ",
                        Operator::BitShiftRight => " >> ",
                        Operator::Equal => " = ",
                        Operator::NotEqual => " != ",
                        Operator::GreaterThan => " > ",
                        Operator::GreaterThanOrEqual => " >= ",
                        Operator::LessThan => " < ",
                        Operator::LessThanOrEqual => " <= ",
                        Operator::Like => " LIKE ",
                    };
                    result.push_str(op_str);
                    result.push_str(&not_chain.into_query_string());
                    result.push_str(&expr.into_query_string(
                        data,
                        false,
                        for_custom_select,
                    ));
                }

                result
            }
            Expr::In(val, values) => {
                add_operator_support_check(
                    data,
                    "In",
                );
                let val_sql = val.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                match *values {
                    ValueIn::Multiple(vals) => {
                        let mut in_items = Vec::new();
                        for v in vals.into_iter() {
                            in_items.push(v.into_query_string(
                                data,
                                false,
                                for_custom_select,
                            ));
                        }
                        format!("{} IN ({})", val_sql, in_items.join(", "))
                    }
                    ValueIn::SingleColumn(col) => {
                        // Single column reference - convert to Value and process
                        let col_value = Value::Column(col.clone());
                        let col_sql = col_value.into_query_string(
                            data,
                            false,
                            for_custom_select,
                        );
                        format!("{} IN ({})", val_sql, col_sql)
                    }
                    ValueIn::SingleVar(v) => {
                        // Generate dynamic placeholder list based on the runtime length of the collection
                        let debug_str = format!(
                            "Failed to bind items from `{}` to query parameters",
                            v.to_token_stream()
                        );

                        let param_start = *data.current_param_n;

                        // Create a runtime binding and placeholder generation for the collection
                        data.binds.push(quote::quote_spanned! {v.span()=>
                            #[allow(unused_parens)]
                            for __easy_sql_in_item in (#v) {
                                _easy_sql_args.add(__easy_sql_in_item).map_err(anyhow::Error::from_boxed).context(#debug_str)?;
                            }
                        });

                        let format_param_n = data.format_params.len();

                        let before_param_n_name =
                            format_ident!("__easy_sql_before_param_n_{}", format_param_n);
                        let before_param_n = &mut data.before_param_n;

                        data.before_format.push(quote! {
                            let #before_param_n_name:usize;
                        });

                        let parameter_placeholder_call =
                            data.driver.parameter_placeholder_fn(data.sql_crate, v.span());

                        // Create format parameter that generates placeholders at runtime
                        data.format_params.push(quote::quote_spanned! {v.span()=>
                            
                            {
                                #[allow(clippy::needless_borrow)]
                                {
                                    #before_param_n_name = (#v).len();
                                    let mut __easy_sql_in_placeholders = Vec::with_capacity(#before_param_n_name);
                                    for __easy_sql_in_i in 0..#before_param_n_name {
                                        __easy_sql_in_placeholders.push(
                                            #parameter_placeholder_call(#before_param_n #param_start + __easy_sql_in_i)
                                        );
                                    }
                                    __easy_sql_in_placeholders.join(", ")
                                }
                            }
                        });


                        **before_param_n = quote! {#before_param_n_name + #before_param_n};

                        format!("{} IN ({{}})", val_sql)
                    }
                }
            }
            Expr::Between(val, min, max) => {
                add_operator_support_check(
                    data,
                    "Between",
                );
                let val_sql = val.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                let min_sql = min.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                let max_sql = max.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                format!("({} BETWEEN {} AND {})", val_sql, min_sql, max_sql)
            }
        }
    }

    /// Check if this expression contains any outside variables ({arg0}, {arg1}, etc.)
    /// Returns a set of argument indices found in the expression.
    pub fn collect_indices_impl(&self, indices: &mut std::collections::BTreeSet<usize>) {
        match self {
            Expr::Value(v) => v.collect_indices_impl(indices),
            Expr::Parenthesized(inner) => inner.collect_indices_impl(indices),
            Expr::OperatorChain(_, first, chain) => {
                first.collect_indices_impl(indices);
                for (_, _, expr) in chain.iter() {
                    expr.collect_indices_impl(indices);
                }
            }
            Expr::IsNull(v) | Expr::IsNotNull(v) => v.collect_indices_impl(indices),
            Expr::In(v, value_in) => {
                v.collect_indices_impl(indices);
                match &**value_in {
                    ValueIn::SingleVar(_) | ValueIn::SingleColumn(_) => {}
                    ValueIn::Multiple(exprs) => {
                        for e in exprs.iter() {
                            e.collect_indices_impl(indices);
                        }
                    }
                }
            }
            Expr::Between(v, low, high) => {
                v.collect_indices_impl(indices);
                low.collect_indices_impl(indices);
                high.collect_indices_impl(indices);
            }
        }
    }
}
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

#[always_context]
impl Value {
    fn lookahead(input: &syn::parse::ParseStream) -> bool {
        // Check for literals and braces using lookahead
        let lookahead = input.lookahead1();
        lookahead.peek(syn::Lit)
            || lookahead.peek(syn::token::Brace)
            || lookahead.peek(syn::Ident::peek_any)
    }

    fn function_call_start(
        input: &syn::parse::ParseStream,
    ) -> syn::Result<Option<proc_macro2::Ident>> {
        if input.peek(syn::Ident::peek_any) && input.peek2(syn::token::Paren) {
            let ident = input.call(syn::Ident::parse_any)?;
            Ok(Some(ident))
        } else {
            Ok(None)
        }
    }
    ///`main_table_type` - None means we're inside of table join
    fn into_query_string(
        self,
        data: &mut CollectedData,
        inside_count: bool,
        for_custom_select: bool,
    ) -> String {
        let sql_crate= data.sql_crate;
        match self {
            Value::Column(col) => col.into_query_string(data, for_custom_select),
            Value::Lit(lit) => {
                // In custom select mode, embed literals directly in the SQL string
                if for_custom_select {
                    match lit {
                        syn::Lit::Str(lit_str) => {
                            // SQL string literals use single quotes and need escaping
                            let value = lit_str.value();
                            let escaped = value.replace("'", "''"); // SQL escape single quotes
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
                            // For other literal types, convert to string representation
                            return lit.to_token_stream().to_string();
                        }
                    }
                }

                // Normal mode: use bind parameters
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
                // Check if this is an {argN} pattern for custom select
                if for_custom_select {
                    let expr_val_span = expr_val.span();
                    if let syn::Expr::Path(expr_path) = expr_val
                        && expr_path.path.segments.len() == 1 {
                            let ident = &expr_path.path.segments[0].ident;
                            let ident_str = ident.to_string();
                            if let Some(stripped) = ident_str.strip_prefix("arg")
                                && let Ok(_idx) = stripped.parse::<usize>() {
                                    // This is an {argN} placeholder - push the identifier as a variable reference
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
                    // Inside table join - outside variables are not allowed
                    data.checks.push(quote::quote_spanned! {expr_val.span()=>
                        {
                            compile_error!("Outside variables are not allowed inside of JOIN clauses. (yet)");
                        }
                    });
                    return "{}".to_string();
                }

                // Normal outside variable handling
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

                let arg_sql = expr.into_query_string(
                    data,
                    inside_count,
                    for_custom_select,
                );
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

                    (
                        func_name.clone(),
                        func_name == "COUNT",
                    )
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
                

                if let Some(args)=args{
                    for arg in args {
                    let arg_sql = arg.into_query_string(
                        data,
                        is_count,
                        for_custom_select,
                    );
                    arg_strings.push(arg_sql);
                }
                format!("{}({})", func_name, arg_strings.join(", "))
                }else{
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

    /// Check if this value contains an outside variable ({arg0}, {arg1}, etc.)
    /// Collects argument indices into the provided set.
    fn collect_indices_impl(&self, indices: &mut std::collections::BTreeSet<usize>) {
        match self {
            Value::OutsideVariable(expr) => {
                // Extract index from {argN} pattern
                if let syn::Expr::Path(expr_path) = expr
                    && expr_path.path.segments.len() == 1 {
                        let ident_str = expr_path.path.segments[0].ident.to_string();
                        if let Some(stripped) = ident_str.strip_prefix("arg")
                            && let Ok(idx) = stripped.parse::<usize>() {
                                indices.insert(idx);
                            }
                    }
            }
            Value::FunctionCall { args:Some(args), .. } => {
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
#[derive(Debug, Clone)]
pub enum ValueIn {
    SingleVar(syn::Expr),
    SingleColumn(Column),
    Multiple(Vec<Expr>),
}

#[always_context]
impl Parse for Value {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // First, check for zero-arg builtin functions without parentheses (aka SQL Value)
        if input.peek(syn::Ident::peek_any) && !input.peek2(syn::token::Paren) {
            let lookahead = input.fork();
            let ident = lookahead.call(syn::Ident::parse_any)?;
            if let Some(builtin_fn_data) = builtin_functions::get_builtin_fn(&ident.to_string())
                && builtin_fn_data.maybe_value
            {
                let ident = input.call(syn::Ident::parse_any)?;
                return Ok(Value::FunctionCall {
                    name: ident,
                    args: None,
                });
            }
        }

        // Next, check if this is a function call (which may include Rust keywords)
        if let Some(func_name) = Value::function_call_start(&input)? {
            // Parse the arguments
            let inside_paren;
            syn::parenthesized!(inside_paren in input);

            let mut args = Vec::new();

            let func_name_str = func_name.to_string();
            let builtin_fn_data = builtin_functions::get_builtin_fn(&func_name_str);

            if func_name_str.eq_ignore_ascii_case("cast") {
                let expr = sub_where_expr(&inside_paren)?;
                if inside_paren.is_empty() {
                    return Err(inside_paren.error("CAST expects syntax: CAST(expr AS Type)"));
                }
                inside_paren.parse::<keyword::as_kw>()?;
                let ty: syn::Type = inside_paren.parse()?;
                if !inside_paren.is_empty() {
                    return Err(inside_paren.error("CAST expects syntax: CAST(expr AS Type)"));
                }
                return Ok(Value::Cast {
                    expr: Box::new(expr),
                    ty,
                });
            }

            if !inside_paren.is_empty() {
                let lookahead_star = inside_paren.lookahead1();
                if lookahead_star.peek(syn::Token![*]) {
                    // Check for special case: COUNT(*) with zero regular arguments
                    //
                    // This is a star argument - check if function accepts it
                    let func_name_str = func_name.to_string();
                    if !builtin_fn_data
                        .map(|data| data.accepts_star)
                        .unwrap_or(false)
                    {
                        return Err(syn::Error::new(
                            func_name.span(),
                            format!(
                                "Function {} does not accept * as an argument",
                                func_name_str.to_uppercase()
                            ),
                        ));
                    }

                    let star_token = inside_paren.parse::<syn::Token![*]>()?;

                    // Add star as an Expr::Value(Value::Star)
                    args.push(Expr::Value(Box::new(Value::Star(star_token))));

                    // No comma after star for COUNT(*)
                } else {
                    // Regular arguments
                    while !inside_paren.is_empty() {
                        let arg = sub_where_expr(&inside_paren)?;
                        args.push(arg);

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
                }
            }

            Ok(Value::FunctionCall {
                name: func_name,
                args:Some(args),
            })
        } else {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::Lit) {
                let lit: syn::Lit = input.parse()?;
                Ok(Value::Lit(lit))
            } else if lookahead.peek(syn::token::Brace) {
                let inside_braces;
                syn::braced!(inside_braces in input);
                let expr: syn::Expr = inside_braces.parse()?;
                Ok(Value::OutsideVariable(expr))
            } else if lookahead.peek(syn::Ident) {
                // Not a function call, parse as column
                Ok(Value::Column(input.parse()?))
            } else {
                Err(lookahead.error())
            }
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
                let value = sub_where_expr(&inside_paren)?;
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
        } else if lookahead.peek(syn::Ident) {
            // Could be a column reference or the start of a path
            Ok(ValueIn::SingleColumn(input.parse()?))
        } else if lookahead.peek(syn::token::Brace) {
            // This is a variable in braces: {some_var}
            let inside_braces;
            syn::braced!(inside_braces in input);
            let expr: syn::Expr = inside_braces.parse()?;
            Ok(ValueIn::SingleVar(expr))
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
        return Ok(Expr::Value(Box::new(current_value)));
    }

    if lookahead.peek(keyword::is) {
        input.parse::<keyword::is>()?;
        let lookahead2 = input.lookahead1();
        if lookahead2.peek(keyword::not) {
            input.parse::<keyword::not>()?;
            let lookahead3 = input.lookahead1();
            if lookahead3.peek(keyword::null) {
                input.parse::<keyword::null>()?;
                Ok(Expr::IsNotNull(Box::new(current_value)))
            } else {
                Err(lookahead3.error())
            }
        } else if lookahead2.peek(keyword::null) {
            input.parse::<keyword::null>()?;
            Ok(Expr::IsNull(Box::new(current_value)))
        } else {
            Err(lookahead2.error())
        }
    } else if lookahead.peek(keyword::in_) {
        input.parse::<keyword::in_>()?;
        let right_value = input.parse::<ValueIn>()?;
        Ok(Expr::In(Box::new(current_value), Box::new(right_value)))
    } else if lookahead.peek(keyword::between) {
        input.parse::<keyword::between>()?;
        let middle_value = input.parse::<Value>()?;
        let lookahead2 = input.lookahead1();
        if lookahead2.peek(keyword::and) {
            input.parse::<keyword::and>()?;
            let right_value = input.parse::<Value>()?;
            Ok(Expr::Between(Box::new(current_value), Box::new(middle_value), Box::new(right_value)))
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
        return Ok(Expr::Value(Box::new(current_value)));
    }

    let lookahead = input.lookahead1();

    if lookahead.peek(keyword::and)
        || lookahead.peek(keyword::or)
    || lookahead.peek(syn::Token![+])
    || lookahead.peek(DoubleArrow)
    || lookahead.peek(syn::Token![->])
    || (input.peek(syn::Token![-]) && input.peek2(syn::Token![>>]))
    || (input.peek(syn::Token![-]) && input.peek2(syn::Token![>]))
    || lookahead.peek(syn::Token![-])
        || lookahead.peek(syn::Token![*])
        || lookahead.peek(syn::Token![/])
        || lookahead.peek(syn::Token![%])
        || lookahead.peek(syn::Token![||])
        || lookahead.peek(syn::Token![&])
        || lookahead.peek(syn::Token![|])
        || lookahead.peek(syn::Token![<<])
        || lookahead.peek(syn::Token![>>])
        // NEW
        || lookahead.peek(syn::Token![=])
        || lookahead.peek(syn::Token![!=])
        || lookahead.peek(syn::Token![>=])
        || lookahead.peek(syn::Token![>])
        || lookahead.peek(syn::Token![<=])
        || lookahead.peek(syn::Token![<])
        || lookahead.peek(keyword::like)
    {
        // We handle operators in the Expr::parse method
        Ok(Expr::Value(Box::new(current_value)))
    } else {
        continue_parse_value_no_expr(input, current_value, lookahead)
    }
}

fn sub_where_expr(input: syn::parse::ParseStream) -> syn::Result<Expr> {
    let lookahead = input.lookahead1();

    if lookahead.peek(syn::token::Paren) {
        let inside_paren;
        syn::parenthesized!(inside_paren in input);
        let expr = inside_paren.parse::<Expr>()?;
        Ok(Expr::Parenthesized(Box::new(expr)))
    } else if Value::lookahead(&input) {
        let parsed = input.parse::<Value>()?;

        Ok(continue_parse_value_maybe_expr(input, parsed)?)
    } else {
        #[allow(unused_mut)]
        let mut err = lookahead.error();
        #[cfg(feature = "parse_debug")]
        err.combine(
            input.error("lookahead.peek(syn::token::Paren) && Value::lookahead(&input) failed"),
        );
        Err(err)
    }
}

#[always_context]
impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut first_expr = None;
        let mut first_not_chain = None;
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

            let not_chain: NotChain = input.parse()?;

            #[allow(unused_mut)]
            #[allow(clippy::map_identity)]
            let current_expr = sub_where_expr(input).map_err(|mut e| {
                #[cfg(feature = "parse_debug")]
                e.combine(input.error("sub_where_expr"));
                e
            })?;

            if let Some(and_or) = and_or {
                next_exprs.push((not_chain, and_or, current_expr));
            } else {
                first_expr = Some(current_expr);
                first_not_chain = Some(not_chain);
            }
        }

        let (first_expr, first_not_chain) = if let (Some(first_expr), Some(first_not_chain)) =
            (first_expr, first_not_chain)
        {
            (first_expr, first_not_chain)
        } else {
            return Err(input.error("Expected a valid where expression, if you don't want to use any conditions, use `true`"));
        };

        if next_exprs.is_empty() {
            // Check if we have a NOT chain even without operator chains
            if first_not_chain.not_count > 0 {
                Ok(Expr::OperatorChain(
                    first_not_chain,
                    Box::new(first_expr),
                    vec![],
                ))
            } else {
                Ok(first_expr)
            }
        } else {
            Ok(Expr::OperatorChain(
                first_not_chain,
                Box::new(first_expr),
                next_exprs,
            ))
        }
    }
}
