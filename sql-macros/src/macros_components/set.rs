use ::syn::{self, parse::Parse};

use super::{expr::Expr, next_clause::next_clause_token};

#[derive(Debug, Clone)]
pub struct SetExpr {
    pub(crate) updates: Vec<(syn::Ident, Expr)>,
}

impl Parse for SetExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut updates = Vec::new();

        while !input.is_empty() {
            // Check if we've hit a keyword like WHERE or RETURNING
            let lookahead = input.lookahead1();
            if next_clause_token(&lookahead) {
                break;
            }

            let ident: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            let where_expr: Expr = input.parse()?;
            
            // Add to updates BEFORE checking for comma
            updates.push((ident, where_expr));
            
            // Check for comma to continue, or next clause to end
            if !input.is_empty() {
                let lookahead = input.lookahead1();
                if lookahead.peek(syn::Token![,]) {
                    input.parse::<syn::Token![,]>()?;
                    // Continue to next iteration
                } else if next_clause_token(&lookahead) {
                    break;
                } else {
                    return Err(lookahead.error());
                }
            }
        }

        Ok(SetExpr { updates })
    }
}
