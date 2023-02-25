use proc_macro::TokenStream;

#[proc_macro_derive(Builder)]
pub fn my_builder(_input: TokenStream) -> TokenStream {
    // 如果返回input，那么会导致the name `Command` is defined multiple times
    // 因为原来的文件中的内容struct是不变的(derive增加内容)
    // input

    TokenStream::new()
}
