fn consume(args: std::fmt::Arguments<'_>) { println!("{args}"); }
fn main() { consume(format_args!("{:?}", { &String::from("owned") })); }
