struct Count(u8);
impl core::ops::Add for Count {
    type Output = u16;
    fn add(self, rhs: Count) -> u8 { self.0 + rhs.0 }
}
fn main() {}
