fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
union U{x:u32,y:u32}fn main(){let mut u=U{x:3};let p=&raw mut u.x;unsafe{*p=19;check(u.y==19);}}