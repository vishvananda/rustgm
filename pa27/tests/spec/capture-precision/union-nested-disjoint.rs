union U{a:u32,b:u8}struct Token(u32);struct S{u:U,free:Token}fn main(){let s=S{u:U{a:7},free:Token(1)};let c=||unsafe{s.u.a};let t=s.free;let _=c();let _=t.0;}
