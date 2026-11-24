#[repr(C)]
pub struct ProviderOwned { mark: u8 }
impl Drop for ProviderOwned {
    fn drop(&mut self) {
        self.mark = match self.mark {
            41 => 42,
            42 => 43,
            _ => 99,
        };
    }
}
