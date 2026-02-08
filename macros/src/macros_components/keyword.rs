use syn::{self, custom_punctuation};

macro_rules! sql_keyword {
    ($ident:ident,$struct_name:ident) => {
        #[allow(non_camel_case_types)]

        pub struct $struct_name {
            #[allow(dead_code)]
            pub span: syn::__private::Span,
        }

        #[doc(hidden)]
        #[allow(dead_code, non_snake_case)]

        pub fn $struct_name<__S: syn::__private::IntoSpans<syn::__private::Span>>(
            span: __S,
        ) -> $struct_name {
            $struct_name {
                span: syn::__private::IntoSpans::into_spans(span),
            }
        }

        const _: () = {
            impl syn::__private::Default for $struct_name {
                fn default() -> Self {
                    $struct_name {
                        span: syn::__private::Span::call_site(),
                    }
                }
            }

            impl_parse_for_custom_keyword!($ident, $struct_name);

            impl_to_tokens_for_custom_keyword!($ident, $struct_name);

            impl_clone_for_custom_keyword!($ident, $struct_name);

            impl_extra_traits_for_custom_keyword!($ident, $struct_name);
        };
    };
    ($ident:ident) => {
        #[allow(non_camel_case_types)]

        pub struct $ident {
            #[allow(dead_code)]
            pub span: syn::__private::Span,
        }

        #[doc(hidden)]
        #[allow(dead_code, non_snake_case)]

        pub fn $ident<__S: syn::__private::IntoSpans<syn::__private::Span>>(span: __S) -> $ident {
            $ident {
                span: syn::__private::IntoSpans::into_spans(span),
            }
        }

        const _: () = {
            impl syn::__private::Default for $ident {
                fn default() -> Self {
                    $ident {
                        span: syn::__private::Span::call_site(),
                    }
                }
            }

            impl_parse_for_custom_keyword!($ident);

            impl_to_tokens_for_custom_keyword!($ident);

            impl_clone_for_custom_keyword!($ident);

            impl_extra_traits_for_custom_keyword!($ident);
        };
    };
}

macro_rules! impl_parse_for_custom_keyword {
    ($ident:ident,$struct_name:ident) => {
        // For peek.

        impl syn::__private::CustomToken for $struct_name {
            fn peek(cursor: syn::buffer::Cursor) -> syn::__private::bool {
                if let syn::__private::Some((ident, _rest)) = cursor.ident() {
                    ident.to_string().to_lowercase() == stringify!($ident)
                } else {
                    false
                }
            }

            fn display() -> &'static str {
                syn::__private::concat!("`", syn::__private::stringify!($ident), "`")
            }
        }

        impl syn::parse::Parse for $struct_name {
            fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<$struct_name> {
                input.step(|cursor| {
                    if let syn::__private::Some((ident, rest)) = cursor.ident() {
                        if ident.to_string().to_lowercase() == syn::__private::stringify!($ident) {
                            return syn::__private::Ok(($struct_name { span: ident.span() }, rest));
                        }
                    }

                    syn::__private::Err(cursor.error(syn::__private::concat!(
                        "expected `",
                        syn::__private::stringify!($ident),
                        "`",
                    )))
                })
            }
        }
    };
    ($ident:ident) => {
        // For peek.

        impl syn::__private::CustomToken for $ident {
            fn peek(cursor: syn::buffer::Cursor) -> syn::__private::bool {
                if let syn::__private::Some((ident, _rest)) = cursor.ident() {
                    ident.to_string().to_lowercase() == stringify!($ident)
                } else {
                    false
                }
            }

            fn display() -> &'static str {
                syn::__private::concat!("`", syn::__private::stringify!($ident), "`")
            }
        }

        impl syn::parse::Parse for $ident {
            fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<$ident> {
                input.step(|cursor| {
                    if let syn::__private::Some((ident, rest)) = cursor.ident() {
                        if ident.to_string().to_lowercase() == syn::__private::stringify!($ident) {
                            return syn::__private::Ok(($ident { span: ident.span() }, rest));
                        }
                    }

                    syn::__private::Err(cursor.error(syn::__private::concat!(
                        "expected `",
                        syn::__private::stringify!($ident),
                        "`",
                    )))
                })
            }
        }
    };
}

