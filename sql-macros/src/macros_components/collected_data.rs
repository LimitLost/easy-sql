use proc_macro2::TokenStream;

use super::ProvidedDrivers;

pub struct CollectedData<'a> {
    pub format_str: &'a mut String,
    pub format_params: &'a mut Vec<TokenStream>,
    pub binds: &'a mut Vec<TokenStream>,
    pub checks: &'a mut Vec<TokenStream>,
    pub sql_crate: &'a TokenStream,
    pub driver: &'a ProvidedDrivers,
    pub current_param_n: &'a mut usize,
    pub before_param_n: &'a mut TokenStream,
    pub before_format: &'a mut Vec<TokenStream>,
    pub output_ty: Option<&'a TokenStream>,
    pub main_table_type: Option<&'a TokenStream>,
    pub types_driver_support_needed: &'a mut Vec<proc_macro2::TokenStream>,
}

impl<'a> CollectedData<'a> {
    pub fn new(
        format_str: &'a mut String,
        format_params: &'a mut Vec<TokenStream>,
        binds: &'a mut Vec<TokenStream>,
        checks: &'a mut Vec<TokenStream>,
        sql_crate: &'a TokenStream,
        driver: &'a ProvidedDrivers,
        current_param_n: &'a mut usize,
        before_param_n: &'a mut TokenStream,
        before_format: &'a mut Vec<TokenStream>,
        output_ty: Option<&'a TokenStream>,
        main_table_type: Option<&'a TokenStream>,
        types_driver_support_needed: &'a mut Vec<proc_macro2::TokenStream>,
    ) -> Self {
        Self {
            format_str,
            format_params,
            binds,
            checks,
            sql_crate,
            driver,
            current_param_n,
            before_param_n,
            before_format,
            output_ty,
            main_table_type,
            types_driver_support_needed,
        }
    }

    pub fn with_format_str_and_params<'b>(
        &'b mut self,
        format_str: &'b mut String,
        format_params: &'b mut Vec<TokenStream>,
    ) -> CollectedData<'b>
    where
        'a: 'b,
    {
        CollectedData {
            format_str,
            format_params,
            binds: self.binds,
            checks: self.checks,
            sql_crate: self.sql_crate,
            driver: self.driver,
            current_param_n: self.current_param_n,
            before_param_n: self.before_param_n,
            before_format: self.before_format,
            output_ty: self.output_ty,
            main_table_type: self.main_table_type,
            types_driver_support_needed: self.types_driver_support_needed,
        }
    }

    pub fn with_format_params<'b>(
        &'b mut self,
        format_params: &'b mut Vec<TokenStream>,
    ) -> CollectedData<'b>
    where
        'a: 'b,
    {
        CollectedData {
            format_str: self.format_str,
            format_params,
            binds: self.binds,
            checks: self.checks,
            sql_crate: self.sql_crate,
            driver: self.driver,
            current_param_n: self.current_param_n,
            before_param_n: self.before_param_n,
            before_format: self.before_format,
            output_ty: self.output_ty,
            main_table_type: self.main_table_type,
            types_driver_support_needed: self.types_driver_support_needed,
        }
    }
}
