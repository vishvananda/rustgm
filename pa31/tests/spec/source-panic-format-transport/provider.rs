
#[lang = "format_argument"]
#[repr(align(2))]
pub struct Argument<'a> {
    value: &'a u8,
}

#[lang = "format_arguments"]
pub struct Arguments<'a> {
    template: NonNull<u8>,
    args: NonNull<Argument<'a>>,
}

#[lang = "panic_info"]
pub struct PanicInfo<'a> {
    message: &'a Arguments<'a>,
    location: &'static Location<'static>,
    pub can_unwind: bool,
    force_no_backtrace: bool,
}

impl Clone for bool {
    fn clone(&self) -> Self { *self }
}

impl Copy for bool {}
