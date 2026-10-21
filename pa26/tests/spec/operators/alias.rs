use core::ops::Add as Sum;
struct Count(u8);
impl Sum for Count {
    type Output = Count;
    fn add(self, rhs: Count) -> Count { Count(self.0 + rhs.0) }
}
fn main() { let _ = Count(3) + Count(9); }
