extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed(left: u8, right: u8) -> u8 {
    base::apply(base::Wrapped::<3>(left), &base::Wrapped::<3>(right))
}

#[unsafe(no_mangle)]
pub extern "C" fn other_instance(left: u8, right: u8) -> u8 {
    base::apply_through_tuple(base::Wrapped::<7>(left), &base::Wrapped::<7>(right))
}
