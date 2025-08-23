use easy_macros::{
    anyhow,
    helpers::parse_macro_input,
    macros::{all_syntax_cases, always_context},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::{self},
};

#[derive(Debug, Default)]
struct SqlData {
    potential_table: Option<TokenStream>,
}

all_syntax_cases! {
    setup=>{
        generated_fn_prefix:"macro_search",
        additional_input_type:&mut SqlData
    }
    default_cases=>{
        fn macro_check(item: &mut syn::Macro, context_info: &mut SqlData);
    }
    special_cases=>{
        fn call_check(item: &mut syn::ExprCall, context_info: &mut SqlData);
    }
}

fn table_function_check(fn_path: &syn::Path) -> Option<TokenStream> {
    let last_segment = if let Some(s) = fn_path.segments.last() {
        s
    } else {
        return None;
    };
    let last_segment_str = last_segment.ident.to_string();

    match last_segment_str.as_str() {
        "select"
        | "get"
        | "select_lazy"
        | "get_lazy"
        | "exists"
        | "insert"
        | "insert_returning"
        | "insert_returning_lazy"
        | "update"
        | "update_returning"
        | "update_returning_lazy"
        | "delete"
        | "delete_returning"
        | "delete_returning_lazy" => {
            let mut path = fn_path.clone();
            path.segments.pop();
            path.segments.pop_punct();
            Some(path.to_token_stream())
        }
        _ => None,
    }
}

fn call_check(item: &mut syn::ExprCall, context_info: &mut SqlData) {
    let mut reset_after = false;
    match &*item.func {
        syn::Expr::Path(expr_path) => {
            let before = context_info.potential_table.is_none();
            let potential = table_function_check(&expr_path.path);
            if before && potential.is_some() {
                context_info.potential_table = potential;
                reset_after = true;
            }
        }
        _ => {}
    }

    for arg in item.args.iter_mut() {
        macro_search_expr_handle(arg, context_info);
    }

    if reset_after {
        context_info.potential_table = None;
    }
}

fn macro_check(item: &mut syn::Macro, context_info: &mut SqlData) {
    let path = item.path.to_token_stream().to_string();
    match path.as_str() {
        "sql"
        | "sql_where"
        | "sql_where_debug"
        | "easy_lib::sql::sql"
        | "easy_lib::sql::sql_where"
        | "easy_lib::sql::sql_where_debug" => {
            if let Some(table) = &context_info.potential_table {
                if !item.tokens.to_string().starts_with("|") {
                    replace_with::replace_with_or_abort(&mut item.tokens, |current_tokens| {
                        quote! {
                            | #table | #current_tokens
                        }
                    });
                }
            }
        }
        _ => {}
    }
}

#[always_context]
pub fn sql_convenience(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> anyhow::Result<proc_macro::TokenStream> {
    let mut item = parse_macro_input!(item as syn::Item);

    let mut additional = Default::default();

    macro_search_item_handle(&mut item, &mut additional);

    // panic!("{}", item.to_token_stream());

    Ok(item.into_token_stream().into())
}
