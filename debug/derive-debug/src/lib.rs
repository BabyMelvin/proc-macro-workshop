use std::collections::HashSet;

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
mod bound;
mod generics;

use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, token::Struct, Data, DataStruct, DeriveInput,
    Error, Field, Fields, FieldsNamed, Lit, LitStr, Meta, MetaNameValue, Result,
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

        let mut associated = HashSet::with_capacity(8);
        let generics = &mut input.generics;
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let mut opt = bound::struct_attr(&input.attrs);
        let (mut bound_where_caluse, bound_generics) = opt.unwrap_or_default();
        let closure = |g: &mut syn::TypeParam| {
            generics::add_debug(
                g,
                named.iter().map(|f| &f.ty),
                &mut associated,
                &bound_generics,
            );
        };
        generics.type_params_mut().for_each(closure);
        let mut where_clause = where_clause
            .clone()
            .unwrap_or_else(|| parse_quote! { where });
        let convert =
            |ty: &syn::Type| -> syn::WherePredicate { parse_quote!(#ty: ::std::fmt::Debug) };
        bound_where_caluse.extend(associated.into_iter().map(convert));
        where_clause.predicates.extend(bound_where_caluse);

        let expand = quote! {
            impl #impl_generics ::std::fmt::Debug for #input_name #ty_generics #where_clause {
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
