#![feature(rustc_attrs)]
#![allow(internal_features)]
unsafe extern "C" {
    #[rustc_as_ptr]
    fn marked_foreign()->*const u8;
}
