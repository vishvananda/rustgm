#![feature(rustc_attrs,staged_api)]
#![unstable(feature="tuple_default_fixture",issue="none")]
#[stable(feature = "saturating_int_impl", since = "1.74.0")]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
#[repr(transparent)]
#[rustc_diagnostic_item = "Saturating"]
pub struct Saturating<T>(#[stable(feature = "saturating_int_impl", since = "1.74.0")] pub T);
