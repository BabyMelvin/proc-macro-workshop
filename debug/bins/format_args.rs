use std::fmt;

fn main() {
    let debug = format!("{:?}", format_args!("{} foo {:?}", 1, 2));
    // "1 foo 2"
    println!("{:?}", debug);

    let s = fmt::format(format_args!("hello {}", "world"));
    assert_eq!(s, format!("hello {}", "world"));
}
