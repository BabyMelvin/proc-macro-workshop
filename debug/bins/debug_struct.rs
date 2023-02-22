use std::fmt::{Debug};

struct Foo {
    bar: i32,
    baz: String,
}

// impl Display for Foo {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }

impl Debug for Foo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Foo")
            .field("bar", &self.bar)
            .field("baz", &self.baz)
            .finish()
    }
}

fn main() {
    // Foo { bar: 10, baz: "Hello world" }
    println!(
        "{:?}",
        Foo {
            bar: 10,
            baz: "Hello world".to_string()
        }
    );
}
