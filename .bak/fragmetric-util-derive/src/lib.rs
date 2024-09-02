use syn::{DeriveInput, Error, Ident, ItemStruct};

mod request;
mod upgradable;

type Result<T> = std::result::Result<T, Error>;

#[proc_macro_derive(RequireUpgradable, attributes(upgradable))]
pub fn derive_require_upgradable(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as ItemStruct);
    match upgradable::__derive_require_upgradable(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn request(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let request_ident = syn::parse_macro_input!(attr as Ident);
    let input = syn::parse_macro_input!(item as DeriveInput);
    match request::__request(request_ident, input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
    }
    .into()
}
