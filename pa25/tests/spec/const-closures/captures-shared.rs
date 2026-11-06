#![feature(const_closures, const_trait_impl, const_destruct)]
const fn compute(value:u8)->u8 {
    let call=const |extra:u8| value+extra;
    call(2)
}
pub const ANSWER:u8=compute(21);
