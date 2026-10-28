use core::panic::PanicInfo;
unsafe extern "C" {
    fn evaluated(value: bool) -> bool;
    fn dropped();
    fn seen_handler(line: u32, column: u32, can_unwind: bool, backtrace: bool);
}
struct Owned;
impl core::ops::Drop for Owned {
    fn drop(&mut self) { unsafe { dropped() }; }
}
fn condition(value: bool) -> bool { unsafe { evaluated(value) } }
#[panic_handler]
fn handle(info: &PanicInfo<'_>) -> ! {
    if let Some(location) = info.location() {
        unsafe { seen_handler(location.line(), location.column(), info.can_unwind(), info.force_no_backtrace()) };
    }
    core::intrinsics::abort()
}
#[unsafe(no_mangle)]
pub extern "C" fn run(value: bool) -> u32 {
    let _owned = Owned;
    assert!(condition(value));
    7
}
