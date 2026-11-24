pub mod pin {
    use crate::{ops::Deref, LegacyReceiver};

    #[lang = "pin"]
    #[repr(transparent)]
    pub struct Pin<Ptr> {
        pointer: Ptr,
    }

    impl<Ptr: Deref> Pin<Ptr> {
        #[lang = "new_unchecked"]
        pub const unsafe fn new_unchecked(pointer: Ptr) -> Pin<Ptr> {
            Pin { pointer }
        }

        pub unsafe fn take_inner(pin: Pin<Ptr>) -> Ptr {
            pin.pointer
        }
    }

    impl<Ptr: LegacyReceiver> LegacyReceiver for Pin<Ptr> {}
}
