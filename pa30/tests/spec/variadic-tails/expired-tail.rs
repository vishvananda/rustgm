unsafe extern "C"{fn ignores_tail(tag:u32,...)->u32;}
fn main(){unsafe{ignores_tail(3,{let x=7u64;&x})};}
