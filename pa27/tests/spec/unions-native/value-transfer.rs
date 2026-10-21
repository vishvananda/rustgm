fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
union U{x:u32,y:u32}fn make()->U{U{x:17}}fn take(u:U)->u32{unsafe{u.y}}fn main(){check(take(make())==17);}