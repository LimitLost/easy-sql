use easy_macros::{
    macros::{all_syntax_cases, always_context, macro_result},
    proc_macro2::TokenStream,
    quote::ToTokens,
    syn,
};

struct SqlData {
    potential_table: Option<TokenStream>,
}

all_syntax_cases! {
    setup=>{
        generated_fn_prefix:"macro_search",
        additional_input_type:&mut SearchData
    }
    default_cases=>{
        fn macro_check(item: &mut syn::Macro, context_info: &mut SearchData);
    }
    special_cases=>{}
}

fn call_check(item: &mut syn::ExprCall, context_info: &mut SearchData) {
    item.func
}

fn after_call_check(item: &mut syn::ExprCall, context_info: &mut SearchData) {
    item.func
}

fn macro_check(item: &mut syn::Macro, context_info: &mut SearchData) {
    let path = item.path.to_token_stream().to_string();
    match path.as_str() {
        "sql" | "sql_where" | "easy_lib::sql::sql" | "easy_lib::sql::sql_where" => {
            context_info.found = true;
        }
        _ => {}
    }

    item
}

#[always_context]
pub fn sql_convenience(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> anyhow::Result<proc_macro::TokenStream> {
}
