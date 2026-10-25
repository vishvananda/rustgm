type Borrowed<'a, T, const N: usize> = (&'a T, [T; N]);

fn borrower<'a, U>(borrowed: &'a u8, value: U) -> U {
    value
}

fn mixed<'a, 'b, T, const N: usize>(value: &'b T, array: &'a [T; N]) -> &'b T {
    value
}

fn const_first<'a, const N: usize, T>(value: &'a [T; N]) -> &'a [T; N] {
    value
}

fn alias<'a>(value: Borrowed<'a, u8, 2>) -> Borrowed<'a, u8, 2> {
    value
}

fn main() {
    let byte = 3u8;
    let values = [4u8, 5u8];
    let _: u16 = borrower::<u16>(&byte, 9u16);
    let _: &u8 = mixed::<u8, 2>(&byte, &values);
    let _: &[u8; 2] = const_first::<2, u8>(&values);
    let _: Borrowed<'_, u8, 2> = alias((&byte, values));
}
