trait Fill<R: ?Sized> {
    fn fill(&self) -> u8
    where
        Self: Fill<Self>,
    {
        1
    }
}
impl Fill<u8> for u16 {}
impl Fill<u16> for u16 {}
fn main() {
    let _ = <u16 as Fill<u8>>::fill(&0);
}
