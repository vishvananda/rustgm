const OBSERVED: u8 = relay::forwarded::<u8>(19);

fn fail(divisor: u8) -> u8 {
    1 / divisor
}

fn main() {
    if OBSERVED != 19 || relay::forwarded::<u16>(513) != 513 {
        let _ = fail(0);
    }
}
