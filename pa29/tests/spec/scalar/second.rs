fn helper(n:u32)->u32{n+2}
fn id<T:Copy>(n:T)->T{n}
static VALUE:u32=5;
pub fn value(n:u32)->u32{id(helper(n))+one::value(1)+VALUE}
pub fn small(n:u8)->u8{id(n)}
pub fn real(n:f32)->f32{n}
