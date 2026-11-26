pub struct Table {
    pub handler: fn(u8) -> u8,
}

pub fn table() -> &'static Table {
    static TABLE: Table = Table {
        handler: |value| {
            let local = value;
            local ^ 7
        },
    };
    &TABLE
}

pub const CLOSED: u8 = {
    let local = 6;
    local
};
