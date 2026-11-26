extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn field_observation(input: u8) -> u8 {
    let mut record = base::Record::<base::Concrete> { data: input };
    let old = base::read(&record);
    base::replace(&mut record, input ^ 2);
    if old == input {
        base::read(&record)
    } else {
        255
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn array_observation(left: u8, right: u8) -> u8 {
    let record = base::Record::<base::Bytes> {
        data: [left, right],
    };
    let [first, second] = base::bytes(&record);
    first ^ second
}
