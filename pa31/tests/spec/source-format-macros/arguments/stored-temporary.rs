struct Probe;
impl std::fmt::Debug for Probe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("x")
    }
}
impl Drop for Probe {
    fn drop(&mut self) { print!("drop:x;"); }
}
fn main() {
    {
        let args = format_args!("{:?}", Probe);
        print!("fmt:{args};between;");
    }
    println!("after");
}