macro_rules! impl_to_tokens_for_custom_keyword {
    ($ident:ident,$struct_name:ident) => {
        impl syn::__private::ToTokens for $struct_name {
            fn to_tokens(&self, tokens: &mut syn::__private::TokenStream2) {
                let ident = syn::Ident::new(
                    &syn::__private::stringify!($ident).to_uppercase(),
                    self.span,
                );

                syn::__private::TokenStreamExt::append(tokens, ident);
            }
        }
    };
    ($ident:ident) => {
        impl syn::__private::ToTokens for $ident {
            fn to_tokens(&self, tokens: &mut syn::__private::TokenStream2) {
                let ident = syn::Ident::new(
                    &syn::__private::stringify!($ident).to_uppercase(),
                    self.span,
                );

                syn::__private::TokenStreamExt::append(tokens, ident);
            }
        }
    };
}

macro_rules! impl_clone_for_custom_keyword {
    ($ident:ident,$struct_name:ident) => {
        impl syn::__private::Copy for $struct_name {}

        #[allow(clippy::expl_impl_clone_on_copy)]

        impl syn::__private::Clone for $struct_name {
            fn clone(&self) -> Self {
                *self
            }
        }
    };
    ($ident:ident) => {
        impl syn::__private::Copy for $ident {}

        #[allow(clippy::expl_impl_clone_on_copy)]

        impl syn::__private::Clone for $ident {
            fn clone(&self) -> Self {
                *self
            }
        }
    };
}

macro_rules! impl_extra_traits_for_custom_keyword {
    ($ident:ident,$struct_name:ident) => {
        impl syn::__private::Debug for $struct_name {
            fn fmt(&self, f: &mut syn::__private::Formatter) -> syn::__private::FmtResult {
                syn::__private::Formatter::write_str(
                    f,
                    syn::__private::concat!(
                        "Sql Keyword [",
                        syn::__private::stringify!($ident),
                        "]",
                    ),
                )
            }
        }

        impl syn::__private::Eq for $struct_name {}

        impl syn::__private::PartialEq for $struct_name {
            fn eq(&self, _other: &Self) -> syn::__private::bool {
                true
            }
        }

        impl syn::__private::Hash for $struct_name {
            fn hash<__H: syn::__private::Hasher>(&self, _state: &mut __H) {}
        }
    };
    ($ident:ident) => {
        impl syn::__private::Debug for $ident {
            fn fmt(&self, f: &mut syn::__private::Formatter) -> syn::__private::FmtResult {
                syn::__private::Formatter::write_str(
                    f,
                    syn::__private::concat!(
                        "Sql Keyword [",
                        syn::__private::stringify!($ident),
                        "]",
                    ),
                )
            }
        }

        impl syn::__private::Eq for $ident {}

        impl syn::__private::PartialEq for $ident {
            fn eq(&self, _other: &Self) -> syn::__private::bool {
                true
            }
        }

        impl syn::__private::Hash for $ident {
            fn hash<__H: syn::__private::Hasher>(&self, _state: &mut __H) {}
        }
    };
}

custom_punctuation!(DoubleArrow,->>);

sql_keyword!(distinct);
sql_keyword!(where, where_);
sql_keyword!(order);
sql_keyword!(group);
sql_keyword!(by);
sql_keyword!(having);
sql_keyword!(limit);
sql_keyword!(set);

sql_keyword!(and);
sql_keyword!(or);
sql_keyword!(not);
sql_keyword!(in, in_);
sql_keyword!(like);
sql_keyword!(between);
sql_keyword!(is);
sql_keyword!(null);

sql_keyword!(asc);
sql_keyword!(desc);
sql_keyword!(as, as_kw);

sql_keyword!(inner);
sql_keyword!(left);
sql_keyword!(right);
sql_keyword!(cross);
sql_keyword!(join);
sql_keyword!(on);

sql_keyword!(select);
sql_keyword!(from);
sql_keyword!(insert);
sql_keyword!(into);
sql_keyword!(values);
sql_keyword!(update);
sql_keyword!(delete);
sql_keyword!(returning);
sql_keyword!(exists);
