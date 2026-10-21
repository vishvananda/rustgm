struct Count(u8);
impl core::ops::Add for Count {
    type Output = Count;
    fn add(self, rhs: Count) -> Count { Count(self.0 + rhs.0) }
}
fn moved() -> u8 { let count = Count(3); let _ = count + Count(9); count.0 }
fn main() {}
