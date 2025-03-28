enum SqlClauseKeyword {
    Distinct,
    Where,
    OrderBy,
    GroupBy,
    Having,
    Limit,
}

enum SqlWhereKeyword {
    And,
    Or,
    Not,
    In,
    Like,
    Between,
    IsNull,
    IsNotNull,
}

enum SqlOrderByKeyword {
    Asc,
    Desc,
}

pub fn sql(item: proc_macro::TokenStream) -> proc_macro::TokenStream {}
