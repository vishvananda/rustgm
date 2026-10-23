static B:u8=65;union U{a:(*const u8,usize),b:*const str}const P:*const str=unsafe{U{a:(&B as *const u8,2)}.b};const S:&str=unsafe{&*P};fn main(){let _=S;}
