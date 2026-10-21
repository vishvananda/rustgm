union U{a:u32,b:u32}const fn f()->u32{let mut u=U{a:7};let p=(&raw const u.a)as *mut u32;unsafe{let r=&mut *p;*r=37;u.b}}const X:u32=f();fn main(){check(X==37);}
