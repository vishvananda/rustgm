#[repr(C)]
pub struct ProviderOwned<A, B> { first: A, second: B, mark: u8 }
impl<B, A> Drop for ProviderOwned<A, B> {
    fn drop(&mut self) {
        self.mark = match self.mark {
            41 => 42,
            42 => 43,
            _ => 99,
        };
    }
}
