#![feature(no_core)]
#![no_core]
extern crate base;

unsafe extern "C" {
    fn observe_text(text: *const u8, length: usize);
}

#[unsafe(no_mangle)]
pub extern "C" fn plain() {
    if base::selected!() {
        let text = base::format_args!("plain\n").text;
        unsafe { observe_text(text as *const str as *const u8, 6) }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn with_newline() {
    let text = base::format_args_nl!("line").text;
    unsafe { observe_text(text as *const str as *const u8, 5) }
}

const CONSTANT: base::TextArguments<'static> = base::const_format_args!("constant\n");

#[unsafe(no_mangle)]
pub extern "C" fn constant() {
    unsafe { observe_text(CONSTANT.text as *const str as *const u8, 9) }
}
