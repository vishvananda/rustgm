
#[unsafe(no_mangle)]
pub extern "C" fn escaped() -> u8 {
    let loan;
    {
        let mut values = Values { value: 0 };
        loan = base::Iterator::by_ref(&mut values);
    }
    loan.value
}
