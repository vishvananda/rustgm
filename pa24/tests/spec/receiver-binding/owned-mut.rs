struct S(u8);
impl S {
    fn change(mut self) {
        let mut f = move || { self.0 += 1; };
        f();
    }
}
fn main() {
    let mut s = S(7);
    s.change();
}
