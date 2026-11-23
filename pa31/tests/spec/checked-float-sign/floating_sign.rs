#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[inline(never)]
fn f32_bits(magnitude: u32, sign: u32) -> (u32, u32) {
    let value = f32::from_bits(magnitude);
    let sign = f32::from_bits(sign);
    (value.abs().to_bits(), value.copysign(sign).to_bits())
}

#[inline(never)]
fn f64_bits(magnitude: u64, sign: u64) -> (u64, u64) {
    let value = f64::from_bits(magnitude);
    let sign = f64::from_bits(sign);
    (value.abs().to_bits(), value.copysign(sign).to_bits())
}

fn check() -> i32 {
    let (absolute, signed) = f32_bits(0xffc0_23d7, 0);
    if absolute != 0x7fc0_23d7 || signed != 0x7fc0_23d7 {
        return 1;
    }
    let (absolute, signed) = f32_bits(0x3f80_0000, 0x8000_0000);
    if absolute != 0x3f80_0000 || signed != 0xbf80_0000 {
        return 2;
    }
    let (absolute, signed) = f64_bits(0x8000_0000_0000_0000, 0xfff8_0000_0000_0123);
    if absolute != 0 || signed != 0x8000_0000_0000_0000 {
        return 3;
    }
    let (absolute, signed) = f64_bits(0xfff8_0000_0000_1234, 0);
    if absolute != 0x7ff8_0000_0000_1234 || signed != 0x7ff8_0000_0000_1234 {
        return 4;
    }
    0
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    check()
}

#[cfg(not(guest_probe))]
fn main() {
    std::process::exit(check());
}
