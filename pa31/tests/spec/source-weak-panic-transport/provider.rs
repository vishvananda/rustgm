pub fn forward(info: &PanicInfo<'_>) -> ! {
    unsafe extern "Rust" {
        #[lang = "panic_impl"]
        fn panic_impl(info: &PanicInfo<'_>) -> !;
    }
    unsafe { panic_impl(info) }
}
pub fn forward_again(info: &PanicInfo<'_>) -> ! {
    unsafe extern "Rust" {
        #[lang = "panic_impl"]
        fn panic_impl(info: &PanicInfo<'_>) -> !;
    }
    unsafe { panic_impl(info) }
}
