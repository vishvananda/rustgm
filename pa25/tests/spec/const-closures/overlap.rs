#![feature(const_closures, const_trait_impl, const_destruct)]
pub const fn compute(mut value:u8)->u8 {
    let mut call=const || {value+=1;value};
    value=19;
    call()
}
