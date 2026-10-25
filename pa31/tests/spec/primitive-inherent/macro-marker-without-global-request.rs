#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]
macro_rules! owner {
    () => {
        impl u8 {
            #[rustc_allow_incoherent_impl]
            pub fn __lib020_marker(self) -> u8 { self }
        }
    };
}
owner!();
pub fn proof(value: u8) -> u8 { value.__lib020_marker() }
