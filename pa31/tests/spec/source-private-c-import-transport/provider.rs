#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "sync"]
pub unsafe auto trait Sync {}
#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

mod runtime_symbols {
    unsafe extern "C" {
        #[lang = "strlen_fn"]
        fn strlen(s: *const i8) -> usize;
    }
}

pub(crate) mod imp {
    unsafe extern "C" {
        pub(crate) safe fn helper_diff(a: f64, b: f64) -> f64;
    }
}

pub fn measure_local_strlen(p: *const i8) -> usize {
    unsafe extern "C" {
        fn strlen(s: *const i8) -> usize;
    }
    unsafe { strlen(p) }
}

pub fn measure_module_safe_c(a: f64, b: f64) -> f64 {
    imp::helper_diff(a, b)
}
