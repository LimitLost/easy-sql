use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::spanned::Spanned;

#[derive(Debug, Clone)]
pub enum ProvidedDrivers {
    Single(TokenStream),
    SingleWithChecks {
        driver: TokenStream,
        checks: Vec<TokenStream>,
    },
    MultipleWithConn {
        drivers: Vec<TokenStream>,
        conn: TokenStream,
    },
}

impl ProvidedDrivers {
    pub fn single_driver(&self) -> Option<&TokenStream> {
        match self {
            ProvidedDrivers::Single(driver) => Some(driver),
            _ => None,
        }
    }

    pub fn iter_for_checks(&self) -> ProvidedDriversIterator<'_> {
        match self {
            ProvidedDrivers::Single(_) => ProvidedDriversIterator::Single(self),
            ProvidedDrivers::SingleWithChecks { driver: _, checks } => {
                ProvidedDriversIterator::Multi {
                    iter: checks.iter(),
                }
            }
            ProvidedDrivers::MultipleWithConn { drivers, conn: _ } => {
                ProvidedDriversIterator::Multi {
                    iter: drivers.iter(),
                }
            }
        }
    }

    pub fn arguments(&self, sql_crate: &TokenStream) -> TokenStream {
        match self {
            ProvidedDrivers::Single(driver) | ProvidedDrivers::SingleWithChecks { driver, .. } => {
                quote_spanned! {driver.span()=>
                    #sql_crate::macro_support::DriverArguments::<#driver>::default()
                }
            }
            ProvidedDrivers::MultipleWithConn { drivers: _, conn } => {
                quote_spanned! {conn.span()=>
                    #sql_crate::macro_support::args_for_driver(&(#conn))
                }
            }
        }
    }

    pub fn identifier_delimiter(&self, sql_crate: &TokenStream) -> TokenStream {
        match self {
            ProvidedDrivers::Single(driver) | ProvidedDrivers::SingleWithChecks { driver, .. } => {
                quote_spanned! {driver.span()=>
                    <#driver as #sql_crate::Driver>::identifier_delimiter()
                }
            }
            ProvidedDrivers::MultipleWithConn { drivers: _, conn } => {
                quote_spanned! {conn.span()=>
                    #sql_crate::macro_support::driver_identifier_delimiter(& (#conn))
                }
            }
        }
    }

    pub fn query_add_selected(
        &self,
        sql_crate: &TokenStream,
        output_type: &syn::Type,
        table_type: &syn::Type,
    ) -> TokenStream {
        match self {
            ProvidedDrivers::Single(driver) | ProvidedDrivers::SingleWithChecks { driver, .. } => {
                quote_spanned! {output_type.span()=>
                    <#output_type as #sql_crate::Output<#table_type, #driver>>::select(&mut query);
                }
            }
            ProvidedDrivers::MultipleWithConn { drivers: _, conn } => {
                quote_spanned! {output_type.span()=>
                    #sql_crate::macro_support::query_add_selected::<#table_type, #output_type, _>(
                        &mut query,
                        &(#conn),
                    );
                }
            }
        }
    }

    pub fn query_add_selected_with_args(
        &self,
        sql_crate: &TokenStream,
        table_type: &syn::Type,
        output_type: &syn::Type,
        output_arg_tokens: Vec<TokenStream>,
    ) -> TokenStream {
        match self {
            ProvidedDrivers::Single(driver) | ProvidedDrivers::SingleWithChecks { driver, .. } => {
                quote! {
                    query.push_str(&<#output_type as #sql_crate::macro_support::OutputData<#table_type>>::SelectProvider::__easy_sql_select::<#driver>(
                        _easy_sql_d,
                        #(#output_arg_tokens),*
                    ));
                }
            }
            ProvidedDrivers::MultipleWithConn { conn, .. } => {
                quote! {
                    query.push_str(&<#output_type as #sql_crate::macro_support::OutputData<#table_type>>::SelectProvider::__easy_sql_select_driver_from_conn(
                        &(#conn),
                        _easy_sql_d,
                        #(#output_arg_tokens),*
                    ));
                }
            }
        }
    }

    pub fn query_insert_data(
        &self,
        sql_crate: &TokenStream,
        table_type: &syn::Type,
        to_insert: syn::Expr,
    ) -> TokenStream {
        match self {
            ProvidedDrivers::Single(driver) | ProvidedDrivers::SingleWithChecks { driver, .. } => {
                quote_spanned! {to_insert.span()=>
                    #sql_crate::macro_support::query_insert_data_selected_driver::<#table_type, #driver, _>(
                        #[allow(unused_braces)]
                        {#to_insert},
                        _easy_sql_args,
                    )
                }
            }
            ProvidedDrivers::MultipleWithConn { drivers: _, conn } => {
                quote_spanned! {to_insert.span()=>
                    #sql_crate::macro_support::query_insert_data::<#table_type, _, _>(
                        #[allow(unused_braces)]
                        {#to_insert},
                        _easy_sql_args,
                        &(#conn),
                    )
                }
            }
        }
    }

    pub fn query_update_data(
        &self,
        sql_crate: &TokenStream,
        table_type: &TokenStream,
        update_data: syn::Expr,
    ) -> TokenStream {
        match self {
            ProvidedDrivers::Single(driver) | ProvidedDrivers::SingleWithChecks { driver, .. } => {
                quote_spanned! {update_data.span()=>
                    #sql_crate::macro_support::query_update_data_selected_driver::<#table_type, #driver, _>(
                        {#update_data},
                        _easy_sql_args,
                        &mut query,
                        &mut current_arg_n,
                    )
                }
            }
            ProvidedDrivers::MultipleWithConn { drivers: _, conn } => {
                quote_spanned! {update_data.span()=>
                    #sql_crate::macro_support::query_update_data::<#table_type, _, _>(
                        #[allow(unused_braces)]
                        {#update_data},
                        _easy_sql_args,
                        &mut query,
                        &mut current_arg_n,
                        &(#conn),
                    )
                }
            }
        }
    }

    pub fn table_joins(&self, sql_crate: &TokenStream, table_type: &syn::Type) -> TokenStream {
        match self {
            ProvidedDrivers::Single(driver) | ProvidedDrivers::SingleWithChecks { driver, .. } => {
                quote_spanned! {table_type.span()=>
                    <#table_type as #sql_crate::Table<#driver>>::table_joins(&mut query);
                }
            }
            ProvidedDrivers::MultipleWithConn { drivers: _, conn } => {
                quote_spanned! {table_type.span()=>
                    #sql_crate::macro_support::driver_table_joins::<#table_type, _>(
                        &mut query,
                        & (#conn),
                    );
                }
            }
        }
    }

    pub fn parameter_placeholder_base(&self, sql_crate: &TokenStream) -> TokenStream {
        match self {
            ProvidedDrivers::Single(_) | ProvidedDrivers::SingleWithChecks { .. } => {
                quote! {}
            }
            ProvidedDrivers::MultipleWithConn { drivers: _, conn } => {
                quote_spanned! {conn.span()=>
                    let mut __easy_sql_parameter_placeholder =
                        #sql_crate::macro_support::driver_parameter_placeholder(&(#conn));
                }
            }
        }
    }

    pub fn parameter_placeholder_fn(
        &self,
        sql_crate: &TokenStream,
        quote_span: proc_macro2::Span,
    ) -> TokenStream {
        match self {
            ProvidedDrivers::Single(driver) | ProvidedDrivers::SingleWithChecks { driver, .. } => {
                quote_spanned! {quote_span=>
                    <#driver as #sql_crate::Driver>::parameter_placeholder
                }
            }
            ProvidedDrivers::MultipleWithConn {
                drivers: _,
                conn: _,
            } => {
                quote_spanned! {quote_span=>
                    __easy_sql_parameter_placeholder
                }
            }
        }
    }

    pub fn parameter_placeholder(
        &self,
        sql_crate: &TokenStream,
        span: proc_macro2::Span,
        before_param_n: impl ToTokens,
        current_param_n: impl ToTokens,
    ) -> TokenStream {
        let pp_fn = self.parameter_placeholder_fn(sql_crate, span);
        quote_spanned! {span=>
            #pp_fn(#before_param_n #current_param_n)
        }
    }

    pub fn type_info(&self, sql_crate: &TokenStream, field_type: &syn::Type) -> TokenStream {
        match self {
            ProvidedDrivers::Single(driver) | ProvidedDrivers::SingleWithChecks { driver, .. } => {
                quote_spanned! {field_type.span()=>
                    <#field_type as #sql_crate::macro_support::Type<
                        #sql_crate::macro_support::InternalDriver<#driver>
                    >>::type_info()
                }
            }
            ProvidedDrivers::MultipleWithConn { drivers: _, conn } => {
                quote_spanned! {field_type.span()=>
                    #sql_crate::macro_support::driver_type_info::<#field_type, _>(& (#conn))
                }
            }
        }
    }

    pub fn table_name<T: ToTokens + Spanned>(
        &self,
        sql_crate: &TokenStream,
        table_type: &T,
    ) -> TokenStream {
        match self {
            ProvidedDrivers::Single(driver) | ProvidedDrivers::SingleWithChecks { driver, .. } => {
                quote_spanned! {table_type.span()=>
                    <#table_type as #sql_crate::Table<#driver>>::table_name()
                }
            }
            ProvidedDrivers::MultipleWithConn { drivers: _, conn } => {
                quote_spanned! {table_type.span()=>
                    #sql_crate::macro_support::driver_related_table_name::<#table_type,_>(& (#conn))
                }
            }
        }
    }
}

pub enum ProvidedDriversIterator<'a> {
    Done,
    Single(&'a ProvidedDrivers),
    Multi {
        iter: std::slice::Iter<'a, TokenStream>,
    },
}
impl<'a> Iterator for ProvidedDriversIterator<'a> {
    type Item = &'a TokenStream;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ProvidedDriversIterator::Done => None,
            ProvidedDriversIterator::Single(provided) => {
                let result = match provided {
                    ProvidedDrivers::Single(driver) => Some(driver),
                    _ => unreachable!("Iterator in Single state but ProvidedDrivers is not Single"),
                };
                *self = ProvidedDriversIterator::Done;
                result
            }
            ProvidedDriversIterator::Multi { iter } => iter.next(),
        }
    }
}
