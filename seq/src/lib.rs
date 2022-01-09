use proc_macro::TokenStream;
use proc_macro2::{Group, Ident, TokenStream as TokenStream2};
use syn::{parse::Parse, LitInt, Token};

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let seq = syn::parse_macro_input!(input as Seq);
    TokenStream::from(seq.expand())
}

#[allow(dead_code)]
struct Seq {
    ident:       Ident,
    in_token:    Token![in],
    lhs:         LitInt,
    dot2_token:  Token![..],
    eq_token:    Option<Token![=]>,
    rhs:         LitInt,
    brace_token: syn::token::Brace,
    tokens:      TokenStream2,
}

impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Seq { ident:       input.parse()?,
                 in_token:    input.parse()?,
                 lhs:         input.parse()?,
                 dot2_token:  input.parse()?,
                 eq_token:    input.parse().ok(),
                 rhs:         input.parse()?,
                 brace_token: syn::braced!(content in input),
                 tokens:      content.parse()?, })
    }
}

impl Seq {
    fn expand(self) -> TokenStream2 {
        let Seq { ident, lhs, rhs, tokens, eq_token, .. } = self;
        let buffer = syn::buffer::TokenBuffer::new2(tokens);
        let cursor = buffer.begin();
        let range = if eq_token.is_some() {
            (lhs.base10_parse().unwrap()..=rhs.base10_parse().unwrap()).into()
        } else {
            (lhs.base10_parse().unwrap()..rhs.base10_parse().unwrap()).into()
        };
        repeat::SeqToken::new(cursor, &ident, range).token_stream()
    }
}

mod range;
mod repeat;
mod replace;

// 把 Group 内的 TokenStream 替换掉（保留 delimiter 和 span）
fn new_group(g: &Group, ts: TokenStream2) -> Group {
    let mut group = Group::new(g.delimiter(), ts);
    group.set_span(g.span());
    group
}
