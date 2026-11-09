#![feature(no_core)]
#![no_core]
extern crate base;
struct Local;
trait Mine {}
impl<T: base::Error> Mine for T {}
impl Mine for &mut Local {}
fn need<T: Mine>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<&mut Local>(); need::<base::Fault>(); 42 }
