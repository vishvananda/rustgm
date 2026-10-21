struct Pair(u16, u16);
impl core::ops::Index<bool> for Pair {
    type Output = u16;
    fn index(&self, second: bool) -> &u16 { if second { &self.1 } else { &self.0 } }
}
fn main() { let mut pair = Pair(3, 9); pair[true] = 7; }
