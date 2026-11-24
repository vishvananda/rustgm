// Use the original generic hook ABI with a reduced formatting body.
#[rustc_diagnostic_item = "Display"]
pub trait WriteDisplay: PointeeSized {
    fn fmt(&self) -> u8;
}

pub trait OtherDisplay: PointeeSized {}

impl WriteDisplay for u8 {
    fn fmt(&self) -> u8 {
        41
    }
}

impl WriteDisplay for &str {
    fn fmt(&self) -> u8 {
        19
    }
}

fn stop_display_at_runtime() -> ! {
    loop {}
}

#[track_caller]
#[lang = "panic_display"]
#[rustc_do_not_const_check]
#[rustc_const_stable_indirect]
pub const fn panic_display<T: WriteDisplay>(_: &T) -> ! {
    stop_display_at_runtime()
}

pub const fn ordinary_display<T: WriteDisplay>(_: &T) -> ! {
    loop {}
}

#[lang = "const_panic_fmt"]
#[rustc_const_stable_indirect]
pub const fn const_format_stop(_: Arguments<'_>) -> ! {
    loop {}
}
