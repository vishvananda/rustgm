#![no_std]
#![feature(lang_items)]
unsafe extern "Rust" {
    #[lang = "panic_impl"]
    fn declared_with_another_name(info: &core::panic::PanicInfo<'_>) -> !;
}
#[unsafe(no_mangle)]
pub fn reached_weak(info: &core::panic::PanicInfo<'_>) -> ! {
    unsafe { declared_with_another_name(info) }
}
