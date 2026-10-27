fn escaped() -> std::fmt::Arguments<'static> {
    format_args!("{:?}", String::from("owned"))
}
fn main() { let _ = escaped(); }
