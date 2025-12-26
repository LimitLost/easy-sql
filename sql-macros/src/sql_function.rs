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
/// Syntax: custom_sql_function! {
///     struct FunctionName;
///     sql_name: "SQL_FUNCTION_NAME";
///     args: 1 | 2 | Any;
/// }
struct SqlFunctionInput {
    struct_name: Ident,
    sql_name: String,
    arg_count: ArgCount,
}

impl Parse for SqlFunctionInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse: struct StructName;
        input.parse::<Token![struct]>()?;
        let struct_name: Ident = input.parse()?;
        input.parse::<Token![;]>()?;

        // Parse: sql_name: "FUNCTION_NAME";
        let sql_name_ident: Ident = input.parse()?;
        if sql_name_ident != "sql_name" {
            return Err(syn::Error::new(
                sql_name_ident.span(),
                "Expected 'sql_name'",
            ));
        }
        input.parse::<Token![:]>()?;
        let sql_name_lit: LitStr = input.parse()?;
        let sql_name = sql_name_lit.value();
        input.parse::<Token![;]>()?;

        // Parse: args: N | M | Any;
        let args_ident: Ident = input.parse()?;
        if args_ident != "args" {
            return Err(syn::Error::new(args_ident.span(), "Expected 'args'"));
        }
        input.parse::<Token![:]>()?;
        let arg_count: ArgCount = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(SqlFunctionInput {
            struct_name,
            sql_name,
            arg_count,
        })
    }
}

/// Procedural macro for defining custom SQL functions
///
/// # Example
/// ```rust
/// custom_sql_function! {
///     struct MyCustomFunc;
///     sql_name: "MY_CUSTOM_FUNCTION";
///     args: 2;
/// }
///
/// // Generates a macro_rules! macro called `my_custom_func!` that can be used in queries:
/// // query!(&mut conn, SELECT * FROM Table WHERE my_custom_func!(field1, field2) > 10)
/// ```
pub fn custom_sql_function_impl(input: TokenStream) -> TokenStream {
    let SqlFunctionInput {
        struct_name,
        sql_name,
        arg_count,
    } = parse_macro_input!(input as SqlFunctionInput);

    // Generate the macro_rules! macro name (lowercase version of struct name)
    let macro_name = Ident::new(&struct_name.to_string().to_lowercase(), struct_name.span());

    // Generate argument count validation
    let arg_validation = match arg_count {
        ArgCount::Exact(n) => {
            let error_msg = format!("{} expects exactly {} argument(s)", sql_name, n);
            quote! {
                const _: () = {
                    const ARG_COUNT: usize = $crate::count_args!($($args),*);
                    const EXPECTED: usize = #n;
                    if ARG_COUNT != EXPECTED {
                        panic!(#error_msg);
                    }
                };
            }
        }
        ArgCount::Multiple(counts) => {
            let counts_list = counts
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(" or ");
            let error_msg = format!("{} expects {} argument(s)", sql_name, counts_list);
            let count_checks = counts.iter().map(|c| {
                quote! { ARG_COUNT == #c }
            });
            quote! {
                const _: () = {
                    const ARG_COUNT: usize = $crate::count_args!($($args),*);
                    if !(#(#count_checks)||*) {
                        panic!(#error_msg);
                    }
                };
            }
        }
        ArgCount::Any => {
            quote! {
                // No validation needed for Any
            }
        }
    };

    let output = quote! {
        /// Generated SQL function struct
        #[doc = concat!("Represents the SQL function: ", #sql_name)]
        pub struct #struct_name;

        /// Macro for using the SQL function in queries
        #[macro_export]
        macro_rules! #macro_name {
            ($($args:expr),* $(,)?) => {{
                #arg_validation

                // Generate function call syntax that the parser will recognize
                // This creates an identifier followed by parentheses with arguments
                $crate::__sql_function_call!(#sql_name, $($args),*)
            }};
        }

        // Re-export the macro
        pub use #macro_name;
    };

    output.into()
}
