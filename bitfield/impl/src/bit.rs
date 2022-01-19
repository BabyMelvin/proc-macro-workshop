use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

pub fn expand(input: syn::Item) -> TokenStream2 {
    match input {
        syn::Item::Struct(syn::ItemStruct { attrs,
                                            vis,
                                            ident,
                                            generics,
                                            fields: syn::Fields::Named(fields),
                                            .. }) => {
            let id = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
            let getter = id.clone().map(|i| format_ident!("get_{}", i));
            let setter = id.clone().map(|i| format_ident!("set_{}", i));

            let ty = fields.named.iter().map(|f| &f.ty);
            let ty2 = ty.clone();
            let width = {
                let ty = ty.clone();
                quote! { [#( <#ty as ::bitfield::Specifier>::BITS ),*] }
            };
            let len = fields.named.len();

            let sig_ty = ty.clone().map(|t| quote! { <#t as ::bitfield::Specifier>::T });
            let size = quote! { #( <#ty as ::bitfield::Specifier>::BITS as usize )+* };
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

            let range = 0..len;
            let acc_name = id.clone().map(|i| format_ident!("acc_{}", i));
            let acc_name2 = id.clone().map(|i| format_ident!("acc_{}", i));
            let acc_val = range.map(|n| {
                                   if n == 0 {
                                       quote! { 0 }
                                   } else {
                                       let idx = 0..n;
                                       quote! { #( Self::WIDTH[#idx] )+* }
                                   }
                               });

            let check_bits = fields.named.iter().filter_map(check_bits);

            quote! {
                #(#attrs)*
                #[repr(C)]
                #vis struct #ident #impl_generics #where_clause {
                    // 把原字段内容完全替换成 `data: [u8; #size]`
                    data: [u8; #size >> 3],
                }

                impl #impl_generics #ident #ty_generics #where_clause{
                    #(
                        #vis fn #getter (&self) -> #sig_ty {
                            // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=7e8b096e345dc86054814b095c9e3884
                            <#ty2 as ::bitfield::Specifier>::get::<{Self::#acc_name2}>(&self.data)
                        }

                        #vis fn #setter (&mut self, #id : #sig_ty) {
                            <#ty2 as ::bitfield::Specifier>::set::<{Self::#acc_name2}>(&mut self.data, #id)
                        }
                    )*

                    #vis fn new() -> Self {
                        Self { data: ::std::default::Default::default() }
                    }

                    const WIDTH: [usize; #len] = #width;

                    #(
                        #[allow(non_upper_case_globals)]
                        const #acc_name : usize = #acc_val;
                    )*
                }

                #( #check_bits )*
            }
        }
        _ => unimplemented!(),
    }
}

fn check_bits(f: &syn::Field) -> Option<TokenStream2> {
    fn meta_bits(attr: &syn::Attribute) -> Option<syn::Lit> {
        match attr.parse_meta().ok()? {
            syn::Meta::NameValue(syn::MetaNameValue { lit, path, .. }) if path.is_ident("bits") => Some(lit),
            _ => None,
        }
    }

    f.attrs.iter().find_map(meta_bits).map(|lit| {
        let e = &f.ty;
        quote::quote_spanned! {
            lit.span()=>
                const _ : [(); #lit] = [(); <#e as ::bitfield::Specifier>::BITS];
        }
    })
}
