type Erased<const N: usize> = u8;
type Defaulted<const N: usize = 9> = Erased<N>;

fn fail(value: u8) -> u8 {
    1 / value
}

fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}

fn main() {
    let first: Erased<_> = 7u8;
    let second: Defaulted<_> = 9u8;
    check(first + second == 16u8);
}
