use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, spanned::Spanned, AngleBracketedGenericArguments, Data, DataStruct,
    DeriveInput, Error, Fields, FieldsNamed, GenericArgument, Path, PathArguments, PathSegment,
    Type, TypePath,
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

        // 06
        let fields_setter_method = fields.clone().map(|(i, t)| {
            impl_fn(&vis, i, t)
        });

        Ok(quote! {
            #vis struct #builder_name {
                #(#builder_fields),*
            }

            impl #builder_name {
                #(#fields_setter_method)*

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

fn impl_fn(vis: &syn::Visibility, ident: &Ident, mut ty: &Type) -> proc_macro2::TokenStream {

}
}