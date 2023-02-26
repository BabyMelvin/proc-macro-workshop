use proc_macro2::Ident;
use syn::{punctuated::Punctuated, Attribute, Token, WherePredicate};

type PunctPreds = Punctuated<WherePredicate, Token!(,)>;
type PredsIdent = (PunctPreds, std::collections::HashSet<Ident>);
type OptPredsIdent = Option<PredsIdent>;

pub fn struct_attr(attr: &[Attribute]) -> OptPredsIdent {
    attr.iter()
        .find_map(|attr| attr.parse_meta().ok().and_then(search::debug))
}

mod search {
    use proc_macro2::Ident;
    use syn::{
        parse_quote, punctuated::Punctuated, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Path,
        PredicateType, Type, TypePath, WherePredicate,
    };

    use super::{OptPredsIdent, PunctPreds};

    pub fn debug(meta: Meta) -> OptPredsIdent {
        let debug: Path = parse_quote!(debug);
        if meta.path() == &debug {
            search_bound(meta)
        } else {
            None
        }
    }

    fn search_bound(meta: Meta) -> OptPredsIdent {
        if let Meta::List(MetaList { nested, .. }) = meta {
            nested.iter().find_map(predicate)
        } else {
            None
        }
    }

    fn predicate(m: &NestedMeta) -> OptPredsIdent {
        let bound: Path = parse_quote!(bound);
        match m {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. }))
                if path == &bound =>
            {
                if let Lit::Str(s) = lit {
                    let wp: PunctPreds = s.parse_with(Punctuated::parse_terminated).ok()?;
                    let set = wp.iter().filter_map(search_generics_ident).collect();
                    Some((wp, set))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn search_generics_ident(w: &WherePredicate) -> Option<Ident> {
        if let WherePredicate::Type(PredicateType {
            bounded_ty: Type::Path(TypePath { path, .. }),
            ..
        }) = w
        {
            path.segments.first().map(|seg| seg.ident.clone())
        } else {
            None
        }
    }
}
