unsafe extern "C"{fn ignores_tail(tag:u32,...)->u32;}
fn main(){let mut x=7u64;unsafe{ignores_tail(3,&mut x,&mut x)};}
