use super::keyword;
use ::syn::{self, parse::Parse};
use easy_macros::always_context;

syn::custom_punctuation!(NotEqualsMicrosoft,<>);

#[derive(Debug, Clone)]
pub enum Operator {
    ///AND Keyword
    And,
    ///OR Keyword
    Or,
    ///+
    Add,
    ///-
    Sub,
    ///*
    Mul,
    /// /
    Div,
    ///%
    Mod,
    /// ||
    Concat,
    ///-> or ->>
    JsonExtract,
    ///->>
    JsonExtractText,
    /// &
    BitAnd,
    /// |
    BitOr,
    /// <<
    BitShiftLeft,
    /// >>
    BitShiftRight,
    /// = or ==
    Equal,
    /// != or <>
    NotEqual,
    /// >
    GreaterThan,
    /// >=
    GreaterThanOrEqual,
    /// <
    LessThan,
    /// <=
    LessThanOrEqual,
    /// LIKE
    Like,
}

#[always_context]
impl Parse for Operator {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::and) {
            input.parse::<keyword::and>()?;
            Ok(Operator::And)
        } else if lookahead.peek(keyword::or) {
            input.parse::<keyword::or>()?;
            Ok(Operator::Or)
        } else if lookahead.peek(syn::Token![+]) {
            input.parse::<syn::Token![+]>()?;
            Ok(Operator::Add)
        } else if input.peek(syn::Token![-]) && input.peek2(syn::Token![>>]) {
            input.parse::<syn::Token![-]>()?;
            input.parse::<syn::Token![>>]>()?;
            Ok(Operator::JsonExtractText)
        } else if lookahead.peek(syn::Token![->]) {
            input.parse::<syn::Token![->]>()?;
            Ok(Operator::JsonExtract)
        } else if input.peek(syn::Token![-]) && input.peek2(syn::Token![>]) {
            input.parse::<syn::Token![-]>()?;
            input.parse::<syn::Token![>]>()?;
            Ok(Operator::JsonExtract)
        } else if lookahead.peek(syn::Token![-]) {
            input.parse::<syn::Token![-]>()?;
            Ok(Operator::Sub)
        } else if lookahead.peek(syn::Token![*]) {
            input.parse::<syn::Token![*]>()?;
            Ok(Operator::Mul)
        } else if lookahead.peek(syn::Token![/]) {
            input.parse::<syn::Token![/]>()?;
            Ok(Operator::Div)
        } else if lookahead.peek(syn::Token![%]) {
            input.parse::<syn::Token![%]>()?;
            Ok(Operator::Mod)
        } else if lookahead.peek(syn::Token![||]) {
            input.parse::<syn::Token![||]>()?;
            Ok(Operator::Concat)
        } else if lookahead.peek(syn::Token![&]) {
            input.parse::<syn::Token![&]>()?;
            Ok(Operator::BitAnd)
        } else if lookahead.peek(syn::Token![|]) {
            input.parse::<syn::Token![|]>()?;
            Ok(Operator::BitOr)
        } else if lookahead.peek(syn::Token![<<]) {
            input.parse::<syn::Token![<<]>()?;
            Ok(Operator::BitShiftLeft)
        } else if lookahead.peek(syn::Token![>>]) {
            input.parse::<syn::Token![>>]>()?;
            Ok(Operator::BitShiftRight)
        } else if lookahead.peek(syn::Token![=]) {
            input.parse::<syn::Token![=]>()?;
            Ok(Operator::Equal)
        } else if lookahead.peek(syn::Token![==]) {
            input.parse::<syn::Token![==]>()?;
            Ok(Operator::Equal)
        } else if lookahead.peek(syn::Token![!=]) {
            input.parse::<syn::Token![!=]>()?;
            Ok(Operator::NotEqual)
        } else if lookahead.peek(NotEqualsMicrosoft) {
            input.parse::<NotEqualsMicrosoft>()?;
            Ok(Operator::NotEqual)
        } else if lookahead.peek(syn::Token![>=]) {
            input.parse::<syn::Token![>=]>()?;
            Ok(Operator::GreaterThanOrEqual)
        } else if lookahead.peek(syn::Token![<=]) {
            input.parse::<syn::Token![<=]>()?;
            Ok(Operator::LessThanOrEqual)
        } else if lookahead.peek(syn::Token![<]) {
            input.parse::<syn::Token![<]>()?;
            Ok(Operator::LessThan)
        } else if lookahead.peek(syn::Token![>]) {
            input.parse::<syn::Token![>]>()?;
            Ok(Operator::GreaterThan)
        } else if lookahead.peek(keyword::like) {
            input.parse::<keyword::like>()?;
            Ok(Operator::Like)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NotChain {
    pub not_count: usize,
}

impl Parse for NotChain {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut not_count = 0;
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(keyword::not) {
                input.parse::<keyword::not>()?;
                not_count += 1;
            } else {
                break;
            }
        }
        Ok(NotChain { not_count })
    }
}

impl NotChain {
    pub fn into_query_string(self) -> String {
        let mut current_query = String::new();
        for _ in 0..self.not_count {
            current_query.push_str("NOT ");
        }
        current_query
    }
}

pub(super) fn starts_here(input: syn::parse::ParseStream) -> bool {
    let fork = input.fork();
    fork.parse::<Operator>().is_ok()
}
