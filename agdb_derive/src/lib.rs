use proc_macro::TokenStream;

#[proc_macro_derive(DbUserValue)]
pub fn db_user_value_derive(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
