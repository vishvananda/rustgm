#[repr(C)]
pub struct Mixed {
    pub n: u64,
    pub x: f64,
}

#[repr(C)]
pub struct Large {
    pub cells: [u64; 5],
}

pub extern "C" fn add(a: u32, b: u32) -> u32 {
    a + b
}

pub extern "C" fn real(a: f64, b: f64) -> f64 {
    a + b
}

pub unsafe extern "C" fn guarded(v: u32) -> u32 {
    v + 1
}

#[unsafe(export_name = "rustgm_vm_exact")]
pub extern "C" fn exact(v: u32) -> u32 {
    v + 4
}

pub extern "C" fn bump_mixed(v: Mixed) -> Mixed {
    Mixed {
        n: v.n + 3,
        x: v.x + 2.5,
    }
}

pub extern "C" fn bump_large(mut v: Large) -> Large {
    v.cells[4] += 7;
    v
}

pub static IMM: u32 = 7;
pub static mut MUT: u32 = 11;
pub static REF: &u32 = &IMM;

unsafe extern "C" {
    pub safe static SAFE_EXT: u32;
    pub static mut MUT_EXT: u32;
    pub fn sum_va(count: i32, ...) -> i32;
}
