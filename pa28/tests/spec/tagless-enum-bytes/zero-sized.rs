#[derive(Copy,Clone)]enum E{Only}union U{e:E,b:[u8;0]}const N:bool=unsafe{let src=U{e:E::Only};let mut dst=U{e:E::Only};core::ptr::copy_nonoverlapping(&raw const src,&raw mut dst,1);match dst.e{E::Only=>true}};fn main(){check(N)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
