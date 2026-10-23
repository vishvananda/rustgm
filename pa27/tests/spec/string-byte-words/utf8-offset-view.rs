static B:[u8;3]=[65,0xc3,0xa9];union U{a:(*const u8,usize),b:*const str}const P:*const str=unsafe{U{a:((&raw const B as *const u8).add(1),2)}.b};const S:&str=unsafe{&*P};const N:u8=unsafe{*((S as *const str as *const u8).add(1))};fn main(){if N!=169{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
