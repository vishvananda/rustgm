
const fn through_blanket(value: u8) -> u8 {
    let iterator = base::IntoIterator::into_iter(Values { pending: true, value });
    iterator.value
}

const FROM_BLANKET: u8 = through_blanket(42u8);

#[unsafe(no_mangle)]
pub extern "C" fn observed_const() -> u8 { FROM_BLANKET }
