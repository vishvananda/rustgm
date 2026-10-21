struct S(u8);
impl S {
    fn change<'a>(&'a mut self) {
        let mut f = move || { let old = self; self = old; self.0 += 1; };
        f();
    }
}
fn main() {
    let mut s = S(7);
    s.change();
}
