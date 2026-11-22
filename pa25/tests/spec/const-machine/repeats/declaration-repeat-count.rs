struct Carrier([u8; {
    let widths = [4usize; 1 + 2];
    widths[1]
}]);

fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}

fn fail(divisor: u8) -> u8 {
    1 / divisor
}

fn main() {
    let carrier = Carrier([2, 4, 6, 8]);
    check(carrier.0[3] == 8);
}
