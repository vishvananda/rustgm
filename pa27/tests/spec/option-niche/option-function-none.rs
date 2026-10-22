union U{o:Option<fn()->u32>,b:[u8;8]}const N:Option<fn()->u32>=unsafe{(U{b:[0;8]}).o};fn main(){check(match N{None=>true,Some(_)=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
