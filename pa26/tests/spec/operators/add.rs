struct Count(u8);
impl core::ops::Add<u16> for Count {
    type Output = u16;
    fn add(self, rhs: u16) -> u16 { self.0 as u16 + rhs }
}
fn main() { let _ = Count(3) + 9u16; }
