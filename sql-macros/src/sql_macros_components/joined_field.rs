use easy_macros::syn;

pub struct JoinedField {
    pub field: syn::Field,
    pub table: syn::Path,
    pub table_field: syn::Ident,
}
