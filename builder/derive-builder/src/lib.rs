use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, spanned::Spanned, AngleBracketedGenericArguments, Data, DataStruct,
    DeriveInput, Error, Fields, FieldsNamed, GenericArgument, Meta, MetaList, MetaNameValue,
    NestedMeta, Path, PathArguments, PathSegment, Type, TypePath,
};

#[proc_macro_derive(Builder)]
pub fn my_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match drive_builder(input) {
        Ok(token) => TokenStream::from(token),
        Err(e) => TokenStream::from(e.to_compile_error()),
    }
}

fn drive_builder(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = input.data
    {
        let (input_name, vis) = (input.ident, input.vis);
        let builder_name = format_ident!("{}Builder", input_name);
        let fields = named
            .iter()
            .map(|f| (f.ident.as_ref().expect("field name not found"), &f.ty));
        let idents = fields.clone().map(|(i, _)| i);
        let builder_fields = fields.clone().map(|(i, t)| {
            quote! {
                #i: ::core::option::Option<#t>
            }
        });
        let instance_builder = fields.clone().map(__new);

        let mut each_names = Vec::with_capacity(named.len());
        for field in named.iter() {
            if let Some(attr) = field.attrs.first() {
                each_names.push(__each_attr(attr)?)
            } else {
                each_names.push(None)
            }
        }

        let (more, impl_fns): (Vec<_>, Vec<_>) = fields
            .clone()
            .zip(each_names)
            .map(|((ident, ty), each_name)| match each_name {
                Some(name) => (&name != ident, impl_fn(&vis, ident, ty, Some(&name))),
                None => (false, impl_fn(&vis, ident, ty, None)),
            })
            .unzip();

        #[rustfmt::skip]
        let impl_fns_more = fields.zip(more)
        .filter_map(|((ident, ty), m)|{
                if m {
                    Some(impl_fn(&vis, ident, ty, None))
                } else {
                    None
                }
        });
        Ok(quote! {
            #vis struct #builder_name {
                #(#builder_fields),*
            }

            impl #builder_name {
                #(#impl_fns)*
                #(#impl_fns_more)*

                #vis fn build(&mut self) -> ::core::result::Result<#input_name, std::boxed::Box<dyn ::std::error::Error>> {
                    Ok(#input_name {
                        #(
                            #idents: self.#idents.take().ok_or_else(||
                                format!("`{}` is not set", stringify!(#idents))
                            )?
                        ),*
                    })
                }
            }

            impl #input_name {
                #vis fn builder() -> #builder_name {
                    #builder_name {
                        #(#instance_builder),*
                    }
                }
            }
        })
    } else {
        Err(Error::new(input.span(), "Named Struct Only :)"))
    }
}

enum CheckFieldType {
    Raw,
    Option,
    Vec,
}

fn check(ty: &mut &Type, vec_t: bool) -> CheckFieldType {
    if let Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon,
            segments,
        },
    }) = ty
    {
        if leading_colon.is_none() && segments.len() == 1 {
            if let Some(PathSegment {
                ident,
                arguments:
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
            }) = segments.first()
            {
                if let (1, Some(GenericArgument::Type(t))) = (args.len(), args.first()) {
                    if ident == "Option" {
                        *ty = t;
                        return CheckFieldType::Option;
                    } else if ident == "Vec" {
                        if vec_t {
                            *ty = t;
                        }
                        return CheckFieldType::Vec;
                    }
                }
            }
        }
    }

    CheckFieldType::Raw
}

/// 将原类型增加 T -> Option<T>
fn __new((i, mut t): (&Ident, &Type)) -> proc_macro2::TokenStream {
    match check(&mut t, false) {
        CheckFieldType::Raw => quote! {
            #i: ::core::option::Option::None
        },
        CheckFieldType::Option => quote! {
            #i: ::core::option::Option::Some(::core::option::Option::None)
        },
        CheckFieldType::Vec => quote! {
            #i: ::core::option::Option::None
        },
    }
}

fn impl_fn(
    vis: &syn::Visibility,
    ident: &Ident,
    mut ty: &Type,
    each_name: Option<&Ident>,
) -> proc_macro2::TokenStream {
    let vec_t = each_name.is_some();
    match check(&mut ty, vec_t) {
        CheckFieldType::Option => quote! {
            #vis fn #ident (&mut self, #ident : #ty) -> &mut Self {
                self.#ident = ::core::option::Option::Some(::core::option::Option::Some(#ident));
                self
            }
        },
        CheckFieldType::Vec if vec_t => {
            let each_name = each_name.expect("failed to get `each` name");
            quote! {
                #vis fn #each_name (&mut self, #each_name: #ty) -> &mut Self {
                    self.#ident.as_mut().map(|v|v.push(#each_name));
                    self
                }
            }
        }
        _ => quote! {
            #vis fn #ident (&mut self, #ident: #ty) -> &mut Self {
                self.#ident = ::core::option::Option::Some(#ident);
                self
            }
        },
    }
}

fn __each_attr(attr: &syn::Attribute) -> syn::Result<Option<Ident>> {
    let meta = attr.parse_meta()?;
    match &meta {
        syn::Meta::List(MetaList { path, nested, .. }) if path.is_ident("builder") => {
            if let Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue { lit, path, .. }))) =
                nested.first()
            {
                match lit {
                    syn::Lit::Str(s) if path.is_ident("each") => {
                        Ok(Some(format_ident!("{}", s.value())))
                    }
                    _ => Err(Error::new(meta.span(), "expected `builder(each = \"...\"`")),
                }
            } else {
                Err(Error::new(meta.span(), "expected `builder(each = \"...\"`"))
            }
        }
        _ => Ok(None),
    }
}
