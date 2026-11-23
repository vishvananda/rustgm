#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

struct View<'a> {
    copies: &'a mut u8,
    body: &'a u8,
    permit: Option<&'a u8>,
}

impl<'a> View<'a> {
    fn checked(copies: &'a mut u8, body: &'a u8, proof: &'a u8) -> Result<Self, &'static str> {
        if body != proof {
            return Err("different body");
        }
        Ok(Self {
            copies,
            body,
            permit: None,
        })
    }

    fn generated(
        mut self,
        types: &u8,
        body: &'a u8,
        permit: Option<&'a u8>,
    ) -> Result<Self, &'static str> {
        if self.body != body {
            return Err("different generated body");
        }
        *self.copies += *types;
        self.permit = permit;
        Ok(self)
    }

    fn value(self) -> u8 {
        *self.copies + *self.body + self.permit.copied().unwrap_or(0)
    }
}

struct Environment<'a> {
    copies: &'a mut u8,
    types: &'a u8,
}

#[inline(never)]
fn initialize<'body>(
    environment: &mut Environment<'_>,
    body: &'body u8,
    proof: &u8,
    future: Option<&'body u8>,
    separate_copy: bool,
) -> Result<u8, &'static str> {
    let mut copies = separate_copy.then_some(4);
    let view = View::checked(copies.as_mut().unwrap_or(environment.copies), body, proof)
        .and_then(|view| view.generated(environment.types, body, future))?;
    Ok(view.value())
}

fn check() -> i32 {
    let mut copies = 2;
    let mut environment = Environment {
        copies: &mut copies,
        types: &10,
    };
    if initialize(&mut environment, &7, &7, Some(&13), false) != Ok(32) {
        return 1;
    }
    if initialize(&mut environment, &7, &7, None, true) != Ok(21) {
        return 2;
    }
    if initialize(&mut environment, &7, &6, None, false) != Err("different body") {
        return 3;
    }
    i32::from(copies != 12)
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
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
