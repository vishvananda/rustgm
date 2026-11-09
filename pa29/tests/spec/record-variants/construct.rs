#![feature(no_core)]
#![no_core]
extern crate base;
// Record variants built by field name, with shorthand and reordered
// initializers, and matched by name with `..` and renamed bindings.
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let h = 2;
    let s = base::Shape::Rect { h, w: 40 };
    match s {
        base::Shape::Rect { w: 40, h: x } => match x {
            2 => match base::height(base::rect(1, 2)) {
                2 => match base::width(base::Shape::Empty {}) {
                    1 => match base::width(base::pair(3, 4)) { 3 => 42, _ => 5 },
                    _ => 4,
                },
                _ => 3,
            },
            _ => 2,
        },
        _ => 1,
    }
}
