fn add<T: core::ops::Add<u8, Output = u16>>(x: T) -> u16 { x + 4u8 }
struct Count(u16);
impl core::ops::Add<u8> for Count {
    type Output = u16;
    fn add(self, rhs: u8) -> u16 { self.0 + rhs as u16 }
}
fn main() { let _ = add(Count(3)); }
