#[repr(u8)]
pub enum Void {
    First,
    Second,
}

pub mod runtime {
    use super::Void;
    unsafe extern "C" {
        #[lang = "memcpy_fn"]
        pub fn memcpy(dest: *mut Void, src: *const Void, count: usize) -> *mut Void;
        #[lang = "memmove_fn"]
        pub fn memmove(dest: *mut Void, src: *const Void, count: usize) -> *mut Void;
        #[lang = "memset_fn"]
        pub fn memset(dest: *mut Void, value: i32, count: usize) -> *mut Void;
        #[lang = "memcmp_fn"]
        pub fn memcmp(left: *const Void, right: *const Void, count: usize) -> i32;
        #[lang = "bcmp_fn"]
        pub fn bcmp(left: *const Void, right: *const Void, count: usize) -> i32;
        #[lang = "strlen_fn"]
        #[link_name = "strlen"]
        pub fn text_length(text: *const i8) -> usize;
    }
}
pub fn marker() -> u8 {
    34
}
