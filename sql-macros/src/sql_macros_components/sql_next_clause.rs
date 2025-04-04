use easy_macros::syn::parse::Lookahead1;

use super::sql_keyword;

/// Checks if the next token starts a new clause in SQL.
pub fn next_clause_token(lookahead: &Lookahead1) -> bool {
    lookahead.peek(sql_keyword::distinct)
        || lookahead.peek(sql_keyword::where_)
        || lookahead.peek(sql_keyword::having)
        || lookahead.peek(sql_keyword::group)
        || lookahead.peek(sql_keyword::order)
        || lookahead.peek(sql_keyword::limit)
}
