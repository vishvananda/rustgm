#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

type Payment<'a> = &'a dyn Fn(usize, usize) -> Result<(), &'static str>;
type Eligible<'a> = &'a dyn Fn(usize) -> Result<bool, &'static str>;

struct Package { allowance: usize }
impl Package {
    fn pay(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        if steps <= self.allowance && bytes == 0 { Ok(()) } else { Err("exhausted") }
    }
}

struct Checker<'a> { package: &'a Package }
impl Checker<'_> {
    fn decide(&mut self, id: usize) -> Result<bool, &'static str> {
        let pay = |steps, bytes| self.package.pay(steps, bytes);
        let eligible = |id| registered_layout(id, &pay);
        run_both(id, &eligible, &pay)
    }
}

fn registered_layout(id: usize, pay: Payment<'_>) -> Result<bool, &'static str> {
    pay(id, 0)?;
    Ok(true)
}

fn run_both(id: usize, eligible: Eligible<'_>, pay: Payment<'_>) -> Result<bool, &'static str> {
    pay(0, 0)?;
    eligible(id)
}

fn experiment() -> i32 {
    let owner = Package { allowance: 42 };
    let mut checker = Checker { package: &owner };
    match checker.decide(42) { Ok(true) => 0, _ => 1 }
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! { loop {} }

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 { experiment() }

#[cfg(not(guest_probe))]
fn main() { assert_eq!(experiment(), 0); }
