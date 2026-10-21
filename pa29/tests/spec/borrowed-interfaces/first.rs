pub fn identity(value:&u8)->&u8{value}
pub fn choose<'a,'b>(first:&'a u8,_second:&'b u8)->&'a u8{first}
pub fn update<'a>(value:&'a mut u32,delta:u32)->&'a mut u32{*value+=delta;value}
pub fn constant()->&'static u8{&7}
pub unsafe fn guarded<'a>(value:&'a u8)->&'a u8{value}
pub fn deep<'a,'b>(value:&'a &'b u8)->&'b u8{*value}
pub fn reborrow<'a>(value:&'a mut u8)->&'a u8{&*value}
pub fn unit(value:&())->&(){value}
pub fn wide(value:&u128)->&u128{value}
pub fn real(value:&f64)->&f64{value}
