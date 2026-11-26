pub const FREE: [u8; 2] = {
    const INNER: [u8; 2] = [7, 11];
    INNER
};

pub struct Carrier;

impl Carrier {
    pub const ASSOCIATED: [u8; 2] = {
        const INNER: [u8; 2] = [13, 17];
        INNER
    };
}
