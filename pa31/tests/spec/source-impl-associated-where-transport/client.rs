#![feature(no_core)]
#![no_core]
extern crate base;

fn project<'a, 'b>(
    left: &'a u8,
    right: &'b u8,
) -> <base::Needle<'b> as base::Pattern>::Searcher<'a>
where
    'b: 'a,
{
    base::Pair(left, right)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed<'a>(left: &'a u8, right: &'a u8) -> &'a u8 {
    project(left, right).0
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_captured<'a>(left: &'a u8, right: &'a u8) -> &'a u8 {
    project(left, right).1
}

#[cfg(missing_family_input)]
pub fn missing_family_input<'a, 'b>(
    left: &'a u8,
    right: &'b u8,
) -> <base::Needle<'b> as base::Pattern>::Searcher<'a> {
    base::Pair(left, right)
}
