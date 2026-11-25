#![feature(no_core)]
#![no_core]
extern crate base;

const CHAINED: &base::Location<'static> = base::outer_tracked::<u32>();
const UNTRACKED: &base::Location<'static> = base::untracked_wrapper::<u32>();
const BRANCH_A: &base::Location<'static> = base::branch_caller(true);
const BRANCH_B: &base::Location<'static> = base::branch_caller(false);

#[unsafe(no_mangle)]
pub extern "C" fn chained_native_line() -> u32 {
    base::outer_tracked::<u32>().line()
}

#[unsafe(no_mangle)]
pub extern "C" fn chained_const_line() -> u32 {
    CHAINED.line()
}

#[unsafe(no_mangle)]
pub extern "C" fn untracked_native_line() -> u32 {
    base::untracked_wrapper::<u32>().line()
}

#[unsafe(no_mangle)]
pub extern "C" fn untracked_const_line() -> u32 {
    UNTRACKED.line()
}

#[unsafe(no_mangle)]
pub extern "C" fn branch_native_a() -> u32 {
    base::branch_caller(true).line()
}

#[unsafe(no_mangle)]
pub extern "C" fn branch_native_b() -> u32 {
    base::branch_caller(false).line()
}

#[unsafe(no_mangle)]
pub extern "C" fn branch_const_a() -> u32 {
    BRANCH_A.line()
}

#[unsafe(no_mangle)]
pub extern "C" fn branch_const_b() -> u32 {
    BRANCH_B.line()
}

#[unsafe(no_mangle)]
pub extern "C" fn trait_tracked_line() -> u32 {
    base::call_locate::<base::TrackedImpl>().line()
}

#[unsafe(no_mangle)]
pub extern "C" fn trait_untracked_line() -> u32 {
    base::call_locate::<base::UntrackedImpl>().line()
}
