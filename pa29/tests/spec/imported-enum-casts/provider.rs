const fn tag() -> u8 {
    16
}

#[repr(u8)]
pub enum Header {
    Explicit = tag(),
    Next,
}

#[repr(u8)]
pub enum EmptyShapes {
    Tuple(),
    Record {},
}

#[repr(u8)]
pub enum Payload {
    Data(u8),
    Empty,
}

#[repr(u8)]
pub enum ExplicitEmpty {
    Tuple() = 7,
    Unit,
}
