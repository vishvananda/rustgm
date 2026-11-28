#[no_mangle]
pub extern "C" fn pa34_mixed_c_helper(x: i32) -> i32 { x * 2 }
extern "C" { #[link_name = "pa34_mixed_c_helper"] fn imported_c_helper(x: i32) -> i32; }
pub fn combine(x: i32) -> i32 { legacy_alias::legacy_offset(unsafe { imported_c_helper(x) }) }
