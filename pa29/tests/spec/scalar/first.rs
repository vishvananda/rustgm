fn helper(n:u32)->u32{n+1}
fn id<T:Copy>(n:T)->T{n}
static VALUE:u32=3;
pub fn value(n:u32)->u32{id(helper(n))+VALUE}
pub fn small(n:u8)->u8{id(n)}
pub fn entry(){}
pub fn signed(n:i128)->i128{n}
pub fn truth(n:bool)->bool{n}
pub fn character(n:char)->char{n}
pub fn real(n:f64)->f64{n}
pub mod nested{pub fn twice(n:u32)->u32{super::helper(n)+super::helper(n)}}
