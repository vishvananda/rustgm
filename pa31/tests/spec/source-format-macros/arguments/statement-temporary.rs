struct Probe;
impl std::fmt::Debug for Probe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("x")
    }
}
impl Drop for Probe {
    fn drop(&mut self) { print!("drop:x;"); }
}
fn consume(args: std::fmt::Arguments<'_>) { print!("fmt:{args};"); }
fn main() {
    consume(format_args!("{:?}", Probe));
    println!("after");
}
