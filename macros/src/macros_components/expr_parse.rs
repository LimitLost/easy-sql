use super::{
    builtin_functions,
    expr::Expr,
    keyword,
    next_clause::next_clause_token,
    operator::{self, NotChain, Operator},
    value::{Value, ValueIn},
};
use ::syn::{self, parse::Parse};
use easy_macros::always_context;
use syn::ext::IdentExt;

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
            Ok(Expr::Between(
                Box::new(current_value),
                Box::new(middle_value),
                Box::new(right_value),
            ))
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
    if operator::starts_here(input) {
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
impl Parse for Value {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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

        if let Some(func_name) = Value::function_call_start(&input)? {
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
                    let func_name_str = func_name.to_string();
                    if !builtin_fn_data.map(|data| data.accepts_star).unwrap_or(false) {
                        return Err(syn::Error::new(
                            func_name.span(),
                            format!(
                                "Function {} does not accept * as an argument",
                                func_name_str.to_uppercase()
                            ),
                        ));
                    }

                    let star_token = inside_paren.parse::<syn::Token![*]>()?;
                    args.push(Expr::Value(Box::new(Value::Star(star_token))));
                } else {
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
                args: Some(args),
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
            Ok(ValueIn::SingleColumn(input.parse()?))
        } else if lookahead.peek(syn::token::Brace) {
            let inside_braces;
            syn::braced!(inside_braces in input);
            let expr: syn::Expr = inside_braces.parse()?;
            Ok(ValueIn::SingleVar(expr))
        } else {
            Err(lookahead.error())
        }
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

        let (first_expr, first_not_chain) =
            if let (Some(first_expr), Some(first_not_chain)) = (first_expr, first_not_chain) {
                (first_expr, first_not_chain)
            } else {
                return Err(input.error("Expected a valid where expression, if you don't want to use any conditions, use `true`"));
            };

        if next_exprs.is_empty() {
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
