unsafe extern "C"{fn ignores_tail(tag:u32,...)->u32;}
const VALUE:u32=unsafe{ignores_tail(3,7u64)};fn main(){let _=VALUE;}
