#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::hint::black_box;

struct Payload<'a> {
    words: [u64; 3],
    dropped: &'a mut u8,
}

impl Drop for Payload<'_> {
    fn drop(&mut self) {
        let [first, second, third] = self.words;
        *self.dropped = if *self.dropped == 0 && first == 7 && second == 13 && third == 41 {
            23
        } else {
            1
        };
    }
}

fn experiment() -> i32 {
    if black_box(0x7f3e_u16) != 0x7f3e {
        return 1;
    }
    let mut dropped = 0;
    let payload = black_box(Payload {
        words: [7, 13, 41],
        dropped: &mut dropped,
    });
    let () = black_box(());
    let [first, second, third] = payload.words;
    if first != 7 || second != 13 || third != 41 {
        return 2;
    }
    drop(payload);
    if dropped == 23 { 0 } else { 3 }
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    experiment()
}

#[cfg(not(guest_probe))]
fn main() {
    assert_eq!(experiment(), 0);
}
