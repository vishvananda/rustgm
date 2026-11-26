extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn built(input: u8) -> u8 {
    let a = base::build::<base::Concrete>(input);
    let b = base::build_concrete(input ^ 7);
    if a.data == input { b.data } else { 255 }
}
