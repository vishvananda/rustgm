fn put(out: &mut u8, value: u8) { *out += value; }
pub fn count(out: &mut u8) -> u8 {
    let mut add = || *out += 1;
    add();
    let mut write = |value| put(out, value);
    write(2);
    *out
}
pub struct Ident { pub ascii: u8, pub punycode: u8 }
impl Ident {
    pub fn total(&self) -> u8 { let sum = || self.ascii + self.punycode; sum() }
}
