#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

struct Payload<'a> {
    bytes: [u64; 3],
    dropped: &'a mut u8,
    marker: u8,
}

impl Drop for Payload<'_> {
    fn drop(&mut self) {
        let [first, second, third] = self.bytes;
        *self.dropped = if *self.dropped == 0 && first == 7 && second == 13 && third == 41 {
            self.marker
        } else {
            1
        };
    }
}

fn experiment() -> i32 {
    let mut old_dropped = 0;
    let mut new_dropped = 0;
    let mut slot = Payload {
        bytes: [7, 13, 41],
        dropped: &mut old_dropped,
        marker: 9,
    };
    let replacement = Payload {
        bytes: [7, 13, 41],
        dropped: &mut new_dropped,
        marker: 17,
    };
    let old = core::mem::replace(&mut slot, replacement);
    let () = core::mem::replace(&mut (), ());
    if old.marker != 9 || slot.marker != 17 {
        return 1;
    }
    drop(old);
    drop(slot);
    if old_dropped != 9 {
        return 2;
    }
    if new_dropped == 17 { 0 } else { 3 }
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
