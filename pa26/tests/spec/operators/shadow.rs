trait Add { type Output; fn add(self, rhs: Self) -> Self::Output; }
struct Count(u8);
impl Add for Count {
    type Output = u8;
    fn add(self, rhs: Count) -> u8 { self.0 + rhs.0 }
}
fn main() { let _ = Count(3) + Count(9); }
