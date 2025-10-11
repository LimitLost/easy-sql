use ::syn::{self, parse::Parse};

use crate::sql_macros_components::sql_expr::SqlExpr;

pub struct SetExpr {
    pub(crate) updates: Vec<(syn::Ident, SqlExpr)>,
}

impl Parse for SetExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut updates = Vec::new();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            let where_expr: SqlExpr = input.parse()?;
            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
            updates.push((ident, where_expr));
        }

        Ok(SetExpr { updates })
    }
}
