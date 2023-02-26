use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, token::Struct, Data, DataStruct, DeriveInput, Error,
    Field, Fields, FieldsNamed, Lit, LitStr, Meta, MetaNameValue, Result,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn my_custom_debug(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_builder(input) {
        Ok(t) => TokenStream::from(t),
        Err(e) => TokenStream::from(e.to_compile_error()),
    }
}

fn derive_builder(mut input: DeriveInput) -> Result<TokenStream2> {
    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = input.data
    {
        let (input_name, input_name_str) = (&input.ident, &input.ident.to_string());
        let fields = named
            .iter()
            .map(|f| (f.ident.as_ref().expect("field name not found"), &f.ty));
        let field_ident = fields.clone().map(|(i, _)| i);
        let field_ident_str = field_ident.clone().map(|i| i.to_string());
        let field_rhs = field_ident
            .zip(named.iter().map(|f| f.attrs.as_slice()))
            .map(|(i, a)| attr_debug(a, i))
            .collect::<Result<Vec<_>>>()?;

        let expand = quote! {
            impl ::std::fmt::Debug for #input_name {
                fn fmt(&self, f: &mut::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
                    f.debug_struct(#input_name_str)
                    #(
                        .field(#field_ident_str, #field_rhs)
                    )*
                    .finish()
                }
            }
        };
        Ok(expand)
    } else {
        Err(Error::new(input.span(), "Expected `struct `only"))
    }
}

fn attr_debug(attrs: &[syn::Attribute], ident: &syn::Ident) -> Result<TokenStream2> {
    fn debug(attr: &syn::Attribute) -> Option<Result<LitStr>> {
        match attr.parse_meta() {
            Ok(Meta::NameValue(MetaNameValue {
                path,
                lit: Lit::Str(s),
                ..
            })) if path.is_ident("debug") => Some(Ok(s)),
            _ => Some(Err(syn::Error::new(
                attr.span(),
                "failed to parse attr meta",
            ))),
        }
    }

    match attrs.iter().find_map(|attr| debug(attr)) {
        None => Ok(quote! { &self.#ident}),
        Some(Ok(fmt)) => Ok(quote! { &::std::format_args!(#fmt, self.#ident)}),
        Some(Err(err)) => Err(err),
    }
}
