// Reduced Pin companion for the original Drop forwarding methods.
pub mod pin {
    use crate::{ops::Deref, LegacyReceiver};
    #[lang = "pin"]
    #[repr(transparent)]
    pub struct Pin<Ptr> { pointer: Ptr }
    impl<Ptr: Deref> Pin<Ptr> {
        #[lang = "new_unchecked"]
        pub const unsafe fn new_unchecked(pointer: Ptr) -> Pin<Ptr> {
            Pin { pointer }
        }
        pub unsafe fn into_inner_unchecked(pin: Pin<Ptr>) -> Ptr {
            pin.pointer
        }
    }
    impl<T: ?Sized> Deref for Pin<&mut T> {
        type Target = T;
        fn deref(&self) -> &T { self.pointer }
    }
    impl<Ptr: LegacyReceiver> LegacyReceiver for Pin<Ptr> {}
}

