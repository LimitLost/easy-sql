use syn::{self, parse::Lookahead1};

use super::keyword;

/// Checks if the next token starts a new clause in SQL.
pub fn next_clause_token(lookahead: &Lookahead1) -> bool {
    lookahead.peek(keyword::distinct)
        || lookahead.peek(keyword::where_)
        || lookahead.peek(keyword::having)
        || lookahead.peek(keyword::group)
        || lookahead.peek(keyword::order)
        || lookahead.peek(keyword::limit)
        || lookahead.peek(keyword::join)
        || lookahead.peek(keyword::inner)
        || lookahead.peek(keyword::left)
        || lookahead.peek(keyword::right)
        || lookahead.peek(keyword::cross)
        || lookahead.peek(syn::Token![,])
}
