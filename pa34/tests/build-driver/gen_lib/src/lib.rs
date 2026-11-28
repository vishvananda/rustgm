include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[no_mangle]
pub extern "C" fn pa34_driver_scale(x: i32) -> i32 {
    x + 7
}

extern "C" {
    #[link_name = "pa34_driver_scale"]
    fn imported_scale(x: i32) -> i32;
}

#[cfg(generated_mode)]
pub fn tag_len() -> usize {
    match env!("PA34_TAG") {
        "build_ok" => 8,
        _ => 0,
    }
}

pub fn compute() -> i32 {
    unsafe { imported_scale(GENERATED_BASE) }
}
