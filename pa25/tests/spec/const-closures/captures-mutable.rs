#![feature(const_closures, const_trait_impl, const_destruct)]
const fn compute(mut value:u8)->u8 {
    let mut call=const |extra:u8| {value+=extra;value};
    let first=call(2);
    call(first)
}
pub const ANSWER:u8=compute(11);
