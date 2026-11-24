#![feature(no_core,full_drop_fixture,pin_ergonomics)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_ordinary(value: usize) {
    struct Watch<'a>(&'a mut u8);
    impl base::Drop for Watch<'_> {
        fn pin_drop(self: base::pin::Pin<&mut Self>) {
            let slot = &mut *unsafe { base::pin::Pin::into_inner_unchecked(self) }.0;
            *slot = match *slot { 41 => 73, _ => 79 };
        }
    }
    let _watch = Watch(unsafe { &mut *(value as *mut u8) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_glue(value: usize) {
    unsafe { base::drop_glue(&mut *(value as *mut base::ProviderOwned)); }
}
