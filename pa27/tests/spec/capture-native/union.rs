union U{a:u32,b:u32}fn main(){let mut u=U{a:7};let mut write=||{u.b=9;};write();let read=||unsafe{u.a};check(read()==9);check(read()==9);let raw=||&raw const u.a;check(unsafe{*raw()}==9);}
