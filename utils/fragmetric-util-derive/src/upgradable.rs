use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use syn::{spanned::Spanned, Attribute, Error, Field, Fields, Ident, ItemStruct, Meta, Type};

use super::Result;

const REQUIRES_UPGRADABLE_FIELD: &str = "There must be an upgradable data field";
const DUPLICATE_UPGRADABLE_FIELD: &str = "Upgradable data field must be unique";
const INVALID_UPGRADABLE_FIELD_TYPE: &str = "Upgradable data field must be enum type

At this point, we only filter out tuple, array/slice, fn type, impl trait, trait object, infer type(_), reference, and raw pointer, but your code will eventually fail to compile except enum type";
const REQUIRES_ATTRIBUTES: &str = "Upgradable data field requires attributes

* `latest`: name of latest version
* `variant`: (optional) name of enum variant bounded with latest version
";

pub(crate) fn __derive_require_upgradable(input: ItemStruct) -> Result<TokenStream> {
    let struct_ident = input.ident;
    let struct_ident_span = struct_ident.span();
    let _upgradable_field = __search_upgradable_field(struct_ident_span, input.fields)?;
    let upgradable_field_type_ident = __into_field_type_ident(_upgradable_field.ty)?;
    let upgradable_field_ident = _upgradable_field.ident.unwrap();
    let (latest_version_ident, variant_ident) =
        __into_upgradable_field_metadata(_upgradable_field.attrs)?;

    __generate_code(
        struct_ident,
        upgradable_field_ident,
        upgradable_field_type_ident,
        latest_version_ident,
        variant_ident,
    )
}

fn __generate_code(
    struct_ident: Ident,
    upgradable_field_ident: Ident,
    upgradable_field_type_ident: Ident,
    latest_version_ident: Ident,
    variant_ident: Ident,
) -> Result<TokenStream> {
    Ok(quote! {
        impl ::fragmetric_util::upgradable::__private::__AsMut<#latest_version_ident> for #struct_ident {
            fn __as_mut(&mut self) -> &mut #latest_version_ident {
                #[allow(unreachable_patterns)]
                match self.#upgradable_field_ident {
                    #upgradable_field_type_ident::#variant_ident(ref mut __inner) => __inner,
                    _ => unreachable!(),
                }
            }
        }

        impl ::fragmetric_util::upgradable::__private::__RequireUpgradable<#latest_version_ident> for #struct_ident {}
    })
}

fn __search_upgradable_field(struct_ident_span: Span, fields: Fields) -> Result<Field> {
    let mut iter = fields.into_iter().filter(__is_field_upgradable);
    let upgradable_field = iter
        .next()
        .ok_or_else(|| Error::new(struct_ident_span, REQUIRES_UPGRADABLE_FIELD))?;
    if let Some(field) = iter.next() {
        let attr_meta = __into_upgradable_attribute_metadata(field.attrs).unwrap();
        return Err(Error::new(attr_meta.span(), DUPLICATE_UPGRADABLE_FIELD));
    }
    return Ok(upgradable_field);
}

fn __is_field_upgradable(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .filter_map(|attr| attr.meta.path().get_ident())
        .any(|ident| ident == "upgradable")
}

fn __into_upgradable_attribute_metadata(attrs: Vec<Attribute>) -> Option<Meta> {
    attrs
        .into_iter()
        .map(|attr| attr.meta)
        .filter(|meta| {
            meta.path()
                .get_ident()
                .is_some_and(|ident| ident == "upgradable")
        })
        .next()
}

fn __into_field_type_ident(ty: Type) -> Result<Ident> {
    let ty_span = ty.span();
    let generate_error = || Error::new(ty_span, INVALID_UPGRADABLE_FIELD_TYPE);
    Ok(match &ty {
        syn::Type::Path(path) => path.path.get_ident().ok_or_else(generate_error)?.clone(),
        _ => return Err(generate_error()),
    })
}

fn __into_upgradable_field_metadata(attrs: Vec<Attribute>) -> Result<(Ident, Ident)> {
    let attr_meta = __into_upgradable_attribute_metadata(attrs).unwrap();
    let attr_meta_span = attr_meta.span();
    let attr_meta_tokens = attr_meta
        .require_list()
        .map_err(|_| Error::new(attr_meta_span, REQUIRES_ATTRIBUTES))?
        .tokens
        .to_owned();

    // Initialize the temporary storage with None
    let mut latest_version_ident = None::<Ident>;
    let mut variant_ident = None::<Ident>;

    let mut iter = attr_meta_tokens.into_iter();
    while let Some(token) = iter.next() {
        let ident = __expect_to_be_ident(token)?;
        if ident == "latest" {
            match &latest_version_ident {
                Some(_) => return Err(Error::new(ident.span(), "latest already provided")),
                None => {
                    let latest = __expect_followed_by_ident(ident.span(), &mut iter)?;
                    latest_version_ident = Some(latest);
                }
            }
        } else if ident == "variant" {
            match &variant_ident {
                Some(_) => return Err(Error::new(ident.span(), "variant already provided")),
                None => {
                    let variant = __expect_followed_by_ident(ident.span(), &mut iter)?;
                    variant_ident = Some(variant);
                }
            }
        } else {
            return Err(Error::new(ident.span(), "Invalid ident"));
        }

        // Expect comma separator or end-of-iterator to follow
        if let Some(token) = iter.next() {
            __expect_to_be_comma(token)?;
        } else {
            break;
        }
    }

    let latest_version_ident = match latest_version_ident {
        Some(ident) => ident,
        None => return Err(Error::new(attr_meta_span, "latest must be provided")),
    };
    let variant_ident = match variant_ident {
        Some(ident) => ident,
        None => latest_version_ident.clone(),
    };
    Ok((latest_version_ident, variant_ident))
}

fn __expect_to_be_ident(token: TokenTree) -> Result<Ident> {
    match token {
        TokenTree::Ident(ident) => Ok(ident),
        _ => Err(Error::new(token.span(), "Expected ident")),
    }
}

fn __expect_to_be_equal(token: TokenTree) -> Result<()> {
    match token {
        TokenTree::Punct(punct) if punct.as_char() == '=' => Ok(()),
        _ => Err(Error::new(token.span(), "Expected `=`")),
    }
}

fn __expect_followed_by_ident(
    span: Span,
    iter: &mut impl Iterator<Item = TokenTree>,
) -> Result<Ident> {
    const REQUIRES_IDENT: &str = "Requires `= <ident>`";
    let equal = iter
        .next()
        .ok_or_else(|| Error::new(span, REQUIRES_IDENT))?;
    __expect_to_be_equal(equal)?;
    let value = iter
        .next()
        .ok_or_else(|| Error::new(span, REQUIRES_IDENT))?;
    __expect_to_be_ident(value)
}

fn __expect_to_be_comma(token: TokenTree) -> Result<()> {
    match token {
        TokenTree::Punct(punct) if punct.as_char() == ',' => Ok(()),
        _ => Err(Error::new(token.span(), "Expected `,`")),
    }
}
