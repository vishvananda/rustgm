struct Count(i32);
impl core::ops::Neg for Count {
    type Output = i32;
    fn neg(self) -> i32 { -self.0 }
}
fn main() { let _ = -Count(7); }
