#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

struct Member {
    root: u8,
}

struct Package {
    member: Option<Member>,
    first: u8,
    second: u8,
}

impl Package {
    fn roots(&self) -> impl Iterator<Item = (u8, u8)> + '_ {
        let guests = self.member.as_ref().into_iter();
        guests
            .map(|member| member.root)
            .chain(Some(0))
            .map(|module| {
                let value = if module > 1 { self.second } else { self.first };
                (module, value)
            })
    }
}

fn experiment() -> i32 {
    let package = Package {
        member: Some(Member { root: 2 }),
        first: 7,
        second: 9,
    };
    let _roots = package.roots();
    0
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
