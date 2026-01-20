use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, LitStr, Token, parse_macro_input};

/// Represents the argument count specification for a SQL function
#[derive(Debug, Clone)]
pub enum ArgCount {
    /// Exactly N arguments
    Exact(usize),
    /// Any number of arguments
    Any,
    /// Multiple specific counts allowed (e.g., 1 or 2 arguments)
    Multiple(Vec<usize>),
}

impl Parse for ArgCount {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(syn::Ident) {
            let ident: Ident = input.parse()?;
            if ident == "Any" {
                Ok(ArgCount::Any)
            } else {
                Err(syn::Error::new(ident.span(), "Expected 'Any' or a number"))
            }
        } else if lookahead.peek(LitInt) {
            let first: LitInt = input.parse()?;
            let first_val = first.base10_parse::<usize>()?;

            // Check if there are more numbers (separated by |)
            let mut values = vec![first_val];

            while input.peek(Token![|]) {
                input.parse::<Token![|]>()?;
                let next: LitInt = input.parse()?;
                values.push(next.base10_parse()?);
            }

            if values.len() == 1 {
                Ok(ArgCount::Exact(values[0]))
            } else {
                Ok(ArgCount::Multiple(values))
            }
        } else {
            Err(lookahead.error())
        }
    }
}

/// Input for the custom_sql_function macro
///
/// Syntax: custom_sql_function!(FunctionName; "SQL_FUNCTION_NAME"; 1 | 2 | Any)
struct SqlFunctionInput {
    struct_name: Ident,
    sql_name: String,
    arg_count: ArgCount,
}

impl Parse for SqlFunctionInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse: FunctionName;
        let struct_name: Ident = input.parse()?;
        input.parse::<Token![;]>()?;

        // Parse: "FUNCTION_NAME";
        let sql_name_lit: LitStr = input.parse()?;
        let sql_name = sql_name_lit.value();
        input.parse::<Token![;]>()?;

        // Parse: N | M | Any
        let arg_count: ArgCount = input.parse()?;
        if !input.is_empty() {
            input.parse::<Token![;]>()?;
        }

        Ok(SqlFunctionInput {
            struct_name,
            sql_name,
            arg_count,
        })
    }
}

pub fn custom_sql_function_impl(input: TokenStream) -> TokenStream {
    let SqlFunctionInput {
        struct_name,
        sql_name,
        arg_count,
    } = parse_macro_input!(input as SqlFunctionInput);

    let function_name = struct_name.to_string().to_lowercase();
    let macro_name = Ident::new(&function_name, struct_name.span());
    let sql_name_lit = LitStr::new(&sql_name, struct_name.span());

    let arg_error_msg = match &arg_count {
        ArgCount::Exact(count) => format!("{} expects exactly {} argument(s)", sql_name, count),
        ArgCount::Multiple(values) => format!(
            "{} expects {} argument(s)",
            sql_name,
            values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" or ")
        ),
        ArgCount::Any => String::new(),
    };

    let build_arm_for_count = |count: usize| {
        if count == 0 {
            quote! {
                () => ( #sql_name_lit );
            }
        } else {
            let arg_idents: Vec<Ident> = (0..count)
                .map(|idx| Ident::new(&format!("arg{}", idx), struct_name.span()))
                .collect();
            let arg_patterns = arg_idents.iter().map(|ident| quote! { $ #ident : expr });
            quote! {
                (#(#arg_patterns),* $(,)?) => ( #sql_name_lit );
            }
        }
    };

    let macro_arms = match &arg_count {
        ArgCount::Exact(count) => {
            let arm = build_arm_for_count(*count);
            quote! {
                #arm
                ($($args:expr),* $(,)?) => ( compile_error!(#arg_error_msg) );
            }
        }
        ArgCount::Multiple(values) => {
            let arms = values.iter().map(|count| build_arm_for_count(*count));
            quote! {
                #(#arms)*
                ($($args:expr),* $(,)?) => ( compile_error!(#arg_error_msg) );
            }
        }
        ArgCount::Any => {
            quote! {
                ($($args:expr),* $(,)?) => ( #sql_name_lit );
            }
        }
    };

    let output = quote! {
        /// Macro used by `query!` to emit the SQL function name and validate argument count.
        #[macro_export]
        macro_rules! #macro_name {
            #macro_arms
        }

    };

    output.into()
}
