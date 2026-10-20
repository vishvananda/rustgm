fn main() {
    let f = |#[allow(unused)] value: u8| move |#[allow(unused)] next: u8| value + next;
    let _ = f(2)(3);
}
