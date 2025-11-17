use crate::macros_components::keyword;

use ::{
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
};
use easy_macros::{
    all_syntax_cases, always_context, parse_macro_input, token_stream_to_consistent_string,
};

#[derive(Debug, Default)]
struct SqlData {
    potential_table: Option<TokenStream>,
    update_set_arg: bool,
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

fn is_update_check(fn_path: &syn::Path) -> bool {
    let last_segment = if let Some(s) = fn_path.segments.last() {
        s
    } else {
        return false;
    };
    let last_segment_str = last_segment.ident.to_string();

    match last_segment_str.as_str() {
        "update" | "update_returning" | "update_returning_lazy" => true,
        _ => false,
    }
}

fn call_check(item: &mut syn::ExprCall, context_info: &mut SqlData) {
    let mut reset_after = false;
    let mut is_update = false;
    match &*item.func {
        syn::Expr::Path(expr_path) => {
            let before = context_info.potential_table.is_none();
            let potential = table_function_check(&expr_path.path);
            if before && potential.is_some() {
                context_info.potential_table = potential;
                is_update = is_update_check(&expr_path.path);
                reset_after = true;
            }
        }
        _ => {}
    }

    for (i, arg) in item.args.iter_mut().enumerate() {
        if i == 1 && is_update {
            context_info.update_set_arg = true;
        }
        macro_search_expr_handle(arg, context_info);
        context_info.update_set_arg = false;
    }

    if reset_after {
        context_info.potential_table = None;
    }
}

struct SqlMacroInput {
    driver: Option<syn::Path>,
    table: Option<syn::Type>,
    set_keyword_present: bool,
    leftovers: proc_macro2::TokenStream,
}

impl syn::parse::Parse for SqlMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut driver = None;
        let mut table = None;
        let mut set_keyword_present = false;

        // Check for optional driver specification: <Driver>
        if input.peek(syn::Token![<]) {
            input.parse::<syn::Token![<]>()?;
            driver = Some(input.parse::<syn::Path>()?);
            input.parse::<syn::Token![>]>()?;
        }

        if input.peek(syn::Token![|]) {
            input.parse::<syn::Token![|]>()?;
            let ty: syn::Type = input.parse()?;
            table = Some(ty);
            input.parse::<syn::Token![|]>()?;
        }

        if input.peek(keyword::set) {
            input.parse::<keyword::set>()?;
            set_keyword_present = true;
        }

        let leftovers: proc_macro2::TokenStream = input.parse()?;

        Ok(SqlMacroInput {
            driver,
            table,
            set_keyword_present,
            leftovers,
        })
    }
}

fn macro_check(item: &mut syn::Macro, context_info: &mut SqlData) {
    let path = token_stream_to_consistent_string(item.path.to_token_stream());
    match path.as_str() {
        "sql" | "sql_debug" | "easy_sql::sql" | "easy_sql::sql_debug" => {
            let macro_input: SqlMacroInput = if let Ok(r) = syn::parse2(item.tokens.clone()) {
                r
            } else {
                //Error should be produced by the macro itself
                return;
            };

            let table = macro_input
                .table
                .as_ref()
                .map(|t| t.to_token_stream())
                .or_else(|| context_info.potential_table.clone());

            if table.is_none() {
                //Error should be produced by the macro itself
                return;
            }

            let driver = macro_input.driver.as_ref();

            let driver_tokens = if let Some(d) = driver {
                quote! { <#d> }
            } else {
                quote! {}
            };

            let set = if macro_input.set_keyword_present || context_info.update_set_arg {
                quote! { SET }
            } else {
                quote! {}
            };
            let leftovers = macro_input.leftovers;

            item.tokens = quote! {
                #driver_tokens | #table | #set #leftovers
            };
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
