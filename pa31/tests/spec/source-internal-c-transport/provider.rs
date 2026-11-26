#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[repr(C)]
struct Pair {
    left: u8,
    right: u8,
}

const extern "C" fn select(value: Pair) -> u8 {
    match value.right {
        0 => value.left,
        _ => value.right,
    }
}

pub const fn read_pair<const FORCE: u8>(left: u8, right: u8) -> u8 {
    let selected = select(Pair { left, right });
    match FORCE {
        0 => selected,
        _ => FORCE,
    }
}

const _: u8 = read_pair::<0>(7, 0);
