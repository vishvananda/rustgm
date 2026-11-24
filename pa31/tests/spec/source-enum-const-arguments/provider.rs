pub enum Order {
    First,
    Second,
}

impl Structural for Order {}
impl Equality for Order {}
impl Key for Order {}

pub fn selected<const MODE: Order>() -> u8 {
    let mode = MODE;
    match mode {
        Order::First => 37,
        Order::Second => 19,
    }
}

pub fn enum_first<T>() -> u8 {
    selected::<{ Order::First }>()
}

pub fn enum_second<T>() -> u8 {
    selected::<{ Order::Second }>()
}
