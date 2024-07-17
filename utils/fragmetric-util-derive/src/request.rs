use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident};

use super::Result;

pub(crate) fn __request(request_ident: Ident, input: DeriveInput) -> Result<TokenStream> {
    let instruction_ident = &input.ident;
    Ok(quote! {
        #input

        impl fragmetric_util::request::__private::__IntoRequest<#request_ident> for #instruction_ident {}
    })
}
