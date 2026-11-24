#[repr(C)]
pub struct ProviderOwned { mark: u8 }
impl Drop for ProviderOwned {
    fn pin_drop(self: Pin<&mut Self>) {
        let receiver = unsafe { Pin::into_inner_unchecked(self) };
        receiver.mark = match receiver.mark {
            41 => 42,
            42 => 43,
            _ => 99,
        };
    }
}
