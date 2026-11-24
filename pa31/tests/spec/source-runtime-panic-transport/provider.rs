// These roles use the pinned signatures with reduced diverging bodies.
#[track_caller]
#[lang = "panic_fmt"]
#[rustc_do_not_const_check]
#[rustc_const_stable_indirect]
pub const fn panic_fmt(_: Arguments<'_>) -> ! {
    loop {}
}

#[lang = "panic_nounwind"]
#[rustc_nounwind]
#[rustc_const_stable_indirect]
pub const fn panic_nounwind(_: &'static str) -> ! {
    loop {}
}

pub const fn same_signature(_: &'static str) -> ! {
    loop {}
}
